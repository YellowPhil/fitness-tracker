use std::collections::{HashMap, HashSet};

use domain::{
    traits::*,
    types::{
        Exercise, ExerciseId, ExerciseKind, ExerciseMetadata, ExerciseSource, MuscleGroup,
        PerformedSet, QueryType, Workout, WorkoutExercise, WorkoutId, WorkoutQuery, catalog,
    },
};
use time::{Date, OffsetDateTime};
use tracing::{debug, instrument};

pub use super::error::AppError;

pub struct GymApp<E: ExcerciseRepo, W: WorkoutRepo> {
    excercise_repo: E,
    workout_repo: W,
}

pub struct WorkoutQueryResult {
    pub workouts: Vec<Workout>,
    pub excercises: HashMap<ExerciseId, ExerciseMetadata>,
}

impl<E: ExcerciseRepo, W: WorkoutRepo> GymApp<E, W> {
    pub fn new(excercise_repo: E, workout_repo: W) -> Self {
        Self {
            excercise_repo,
            workout_repo,
        }
    }

    #[instrument(skip(self), err)]
    pub async fn get_all_excercises(
        &self,
    ) -> Result<Vec<Exercise>, AppError<E::RepoError, W::RepoError>> {
        self.excercise_repo
            .get_all()
            .await
            .map_err(AppError::ExcerciseRepo)
    }

    /// Inserts the full built-in exercise catalog when the user has none yet.
    #[instrument(skip(self), err)]
    pub async fn seed_built_in_excercises(
        &self,
    ) -> Result<(), AppError<E::RepoError, W::RepoError>> {
        let existing = self
            .excercise_repo
            .get_all()
            .await
            .map_err(AppError::ExcerciseRepo)?;

        let has_built_ins = existing.iter().any(|e| e.source == ExerciseSource::BuiltIn);

        if !has_built_ins {
            debug!("no built-in exercises found, seeding catalog");
            for exercise in catalog::built_in_exercises() {
                self.excercise_repo
                    .save(&exercise)
                    .await
                    .map_err(AppError::ExcerciseRepo)?;
            }
        }
        Ok(())
    }

    #[instrument(skip(self), err)]
    pub async fn get_all_workouts(
        &self,
    ) -> Result<Vec<Workout>, AppError<E::RepoError, W::RepoError>> {
        self.workout_repo
            .get_all()
            .await
            .map_err(AppError::WorkoutRepo)
    }

