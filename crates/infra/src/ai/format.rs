use std::collections::HashMap;
use std::fmt::Write;

use domain::excercise::{ExerciseId, ExerciseMetadata, LoadType, MuscleGroup, Workout};

pub(super) fn format_workouts(
    workouts: &[Workout],
    exercises: &HashMap<ExerciseId, ExerciseMetadata>,
    filter: Option<MuscleGroup>,
) -> String {
    if workouts.is_empty() {
        return match filter {
            Some(group) => format!("No workouts found matching muscle group: {group}"),
            None => "No workouts found.".into(),
        };
    }

    let mut out = String::new();

    for (i, workout) in workouts.iter().enumerate() {
        if i > 0 {
            out.push_str("\n---\n\n");
        }
        format_workout(&mut out, workout, exercises);
    }

    out
}

fn format_workout(
    out: &mut String,
    workout: &Workout,
    exercises: &HashMap<ExerciseId, ExerciseMetadata>,
) {
    let name = workout.name.as_deref().unwrap_or("Unnamed workout");
    let date = workout.start_date.date();
    let time = workout.start_date.time();

    let _ = writeln!(out, "## {name} ({date})");
    let _ = writeln!(out, "Started: {date} {time} UTC");

    if let Some(end) = workout.end_date {
        let _ = writeln!(out, "Ended: {} {} UTC", end.date(), end.time());
    }

    if workout.entries.is_empty() {
        let _ = writeln!(out, "\nNo exercises recorded.");
        return;
    }

    out.push('\n');

    for entry in &workout.entries {
        let (ex_name, muscle) = exercises
            .get(&entry.exercise_id)
            .map(|e| (e.name.as_str(), Some(e.muscle_group)))
            .unwrap_or(("Unknown exercise", None));

        let muscle_label = muscle.map(|m| format!(" ({m})")).unwrap_or_default();
        let _ = writeln!(out, "### {ex_name}{muscle_label}");

        if let Some(ref notes) = entry.notes {
            let _ = writeln!(out, "Notes: {notes}");
        }

        for (j, set) in entry.sets.iter().enumerate() {
            let load = match &set.kind {
                LoadType::Weighted(w) => format!("{} {}", w.value, w.units),
                LoadType::BodyWeight => "bodyweight".into(),
            };
            let _ = writeln!(out, "- Set {}: {} x {} reps", j + 1, load, set.reps);
        }

        out.push('\n');
    }
}

pub(super) fn format_exercises(
    exercises: &[ExerciseMetadata],
    filter: Option<MuscleGroup>,
) -> String {
    if exercises.is_empty() {
        return match filter {
            Some(group) => format!("No exercises found matching muscle group: {group}"),
            None => "No exercises found.".into(),
        };
    }

    let mut out = String::new();

    for exercise in exercises {
        let _ = writeln!(out, "### {}", exercise.name);
        let _ = writeln!(out, "Primary muscle: {}", exercise.muscle_group);

        if let Some(secondary) = exercise.secondary_muscle_groups.as_ref() {
            if !secondary.is_empty() {
                let secondary = secondary
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                let _ = writeln!(out, "Secondary muscles: {secondary}");
            }
        }

        out.push('\n');
    }

    out.trim_end().to_owned()
}
