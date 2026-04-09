use std::{collections::HashMap, sync::Arc};

use anyhow::Context;

use domain::{
    traits::ExcerciseRepo,
    types::{Exercise, ExerciseId, MuscleGroup, UserId},
};

use crate::Databases;

use tracing::instrument;

#[instrument(skip(databases, muscle_groups), fields(user_id = user_id.as_i64()), err)]
pub(super) async fn load_exercises_for_muscle_groups(
    databases: &Arc<Databases>,
    user_id: UserId,
    muscle_groups: &[MuscleGroup],
) -> anyhow::Result<Vec<Exercise>> {
    let mut by_id: HashMap<ExerciseId, Exercise> = HashMap::new();
    let repo = databases.exercise_db.for_user(user_id);
    for &mg in muscle_groups {
        let list = repo
            .get_by_muscle_group(mg)
            .await
            .with_context(|| format!("Failed to load exercises for muscle group {mg}"))?;
        for e in list {
            by_id.insert(e.id, e);
        }
    }
    Ok(by_id.into_values().collect())
}

pub(super) fn sorted_exercise_names(exercises: &[Exercise]) -> Vec<String> {
    let mut names: Vec<String> = exercises.iter().map(|e| e.name.clone()).collect();
    names.sort();
    names
}

pub(super) fn exercises_by_lowercase_name(exercises: &[Exercise]) -> HashMap<String, Exercise> {
    let mut map = HashMap::new();
    for e in exercises {
        let key = e.name.to_lowercase();
        map.entry(key).or_insert_with(|| e.clone());
    }
    map
}
