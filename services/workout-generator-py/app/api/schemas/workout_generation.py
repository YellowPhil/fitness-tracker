from __future__ import annotations

from datetime import date

from pydantic import BaseModel, ConfigDict, Field


class GenerateWorkoutRequest(BaseModel):
    model_config = ConfigDict(extra="forbid")

    user_id: int
    date: date
    muscle_groups: list[str]
    max_exercise_count: int = Field(ge=1)


class GeneratedSetResponse(BaseModel):
    model_config = ConfigDict(extra="forbid")

    reps: int
    weight_kg: float | None


class GeneratedExerciseResponse(BaseModel):
    model_config = ConfigDict(extra="forbid")

    exercise_id: str
    exercise_name: str
    notes: str | None
    sets: list[GeneratedSetResponse]


class GenerateWorkoutResponse(BaseModel):
    model_config = ConfigDict(extra="forbid")

    workout_name: str | None
    exercises: list[GeneratedExerciseResponse]
