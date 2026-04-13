from __future__ import annotations

from datetime import date

from app.domain.models import ExerciseCatalogItem, ExerciseKind


def build_user_prompt_content(
    workout_date: date,
    muscle_groups: list[str],
    exercises: list[ExerciseCatalogItem],
    exercise_names: list[str],
    max_exercise_count: int,
) -> str:
    groups = ", ".join(muscle_groups)

    weighted_names = sorted(
        e.name for e in exercises if e.kind == ExerciseKind.WEIGHTED
    )
    bodyweight_names = sorted(
        e.name for e in exercises if e.kind == ExerciseKind.BODYWEIGHT
    )

    weighted_section = ""
    if weighted_names:
        weighted_section = (
            "Weighted exercises (MUST have a non-null weight_kg in every set):\n"
            + "\n".join(weighted_names)
            + "\n\n"
        )

    bodyweight_section = ""
    if bodyweight_names:
        bodyweight_section = (
            "Bodyweight exercises (set weight_kg to null in every set):\n"
            + "\n".join(bodyweight_names)
            + "\n\n"
        )

    all_names = "\n".join(exercise_names)

    return (
        f"Target workout date: {workout_date.isoformat()}\n"
        f"Muscle groups: {groups}\n"
        f"Maximum number of exercises (hard cap): {max_exercise_count}\n\n"
        f"{weighted_section}"
        f"{bodyweight_section}"
        "Allowed exercise names (use ONLY these exact strings in your final JSON output):\n"
        f"{all_names}\n\n"
        "Rules for weight_kg:\n"
        "- Weighted exercises: weight_kg MUST be a positive number (kg). "
        "If no historical data is available, prescribe a conservative but realistic working weight.\n"
        "- Bodyweight exercises: weight_kg MUST be null.\n\n"
        "Generate a workout plan for the given date. "
        f"The `exercises` array must contain at most {max_exercise_count} entries. "
        "Use the tools to check past workouts before prescribing loads."
    )
