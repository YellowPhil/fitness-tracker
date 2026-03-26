mod error;
mod health;

use domain::{
    excercise::{
        Excercise, ExcerciseId, ExcerciseKind, MuscleGroup, PerformedSet, Workout, WorkoutExercise,
        WorkoutId,
    },
    traits::*,
};

pub use crate::error::AppError;

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

    pub fn get_all_excercises(
        &self,
    ) -> Result<Vec<Excercise>, AppError<E::RepoError, W::RepoError>> {
        self.excercise_repo
            .get_all()
            .map_err(AppError::ExcerciseRepo)
    }

    pub fn get_all_workouts(&self) -> Result<Vec<Workout>, AppError<E::RepoError, W::RepoError>> {
        self.workout_repo.get_all().map_err(AppError::WorkoutRepo)
    }

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

    pub fn create_new_workout(
        &self,
        name: Option<String>,
    ) -> Result<Workout, AppError<E::RepoError, W::RepoError>> {
        let workout = Workout::new(name);
        self.workout_repo
            .save(&workout)
            .map_err(AppError::WorkoutRepo)?;
        Ok(workout)
    }

    pub fn save_workout(
        &self,
        workout: &Workout,
    ) -> Result<(), AppError<E::RepoError, W::RepoError>> {
        self.workout_repo
            .save(workout)
            .map_err(AppError::WorkoutRepo)
    }

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

    pub fn get_workout_by_id(
        &self,
        id: &WorkoutId,
    ) -> Result<Option<Workout>, AppError<E::RepoError, W::RepoError>> {
        self.workout_repo
            .get_by_id(id)
            .map_err(AppError::WorkoutRepo)
    }
}
