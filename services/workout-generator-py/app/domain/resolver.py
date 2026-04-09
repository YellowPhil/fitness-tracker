from __future__ import annotations

from app.application.errors import ExerciseResolutionError
from app.domain.models import (
    AiWorkoutResponse,
    ExerciseCatalogItem,
    GeneratedExercise,
    GeneratedWorkout,
    WorkoutSet,
)


def resolve_workout(
    payload: AiWorkoutResponse,
    exercises_by_lower_name: dict[str, ExerciseCatalogItem],
    max_exercise_count: int,
) -> GeneratedWorkout:
    generated_exercises: list[GeneratedExercise] = []

    for entry in payload.exercises:
        exercise = exercises_by_lower_name.get(entry.exercise_name.lower())
        if exercise is None:
            raise ExerciseResolutionError(f"Unknown exercise name: {entry.exercise_name}")

        sets = [WorkoutSet(reps=set_entry.reps, weight_kg=set_entry.weight_kg) for set_entry in entry.sets]
        generated_exercises.append(
            GeneratedExercise(
                exercise_id=exercise.exercise_id,
                exercise_name=exercise.name,
                notes=entry.notes,
                sets=sets,
            )
        )

    if len(generated_exercises) > max_exercise_count:
        raise ExerciseResolutionError(
            f"Model returned {len(generated_exercises)} exercises, exceeding max_exercise_count of {max_exercise_count}"
        )

    return GeneratedWorkout(workout_name=payload.workout_name, exercises=generated_exercises)
