from datetime import date

import pytest

from app.application.ports.ai_client import CompletionRequest, CompletionResponse, ToolCall
from app.application.services.tool_dispatcher import ToolDispatcher
from app.application.services.workout_generation_service import WorkoutGenerationService
from app.domain.models import (
    ExerciseCatalogItem,
    ExerciseKind,
    GenerateWorkoutCommand,
    HealthProfileAttribute,
    TrainingGoalPreference,
    WorkoutGenerationPreferences,
    WorkoutSplitPreference,
)
from app.infrastructure.tooling.registry import build_tool_registry


class FakeAiClient:
    def __init__(self):
        self.requests: list[CompletionRequest] = []

    async def complete(self, request: CompletionRequest) -> CompletionResponse:
        self.requests.append(request)
        if len(self.requests) == 1:
            return CompletionResponse(
                content=None,
                tool_calls=[
                    ToolCall(
                        id="call_1",
                        name="workout_query",
                        arguments='{"muscle_group":"Chest","last_n":2}',
                    ),
                    ToolCall(
                        id="call_2",
                        name="exercise_list",
                        arguments='{"muscle_group":"Chest"}',
                    ),
                ],
                refusal=None,
            )

        return CompletionResponse(
            content=(
                '{"workout_name":"Upper Push","exercises":['
                '{"exercise_name":"Bench Press","notes":"Start conservative",'
                '"sets":[{"reps":8,"weight_kg":60.0}]}'
                "]}"
            ),
            tool_calls=[],
            refusal=None,
        )


class FakeProvider:
    async def load_health_profile(self, user_id: int):
        return [
            HealthProfileAttribute(key="weight", value="82", unit="kg"),
            HealthProfileAttribute(key="height", value="180", unit="cm"),
            HealthProfileAttribute(key="age", value="31", unit=None),
        ]

    async def load_exercises_for_muscle_groups(self, user_id: int, muscle_groups: list[str]):
        return [
            ExerciseCatalogItem(
                exercise_id="exercise-11",
                name="Bench Press",
                kind=ExerciseKind.WEIGHTED,
                muscle_group="Chest",
            )
        ]

    async def load_workout_preferences(self, user_id: int):
        return WorkoutGenerationPreferences(
            max_sets_per_exercise=4,
            preferred_split=WorkoutSplitPreference.PUSH_PULL_LEGS,
            training_goal=TrainingGoalPreference.HYPERTROPHY,
            session_duration_minutes=75,
            notes="Avoid exercises that require a spotter",
        )

    async def query_workouts(self, user_id: int, arguments_json: str):
        return "Previous bench sessions available"

    async def list_exercises(self, user_id: int, arguments_json: str):
        return "Bench Press"


@pytest.mark.asyncio
async def test_generate_workout_two_step_flow() -> None:
    fake_ai = FakeAiClient()
    provider = FakeProvider()
    dispatcher = ToolDispatcher(build_tool_registry(provider))
    service = WorkoutGenerationService(
        ai_client=fake_ai,
        tool_dispatcher=dispatcher,
        provider=provider,
        model="gpt-test",
        max_completion_tokens=1000,
    )

    result = await service.generate(
        GenerateWorkoutCommand(
            user_id=5,
            date=date(2026, 4, 3),
            muscle_groups=["Chest"],
            max_exercise_count=3,
        )
    )

    assert result.workout_name == "Upper Push"
    assert len(result.exercises) == 1
    assert result.exercises[0].exercise_id == "exercise-11"
    assert len(fake_ai.requests) == 2
    assert fake_ai.requests[0].require_tool_choice is True
    initial_prompt = fake_ai.requests[0].messages[1].content
    assert initial_prompt is not None
    assert "Current health profile parameters" in initial_prompt
    assert "Current workout generation preferences (respect these when possible):" in initial_prompt
    assert "- preferred_split: PushPullLegs" in initial_prompt
    assert "- weight: 82 kg" in initial_prompt
    assert fake_ai.requests[1].response_schema is not None
