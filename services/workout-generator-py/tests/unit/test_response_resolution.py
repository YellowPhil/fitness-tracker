import pytest

from app.application.errors import ExerciseResolutionError
from app.domain.models import AiWorkoutResponse, ExerciseCatalogItem, ExerciseKind
from app.domain.resolver import resolve_workout


def test_resolve_workout_maps_exercise_names_and_preserves_sets() -> None:
    payload = AiWorkoutResponse.model_validate(
        {
            "workout_name": "Chest Day",
            "exercises": [
                {
                    "exercise_name": "bench press",
                    "notes": "Controlled tempo",
                    "sets": [{"reps": 8, "weight_kg": 60.0}],
                }
            ],
        }
    )
    mapping = {
        "bench press": ExerciseCatalogItem(
            exercise_id="exercise-10",
            name="Bench Press",
            kind=ExerciseKind.WEIGHTED,
            muscle_group="Chest",
        )
    }

    result = resolve_workout(payload=payload, exercises_by_lower_name=mapping, max_exercise_count=3)

    assert result.workout_name == "Chest Day"
    assert result.exercises[0].exercise_id == "exercise-10"
    assert result.exercises[0].exercise_name == "Bench Press"
    assert result.exercises[0].sets[0].weight_kg == 60.0


def test_resolve_workout_rejects_unknown_exercise() -> None:
    payload = AiWorkoutResponse.model_validate(
        {
            "workout_name": None,
            "exercises": [{"exercise_name": "Unknown", "notes": None, "sets": [{"reps": 8, "weight_kg": None}]}],
        }
    )

    with pytest.raises(ExerciseResolutionError) as exc:
        resolve_workout(payload=payload, exercises_by_lower_name={}, max_exercise_count=2)

    assert "Unknown exercise name" in str(exc.value)
