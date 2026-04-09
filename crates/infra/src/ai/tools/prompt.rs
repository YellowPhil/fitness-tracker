use domain::types::{Exercise, ExerciseKind, MuscleGroup};

pub(super) const SYSTEM_PROMPT: &str = include_str!("workout-programmer.md");

pub(super) fn build_user_message_content(
    date: time::Date,
    muscle_groups: &[MuscleGroup],
    exercises: &[Exercise],
    exercise_names: &[String],
    max_exercise_count: usize,
) -> String {
    let groups = muscle_groups
        .iter()
        .map(|g| g.to_string())
        .collect::<Vec<_>>()
        .join(", ");

    let mut weighted_names: Vec<&str> = exercises
        .iter()
        .filter(|e| matches!(e.kind, ExerciseKind::Weighted))
        .map(|e| e.name.as_str())
        .collect();
    weighted_names.sort_unstable();

    let mut bodyweight_names: Vec<&str> = exercises
        .iter()
        .filter(|e| matches!(e.kind, ExerciseKind::BodyWeight))
        .map(|e| e.name.as_str())
        .collect();
    bodyweight_names.sort_unstable();

    let weighted_section = if weighted_names.is_empty() {
        String::new()
    } else {
        format!(
            "Weighted exercises (MUST have a non-null weight_kg in every set):\n{}\n\n",
            weighted_names.join("\n")
        )
    };

    let bodyweight_section = if bodyweight_names.is_empty() {
        String::new()
    } else {
        format!(
            "Bodyweight exercises (set weight_kg to null in every set):\n{}\n\n",
            bodyweight_names.join("\n")
        )
    };

    let all_names = exercise_names.join("\n");

    format!(
        "Target workout date: {date}\n\
         Muscle groups: {groups}\n\
         Maximum number of exercises (hard cap): {max_exercise_count}\n\n\
         {weighted_section}\
         {bodyweight_section}\
         Allowed exercise names (use ONLY these exact strings in your final JSON output):\n\
         {all_names}\n\n\
         Rules for weight_kg:\n\
         - Weighted exercises: weight_kg MUST be a positive number (kg). \
           If no historical data is available, prescribe a conservative but realistic working weight \
           (e.g. Deadlift 60–100 kg, Barbell Row 40–70 kg, Bench Press 40–80 kg, Overhead Press 30–60 kg).\n\
         - Bodyweight exercises: weight_kg MUST be null.\n\n\
         Generate a workout plan for the given date. \
         The `exercises` array must contain at most {max_exercise_count} entries. \
         Use the tools to check past workouts before prescribing loads."
    )
}
