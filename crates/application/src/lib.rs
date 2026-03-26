mod error;

use domain::{
    excercise::{
        Excercise, ExcerciseId, ExcerciseKind, MuscleGroup, PerformedSet, Workout, WorkoutExercise,
        WorkoutId,
    },
    traits::*,
};

use crate::error::AppError;

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
            .map_err(|e| AppError::ExcerciseRepo(e))?;
        Ok(())
    }

    pub fn create_new_workout(
        &self,
        name: Option<String>,
    ) -> Result<WorkoutId, AppError<E::RepoError, W::RepoError>> {
        let workout = Workout::new(name);
        self.workout_repo
            .save(&workout)
            .map_err(|e| AppError::WorkoutRepo(e))?;
        Ok(workout.id)
    }

    pub fn add_excercise_to_workout(
        &self,
        workout_id: WorkoutId,
        excercise_id: ExcerciseId,
    ) -> Result<(), AppError<E::RepoError, W::RepoError>> {
        self.workout_repo
            .add_exercise(&workout_id, &WorkoutExercise::new(excercise_id))
            .map_err(|e| AppError::WorkoutRepo(e))?;
        Ok(())
    }

    pub fn add_set_for_excercise(
        &self,
        workout_id: WorkoutId,
        excercise_id: ExcerciseId,
        set: PerformedSet,
    ) -> Result<(), AppError<E::RepoError, W::RepoError>> {
        self.workout_repo
            .add_set(&workout_id, &excercise_id, &set)
            .map_err(|e| AppError::WorkoutRepo(e))?;
        Ok(())
    }
}
