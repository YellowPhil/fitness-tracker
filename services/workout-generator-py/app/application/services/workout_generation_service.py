from __future__ import annotations

import json

import structlog
from structlog.contextvars import bind_contextvars

from app.application.errors import (
    AiEmptyResponseError,
    AiRefusalError,
    AiResponseFormatError,
    RequestValidationError,
)
from app.application.ports.ai_client import (
    AiChatClient,
    ChatMessage,
    CompletionRequest,
    CompletionResponse,
)
from app.application.ports.tool_data_provider import ToolDataProvider
from app.application.services.tool_dispatcher import ToolDispatcher
from app.domain.constants import EXERCISE_LIST_TOOL, WORKOUT_QUERY_TOOL
from app.domain.models import (
    AiWorkoutResponse,
    ExerciseCatalogItem,
    GeneratedWorkout,
    GenerateWorkoutCommand,
)
from app.domain.prompt_builder import build_user_prompt_content
from app.domain.resolver import resolve_workout
from app.domain.response_schema import workout_response_schema
from app.domain.system_prompt import SYSTEM_PROMPT
from app.domain.tool_specs import build_exercise_list_tool, build_workout_query_tool

logger = structlog.get_logger(__name__)


class WorkoutGenerationService:
    def __init__(
        self,
        ai_client: AiChatClient,
        tool_dispatcher: ToolDispatcher,
        provider: ToolDataProvider,
        model: str,
        max_completion_tokens: int,
    ):
        self._ai_client = ai_client
        self._tool_dispatcher = tool_dispatcher
        self._provider = provider
        self._model = model
        self._max_completion_tokens = max_completion_tokens

    async def generate(self, command: GenerateWorkoutCommand) -> GeneratedWorkout:
        bind_contextvars(
            user_id=command.user_id,
            date=command.date.isoformat(),
        )
        self._validate_command(command)

        loaded_exercises = await self._provider.load_exercises_for_muscle_groups(
            user_id=command.user_id,
            muscle_groups=command.muscle_groups,
        )
        logger.debug(
            "exercises_loaded",
            count=len(loaded_exercises),
            muscle_groups=command.muscle_groups,
        )
        if not loaded_exercises:
            raise RequestValidationError(
                "No exercises found for the selected muscle groups"
            )

        sorted_names = sorted(item.name for item in loaded_exercises)
        by_lower_name = self._build_exercise_lower_map(loaded_exercises)

        user_prompt = build_user_prompt_content(
            workout_date=command.date,
            muscle_groups=command.muscle_groups,
            exercises=loaded_exercises,
            exercise_names=sorted_names,
            max_exercise_count=command.max_exercise_count,
        )

        initial_messages = [
            ChatMessage(role="system", content=SYSTEM_PROMPT),
            ChatMessage(role="user", content=user_prompt),
        ]

        tools = [
            build_workout_query_tool(command.muscle_groups),
            build_exercise_list_tool(command.muscle_groups),
        ]

        logger.debug(
            "ai_initial_request_sent",
            model=self._model,
            message_count=len(initial_messages),
        )

        initial_response = await self._ai_client.complete(
            CompletionRequest(
                model=self._model,
                max_completion_tokens=self._max_completion_tokens,
                messages=initial_messages,
                tools=tools,
                require_tool_choice=True,
            )
        )
        logger.debug(
            "ai_initial_response_received",
            has_tool_calls=bool(initial_response.tool_calls),
            tool_call_count=len(initial_response.tool_calls),
            has_content=initial_response.content is not None,
        )

        follow_up_messages = await self._build_follow_up_messages(
            user_id=command.user_id,
            prompt_prefix=initial_messages,
            initial_response=initial_response,
        )

        schema = workout_response_schema(sorted_names, command.max_exercise_count)
        logger.debug(
            "ai_followup_request_sent",
            message_count=len(follow_up_messages),
            has_schema=True,
        )
        follow_up_response = await self._ai_client.complete(
            CompletionRequest(
                model=self._model,
                max_completion_tokens=self._max_completion_tokens,
                messages=follow_up_messages,
                response_schema=schema,
            )
        )
        logger.debug(
            "ai_followup_response_received",
            has_refusal=follow_up_response.refusal is not None,
            content_length=len(follow_up_response.content or ""),
            has_content=follow_up_response.content is not None,
        )

        if follow_up_response.refusal:
            raise AiRefusalError(
                f"Model refused to generate workout: {follow_up_response.refusal}"
            )
        if not follow_up_response.content:
            raise AiEmptyResponseError(
                "OpenAI returned no message content and no refusal"
            )

        logger.debug(
            "structured_response_received",
            content_length=len(follow_up_response.content),
            preview=follow_up_response.content[:300],
        )

        try:
            raw_payload = json.loads(follow_up_response.content.strip())
            payload = AiWorkoutResponse.model_validate(raw_payload)
        except Exception as exc:
            raise AiResponseFormatError(
                f"Failed to parse workout JSON from model: {exc}"
            ) from exc

        generated_workout = resolve_workout(
            payload=payload,
            exercises_by_lower_name=by_lower_name,
            max_exercise_count=command.max_exercise_count,
        )

        prompt_tokens = 0
        completion_tokens = 0
        total_tokens = 0
        if initial_response.usage is not None:
            prompt_tokens += initial_response.usage.prompt_tokens
            completion_tokens += initial_response.usage.completion_tokens
            total_tokens += initial_response.usage.total_tokens
        if follow_up_response.usage is not None:
            prompt_tokens += follow_up_response.usage.prompt_tokens
            completion_tokens += follow_up_response.usage.completion_tokens
            total_tokens += follow_up_response.usage.total_tokens

        logger.info(
            "workout_generated",
            workout_name=generated_workout.workout_name,
            exercise_count=len(generated_workout.exercises),
            max_exercise_count=command.max_exercise_count,
            prompt_tokens=prompt_tokens,
            completion_tokens=completion_tokens,
            total_tokens=total_tokens,
        )

        return generated_workout

    async def _build_follow_up_messages(
        self,
        user_id: int,
        prompt_prefix: list[ChatMessage],
        initial_response: CompletionResponse,
    ) -> list[ChatMessage]:
        if not initial_response.tool_calls:
            assistant_text = initial_response.content or ""
            logger.debug("ai_skipped_tool_calls", content_length=len(assistant_text))
            return [
                *prompt_prefix,
                ChatMessage(role="assistant", content=assistant_text),
            ]

        tool_responses = await self._tool_dispatcher.dispatch(
            user_id, initial_response.tool_calls
        )
        if not tool_responses:
            raise AiEmptyResponseError(
                "Model requested tools but none could be executed"
            )

        assistant_with_tools = ChatMessage(
            role="assistant", tool_calls=[item.tool_call for item in tool_responses]
        )
        tool_messages = [
            ChatMessage(
                role="tool", content=item.content, tool_call_id=item.tool_call.id
            )
            for item in tool_responses
        ]
        return [*prompt_prefix, assistant_with_tools, *tool_messages]

    def _validate_command(self, command: GenerateWorkoutCommand) -> None:
        if not command.muscle_groups:
            raise RequestValidationError("muscle_groups must not be empty")
        if command.max_exercise_count < 1:
            raise RequestValidationError("max_exercise_count must be at least 1")

    def _build_exercise_lower_map(
        self,
        loaded_exercises: list[ExerciseCatalogItem],
    ) -> dict[str, ExerciseCatalogItem]:
        exercise_map: dict[str, ExerciseCatalogItem] = {}
        for exercise in loaded_exercises:
            key = exercise.name.lower()
            if key not in exercise_map:
                exercise_map[key] = exercise
        return exercise_map


def default_tool_names() -> list[str]:
    return [WORKOUT_QUERY_TOOL, EXERCISE_LIST_TOOL]