    #[instrument(
        skip(self, name, secondary_muscle_groups),
        fields(muscle_group = ?muscle_group, kind = ?kind),
        err
    )]
    pub async fn add_new_excercise(
        &self,
        name: String,
        muscle_group: MuscleGroup,
        secondary_muscle_groups: Option<Vec<MuscleGroup>>,
        kind: ExerciseKind,
    ) -> Result<(), AppError<E::RepoError, W::RepoError>> {
        let excercise = Exercise::new(name, muscle_group, secondary_muscle_groups, kind);
        self.excercise_repo
            .save(&excercise)
            .await
            .map_err(AppError::ExcerciseRepo)?;
        Ok(())
    }

    #[instrument(skip(self), fields(name = ?name, date = ?date), err)]
    pub async fn create_new_workout(
        &self,
        name: Option<String>,
        date: Option<OffsetDateTime>,
    ) -> Result<Workout, AppError<E::RepoError, W::RepoError>> {
        let mut workout = Workout::new(name);
        if let Some(date) = date {
            workout.start_date = date;
        }
        self.workout_repo
            .save(&workout)
            .await
            .map_err(AppError::WorkoutRepo)?;
        Ok(workout)
    }

    #[instrument(skip(self, workout), err)]
    pub async fn save_workout(
        &self,
        workout: &Workout,
    ) -> Result<(), AppError<E::RepoError, W::RepoError>> {
        self.workout_repo
            .save(workout)
            .await
            .map_err(AppError::WorkoutRepo)
    }

    #[instrument(skip(self), fields(workout_id = ?workout_id, excercise_id = ?excercise_id), err)]
    pub async fn add_excercise_to_workout(
        &self,
        workout_id: &WorkoutId,
        excercise_id: ExerciseId,
    ) -> Result<(), AppError<E::RepoError, W::RepoError>> {
        self.workout_repo
            .add_exercise(workout_id, &WorkoutExercise::new(excercise_id))
            .await
            .map_err(AppError::WorkoutRepo)?;
        Ok(())
    }

    #[instrument(
        skip(self, set),
        fields(workout_id = ?workout_id, excercise_id = ?excercise_id),
        err
    )]
    pub async fn add_set_for_excercise(
        &self,
        workout_id: &WorkoutId,
        excercise_id: &ExerciseId,
        set: PerformedSet,
    ) -> Result<(), AppError<E::RepoError, W::RepoError>> {
        self.workout_repo
            .add_set(workout_id, excercise_id, &set)
            .await
            .map_err(AppError::WorkoutRepo)?;
        Ok(())
    }

    #[instrument(skip(self), fields(workout_id = ?id), err)]
    pub async fn get_workout_by_id(
        &self,
        id: &WorkoutId,
    ) -> Result<Option<Workout>, AppError<E::RepoError, W::RepoError>> {
        self.workout_repo
            .get_by_id(id)
            .await
            .map_err(AppError::WorkoutRepo)
    }

    #[instrument(skip(self), fields(date = ?date), err)]
    pub async fn get_workout_by_date(
        &self,
        date: Date,
    ) -> Result<Vec<Workout>, AppError<E::RepoError, W::RepoError>> {
        self.workout_repo
            .get_by_date(date)
            .await
            .map_err(AppError::WorkoutRepo)
    }

    #[instrument(skip(self), fields(excercise_id = ?id), err)]
    pub async fn delete_excercise(
        &self,
        id: &ExerciseId,
    ) -> Result<(), AppError<E::RepoError, W::RepoError>> {
        self.workout_repo
            .remove_exercise_from_all(id)
            .await
            .map_err(AppError::WorkoutRepo)?;
        self.excercise_repo
            .delete(id)
            .await
            .map_err(AppError::ExcerciseRepo)?;
        Ok(())
    }

    #[instrument(skip(self), fields(workout_id = ?id), err)]
    pub async fn delete_workout(
        &self,
        id: &WorkoutId,
    ) -> Result<(), AppError<E::RepoError, W::RepoError>> {
        self.workout_repo
            .delete(id)
            .await
            .map_err(AppError::WorkoutRepo)?;
        Ok(())
    }

    #[instrument(skip(self), fields(workout_id = ?id, name = ?name), err)]
    pub async fn update_workout_name(
        &self,
        id: &WorkoutId,
        name: Option<&str>,
    ) -> Result<(), AppError<E::RepoError, W::RepoError>> {
        self.workout_repo
            .update_name(id, name)
            .await
            .map_err(AppError::WorkoutRepo)?;
        Ok(())
    }

    #[instrument(
        skip(self),
        fields(workout_id = ?workout_id, excercise_id = ?excercise_id),
        err
    )]
    pub async fn remove_excercise_from_workout(
        &self,
        workout_id: &WorkoutId,
        excercise_id: &ExerciseId,
    ) -> Result<(), AppError<E::RepoError, W::RepoError>> {
        self.workout_repo
            .remove_exercise(workout_id, excercise_id)
            .await
            .map_err(AppError::WorkoutRepo)?;
        Ok(())
    }

    #[instrument(
        skip(self, set),
        fields(
            workout_id = ?workout_id,
            excercise_id = ?excercise_id,
            set_index = set_index
        ),
        err
    )]
    pub async fn update_set_in_workout(
        &self,
        workout_id: &WorkoutId,
        excercise_id: &ExerciseId,
        set_index: usize,
        set: PerformedSet,
    ) -> Result<(), AppError<E::RepoError, W::RepoError>> {
        self.workout_repo
            .update_set(workout_id, excercise_id, set_index, &set)
            .await
            .map_err(AppError::WorkoutRepo)?;
        Ok(())
    }

    #[instrument(
        skip(self),
        fields(
            workout_id = ?workout_id,
            excercise_id = ?excercise_id,
            set_index = set_index
        ),
        err
    )]
    pub async fn remove_set_from_workout(
        &self,
        workout_id: &WorkoutId,
        excercise_id: &ExerciseId,
        set_index: usize,
    ) -> Result<(), AppError<E::RepoError, W::RepoError>> {
        self.workout_repo
            .remove_set(workout_id, excercise_id, set_index)
            .await
            .map_err(AppError::WorkoutRepo)?;
        Ok(())
    }

    #[instrument(skip(self), fields(from = ?from, to = ?to), err)]
    pub async fn get_workout_dates_in_range(
        &self,
        from: Date,
        to: Date,
    ) -> Result<Vec<Date>, AppError<E::RepoError, W::RepoError>> {
        self.workout_repo
            .get_dates_in_range(from, to)
            .await
            .map_err(AppError::WorkoutRepo)
    }

    #[instrument(skip(self), fields(query = ?query), err)]
    pub async fn query_workout_resource(
        &self,
        query: WorkoutQuery,
    ) -> Result<WorkoutQueryResult, AppError<E::RepoError, W::RepoError>> {
        let mut workouts = match query.date {
            QueryType::OnDate(date) => self
                .workout_repo
                .get_by_date(date)
                .await
                .map_err(AppError::WorkoutRepo)?,
            QueryType::LastN(n) => self
                .workout_repo
                .get_last_n(n)
                .await
                .map_err(AppError::WorkoutRepo)?,
            QueryType::Latest => self
                .workout_repo
                .get_latest()
                .await
                .map_err(AppError::WorkoutRepo)?
                .into_iter()
                .collect(),
        };

        let exercise_ids = collect_excercise_ids(&workouts);
        let mut excercises: HashMap<_, _> = self
            .excercise_repo
            .get_metadata_by_ids(&exercise_ids)
            .await
            .map_err(AppError::ExcerciseRepo)?
            .into_iter()
            .map(|exercise| (exercise.id, exercise))
            .collect();

        if let Some(muscle_group) = query.muscle_group {
            for workout in &mut workouts {
                workout.entries.retain(|entry| {
                    excercises
                        .get(&entry.exercise_id)
                        .is_some_and(|exercise| exercise.matches_muscle_group(muscle_group))
                });
            }
            workouts.retain(|w| !w.entries.is_empty());
        }

        let referenced_excercise_ids: HashSet<_> =
            collect_excercise_ids(&workouts).into_iter().collect();
        excercises.retain(|id, _| referenced_excercise_ids.contains(id));

        Ok(WorkoutQueryResult {
            workouts,
            excercises,
        })
    }

    #[instrument(skip(self), fields(query = ?query), err)]
    pub async fn query_workouts(
        &self,
        query: WorkoutQuery,
    ) -> Result<Vec<Workout>, AppError<E::RepoError, W::RepoError>> {
        self.query_workout_resource(query)
            .await
            .map(|result| result.workouts)
    }

    #[instrument(skip(self), fields(exercise_id = ?id), err)]
    pub async fn get_excercise_by_id(
        &self,
        id: &ExerciseId,
    ) -> Result<Option<Exercise>, AppError<E::RepoError, W::RepoError>> {
        self.excercise_repo
            .get_by_id(id)
            .await
            .map_err(AppError::ExcerciseRepo)
    }
}

fn collect_excercise_ids(workouts: &[Workout]) -> Vec<ExerciseId> {
    let mut seen = HashSet::new();
    let mut ids = Vec::new();

    for workout in workouts {
        for entry in &workout.entries {
            if seen.insert(entry.exercise_id) {
                ids.push(entry.exercise_id);
            }
        }
    }

    ids
}
