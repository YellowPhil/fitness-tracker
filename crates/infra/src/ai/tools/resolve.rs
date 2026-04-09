use std::collections::HashMap;

use anyhow::Context;

use crate::ai::dto;
use domain::types::{
    Exercise, ExerciseKind, LoadType, PerformedSet, Weight, WeightUnits, WorkoutExercise,
};

pub(super) fn resolve_workout(
    response: dto::AiWorkoutResponse,
    exercises_by_name: &HashMap<String, Exercise>,
) -> anyhow::Result<Vec<WorkoutExercise>> {
    let mut out = Vec::with_capacity(response.exercises.len());

    for entry in response.exercises {
        let key = entry.exercise_name.to_lowercase();
        let exercise = exercises_by_name
            .get(&key)
            .with_context(|| format!("Unknown exercise name: {}", entry.exercise_name))?;

        let mut sets = Vec::with_capacity(entry.sets.len());
        for s in entry.sets {
            let kind = match exercise.kind {
                ExerciseKind::BodyWeight => LoadType::BodyWeight,
                ExerciseKind::Weighted => match s.weight_kg {
                    Some(w) if w > 0.0 => {
                        LoadType::Weighted(Weight::new(w, WeightUnits::Kilograms))
                    }
                    _ => LoadType::BodyWeight,
                },
            };

            sets.push(PerformedSet { kind, reps: s.reps });
        }

        out.push(WorkoutExercise {
            exercise_id: exercise.id,
            sets,
            notes: entry.notes,
        });
    }

    Ok(out)
}
