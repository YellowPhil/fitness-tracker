use domain::{
    excercise::{
        Excercise, ExcerciseId, ExcerciseKind, ExcerciseSource, MuscleGroup, PerformedSet, Workout,
        WorkoutExercise, WorkoutId, catalog,
    },
    traits::*,
};
use time::{Date, OffsetDateTime};
use tracing::instrument;

pub use super::error::AppError;

pub struct GymApp<E: ExcerciseRepo, W: WorkoutRepo> {
    excercise_repo: E,
    workout_repo: W,
}

impl<E: ExcerciseRepo, W: WorkoutRepo> GymApp<E, W> {
    pub fn new(excercise_repo: E, workout_repo: W) -> Self {
        Self {
            excercise_repo,
            workout_repo,
        }
    }

    #[instrument(skip(self), err)]
    pub fn get_all_excercises(
        &self,
    ) -> Result<Vec<Excercise>, AppError<E::RepoError, W::RepoError>> {
        self.excercise_repo
            .get_all()
            .map_err(AppError::ExcerciseRepo)
    }

    /// Inserts the full built-in exercise catalog when the user has none yet.
    #[instrument(skip(self), err)]
    pub fn seed_built_in_excercises(
        &self,
    ) -> Result<(), AppError<E::RepoError, W::RepoError>> {
        let existing = self
            .excercise_repo
            .get_all()
            .map_err(AppError::ExcerciseRepo)?;

        let has_built_ins = existing
            .iter()
            .any(|e| e.source == ExcerciseSource::BuiltIn);

        if !has_built_ins {
            for exercise in catalog::built_in_exercises() {
                self.excercise_repo
                    .save(&exercise)
                    .map_err(AppError::ExcerciseRepo)?;
            }
        }
        Ok(())
    }

    #[instrument(skip(self), err)]
    pub fn get_all_workouts(&self) -> Result<Vec<Workout>, AppError<E::RepoError, W::RepoError>> {
        self.workout_repo.get_all().map_err(AppError::WorkoutRepo)
    }

    #[instrument(
        skip(self, name, secondary_muscle_groups),
        fields(muscle_group = ?muscle_group, kind = ?kind),
        err
    )]
    pub fn add_new_excercise(
        &self,
        name: String,
        muscle_group: MuscleGroup,
        secondary_muscle_groups: Option<Vec<MuscleGroup>>,
        kind: ExcerciseKind,
    ) -> Result<(), AppError<E::RepoError, W::RepoError>> {
        let excercise = Excercise::new(name, muscle_group, secondary_muscle_groups, kind);
        self.excercise_repo
            .save(&excercise)
            .map_err(AppError::ExcerciseRepo)?;
        Ok(())
    }

    #[instrument(skip(self), fields(name = ?name, date = ?date), err)]
    pub fn create_new_workout(
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
            .map_err(AppError::WorkoutRepo)?;
        Ok(workout)
    }

    #[instrument(skip(self, workout), err)]
    pub fn save_workout(
        &self,
        workout: &Workout,
    ) -> Result<(), AppError<E::RepoError, W::RepoError>> {
        self.workout_repo
            .save(workout)
            .map_err(AppError::WorkoutRepo)
    }

    #[instrument(skip(self), fields(workout_id = ?workout_id, excercise_id = ?excercise_id), err)]
    pub fn add_excercise_to_workout(
        &self,
        workout_id: &WorkoutId,
        excercise_id: ExcerciseId,
    ) -> Result<(), AppError<E::RepoError, W::RepoError>> {
        self.workout_repo
            .add_exercise(workout_id, &WorkoutExercise::new(excercise_id))
            .map_err(AppError::WorkoutRepo)?;
        Ok(())
    }

    #[instrument(
        skip(self, set),
        fields(workout_id = ?workout_id, excercise_id = ?excercise_id),
        err
    )]
    pub fn add_set_for_excercise(
        &self,
        workout_id: &WorkoutId,
        excercise_id: &ExcerciseId,
        set: PerformedSet,
    ) -> Result<(), AppError<E::RepoError, W::RepoError>> {
        self.workout_repo
            .add_set(workout_id, excercise_id, &set)
            .map_err(AppError::WorkoutRepo)?;
        Ok(())
    }

    #[instrument(skip(self), fields(workout_id = ?id), err)]
    pub fn get_workout_by_id(
        &self,
        id: &WorkoutId,
    ) -> Result<Option<Workout>, AppError<E::RepoError, W::RepoError>> {
        self.workout_repo
            .get_by_id(id)
            .map_err(AppError::WorkoutRepo)
    }

    #[instrument(skip(self), fields(date = ?date), err)]
    pub fn get_workout_by_date(
        &self,
        date: Date,
    ) -> Result<Vec<Workout>, AppError<E::RepoError, W::RepoError>> {
        self.workout_repo
            .get_by_date(date)
            .map_err(AppError::WorkoutRepo)
    }

    #[instrument(skip(self), fields(excercise_id = ?id), err)]
    pub fn delete_excercise(
        &self,
        id: &ExcerciseId,
    ) -> Result<(), AppError<E::RepoError, W::RepoError>> {
        self.workout_repo
            .remove_exercise_from_all(id)
            .map_err(AppError::WorkoutRepo)?;
        self.excercise_repo
            .delete(id)
            .map_err(AppError::ExcerciseRepo)?;
        Ok(())
    }

    #[instrument(skip(self), fields(workout_id = ?id), err)]
    pub fn delete_workout(
        &self,
        id: &WorkoutId,
    ) -> Result<(), AppError<E::RepoError, W::RepoError>> {
        self.workout_repo
            .delete(id)
            .map_err(AppError::WorkoutRepo)
    }

    #[instrument(skip(self), fields(workout_id = ?id, name = ?name), err)]
    pub fn update_workout_name(
        &self,
        id: &WorkoutId,
        name: Option<&str>,
    ) -> Result<(), AppError<E::RepoError, W::RepoError>> {
        self.workout_repo
            .update_name(id, name)
            .map_err(AppError::WorkoutRepo)
    }

    #[instrument(
        skip(self),
        fields(workout_id = ?workout_id, excercise_id = ?excercise_id),
        err
    )]
    pub fn remove_excercise_from_workout(
        &self,
        workout_id: &WorkoutId,
        excercise_id: &ExcerciseId,
    ) -> Result<(), AppError<E::RepoError, W::RepoError>> {
        self.workout_repo
            .remove_exercise(workout_id, excercise_id)
            .map_err(AppError::WorkoutRepo)
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
    pub fn remove_set_from_workout(
        &self,
        workout_id: &WorkoutId,
        excercise_id: &ExcerciseId,
        set_index: usize,
    ) -> Result<(), AppError<E::RepoError, W::RepoError>> {
        self.workout_repo
            .remove_set(workout_id, excercise_id, set_index)
            .map_err(AppError::WorkoutRepo)
    }

    #[instrument(skip(self), fields(from = ?from, to = ?to), err)]
    pub fn get_workout_dates_in_range(
        &self,
        from: Date,
        to: Date,
    ) -> Result<Vec<Date>, AppError<E::RepoError, W::RepoError>> {
        self.workout_repo
            .get_dates_in_range(from, to)
            .map_err(AppError::WorkoutRepo)
    }
}
