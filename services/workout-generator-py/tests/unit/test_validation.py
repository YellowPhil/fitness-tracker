from datetime import date

import pytest

from app.application.errors import RequestValidationError
from app.application.ports.ai_client import CompletionResponse
from app.application.services.tool_dispatcher import ToolDispatcher, ToolRegistry
from app.application.services.workout_generation_service import WorkoutGenerationService
from app.domain.models import (
    ExerciseCatalogItem,
    ExerciseKind,
    GenerateWorkoutCommand,
    WorkoutGenerationPreferences,
)


class FakeAiClient:
    async def complete(self, request):
        return CompletionResponse(content="{}", tool_calls=[], refusal=None)


class FakeProvider:
    async def load_health_profile(self, user_id: int):
        return []

    async def load_exercises_for_muscle_groups(self, user_id: int, muscle_groups: list[str]):
        return [
            ExerciseCatalogItem(
                exercise_id="exercise-1",
                name="Bench Press",
                kind=ExerciseKind.WEIGHTED,
                muscle_group="Chest",
            )
        ]

    async def load_workout_preferences(self, user_id: int):
        return WorkoutGenerationPreferences()

    async def query_workouts(self, user_id: int, arguments_json: str):
        return "ok"

    async def list_exercises(self, user_id: int, arguments_json: str):
        return "ok"


def build_service() -> WorkoutGenerationService:
    registry = ToolRegistry({"workout_query": FakeProvider().query_workouts, "exercise_list": FakeProvider().list_exercises})
    return WorkoutGenerationService(
        ai_client=FakeAiClient(),
        tool_dispatcher=ToolDispatcher(registry),
        provider=FakeProvider(),
        model="test-model",
        max_completion_tokens=1000,
    )


@pytest.mark.asyncio
async def test_generate_rejects_empty_muscle_groups() -> None:
    service = build_service()
    command = GenerateWorkoutCommand(
        user_id=7,
        date=date(2026, 4, 1),
        muscle_groups=[],
        max_exercise_count=2,
    )

    with pytest.raises(RequestValidationError) as exc:
        await service.generate(command)

    assert str(exc.value) == "muscle_groups must not be empty"


@pytest.mark.asyncio
async def test_generate_rejects_max_exercise_count_under_one() -> None:
    service = build_service()
    command = GenerateWorkoutCommand.model_construct(
        user_id=7,
        date=date(2026, 4, 1),
        muscle_groups=["Chest"],
        max_exercise_count=0,
    )

    with pytest.raises(RequestValidationError) as exc:
        await service.generate(command)

    assert str(exc.value) == "max_exercise_count must be at least 1"
