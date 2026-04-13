from __future__ import annotations

from typing import Protocol

from app.domain.models import ExerciseCatalogItem, HealthProfileAttribute


class ToolDataProvider(Protocol):
    async def load_health_profile(self, user_id: int) -> list[HealthProfileAttribute]:
        """Returns the latest user health profile attributes for prompt context."""
        ...

    async def load_exercises_for_muscle_groups(
        self,
        user_id: int,
        muscle_groups: list[str],
    ) -> list[ExerciseCatalogItem]:
        """Returns deduplicated exercises available for the selected muscle groups."""
        ...

    async def query_workouts(self, user_id: int, arguments_json: str) -> str:
        """Returns formatted workout history text for the workout_query tool."""
        ...

    async def list_exercises(self, user_id: int, arguments_json: str) -> str:
        """Returns formatted exercise listing text for the exercise_list tool."""
        ...
