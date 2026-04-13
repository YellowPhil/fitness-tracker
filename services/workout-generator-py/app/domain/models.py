from __future__ import annotations

from datetime import date
from enum import StrEnum

from pydantic import BaseModel, ConfigDict, Field


class ExerciseKind(StrEnum):
    WEIGHTED = "Weighted"
    BODYWEIGHT = "BodyWeight"


class ExerciseCatalogItem(BaseModel):
    model_config = ConfigDict(extra="forbid")

    exercise_id: str
    name: str
    kind: ExerciseKind
    muscle_group: str


class HealthProfileAttribute(BaseModel):
    model_config = ConfigDict(extra="forbid")

    key: str
    value: str
    unit: str | None


class WorkoutSet(BaseModel):
    model_config = ConfigDict(extra="forbid")

    reps: int = Field(ge=1)
    weight_kg: float | None


class GeneratedExercise(BaseModel):
    model_config = ConfigDict(extra="forbid")

    exercise_id: str
    exercise_name: str
    notes: str | None
    sets: list[WorkoutSet]


class GenerateWorkoutCommand(BaseModel):
    model_config = ConfigDict(extra="forbid")

    user_id: int
    date: date
    muscle_groups: list[str]
    max_exercise_count: int = Field(ge=1)


class GeneratedWorkout(BaseModel):
    model_config = ConfigDict(extra="forbid")

    workout_name: str | None
    exercises: list[GeneratedExercise]


class AiSetEntry(BaseModel):
    model_config = ConfigDict(extra="forbid")

    reps: int = Field(ge=1)
    weight_kg: float | None


class AiExerciseEntry(BaseModel):
    model_config = ConfigDict(extra="forbid")

    exercise_name: str
    notes: str | None
    sets: list[AiSetEntry]


class AiWorkoutResponse(BaseModel):
    model_config = ConfigDict(extra="forbid")

    workout_name: str | None
    exercises: list[AiExerciseEntry]
