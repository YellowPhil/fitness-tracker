use time::Date;

use crate::{
    excercise::{Excercise, ExcerciseId, PerformedSet, Workout, WorkoutExercise, WorkoutId},
    health::HealthParams,
};

pub trait ExcerciseRepo {
    type RepoError: std::error::Error + Send + Sync;

    fn get_by_id(&self, id: &ExcerciseId) -> Result<Option<Excercise>, Self::RepoError>;
    fn save(&self, exercise: &Excercise) -> Result<(), Self::RepoError>;

    fn get_all(&self) -> Result<Vec<Excercise>, Self::RepoError>;

    fn delete(&self, id: &ExcerciseId) -> Result<(), Self::RepoError>;
}

pub trait WorkoutRepo {
    type RepoError: std::error::Error + Send + Sync;

    fn get_by_id(&self, id: &WorkoutId) -> Result<Option<Workout>, Self::RepoError>;
    fn get_all(&self) -> Result<Vec<Workout>, Self::RepoError>;
    fn save(&self, workout: &Workout) -> Result<(), Self::RepoError>;

    fn add_exercise(
        &self,
        workout_id: &WorkoutId,
        exercise: &WorkoutExercise,
    ) -> Result<(), Self::RepoError>;

    fn add_set(
        &self,
        workout_id: &WorkoutId,
        exercise_id: &ExcerciseId,
        set: &PerformedSet,
    ) -> Result<(), Self::RepoError>;

    fn get_by_date(&self, date: Date) -> Result<Vec<Workout>, Self::RepoError>;

    fn delete(&self, id: &WorkoutId) -> Result<(), Self::RepoError>;

    fn update_name(
        &self,
        id: &WorkoutId,
        name: Option<&str>,
    ) -> Result<(), Self::RepoError>;

    fn remove_exercise(
        &self,
        workout_id: &WorkoutId,
        exercise_id: &ExcerciseId,
    ) -> Result<(), Self::RepoError>;

    fn remove_exercise_from_all(&self, exercise_id: &ExcerciseId)
        -> Result<(), Self::RepoError>;

    fn remove_set(
        &self,
        workout_id: &WorkoutId,
        exercise_id: &ExcerciseId,
        set_index: usize,
    ) -> Result<(), Self::RepoError>;

    fn get_dates_in_range(
        &self,
        from: Date,
        to: Date,
    ) -> Result<Vec<Date>, Self::RepoError>;
}

pub trait HealthRepo {
    type RepoError: std::error::Error + Send + Sync;

    fn get_health(&self) -> Result<HealthParams, Self::RepoError>;
    fn save(&self, params: &HealthParams) -> Result<(), Self::RepoError>;
}
