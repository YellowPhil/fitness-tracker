use time::Date;

use crate::{
    excercise::{
        Exercise, ExerciseId, ExerciseMetadata, MuscleGroup, PerformedSet, Workout,
        WorkoutExercise, WorkoutId,
    },
    health::HealthParams,
};

#[async_trait::async_trait]
pub trait ExcerciseRepo {
    type RepoError: std::error::Error + Send + Sync;

    async fn get_by_id(&self, id: &ExerciseId) -> Result<Option<Exercise>, Self::RepoError>;
    async fn save(&self, exercise: &Exercise) -> Result<(), Self::RepoError>;

    async fn get_all(&self) -> Result<Vec<Exercise>, Self::RepoError>;

    async fn get_by_muscle_group(
        &self,
        muscle_group: MuscleGroup,
    ) -> Result<Vec<Exercise>, Self::RepoError>;

    async fn get_metadata_by_ids(
        &self,
        ids: &[ExerciseId],
    ) -> Result<Vec<ExerciseMetadata>, Self::RepoError>;

    async fn delete(&self, id: &ExerciseId) -> Result<(), Self::RepoError>;
}

#[async_trait::async_trait]
pub trait WorkoutRepo {
    type RepoError: std::error::Error + Send + Sync;

    async fn get_by_id(&self, id: &WorkoutId) -> Result<Option<Workout>, Self::RepoError>;
    async fn get_all(&self) -> Result<Vec<Workout>, Self::RepoError>;
    async fn save(&self, workout: &Workout) -> Result<(), Self::RepoError>;

    async fn add_exercise(
        &self,
        workout_id: &WorkoutId,
        exercise: &WorkoutExercise,
    ) -> Result<(), Self::RepoError>;

    async fn add_set(
        &self,
        workout_id: &WorkoutId,
        exercise_id: &ExerciseId,
        set: &PerformedSet,
    ) -> Result<(), Self::RepoError>;

    async fn get_by_date(&self, date: Date) -> Result<Vec<Workout>, Self::RepoError>;

    async fn get_latest(&self) -> Result<Option<Workout>, Self::RepoError>;

    async fn get_last_n(&self, n: usize) -> Result<Vec<Workout>, Self::RepoError>;

    async fn delete(&self, id: &WorkoutId) -> Result<(), Self::RepoError>;

    async fn update_name(&self, id: &WorkoutId, name: Option<&str>) -> Result<(), Self::RepoError>;

    async fn remove_exercise(
        &self,
        workout_id: &WorkoutId,
        exercise_id: &ExerciseId,
    ) -> Result<(), Self::RepoError>;

    async fn remove_exercise_from_all(
        &self,
        exercise_id: &ExerciseId,
    ) -> Result<(), Self::RepoError>;

    async fn remove_set(
        &self,
        workout_id: &WorkoutId,
        exercise_id: &ExerciseId,
        set_index: usize,
    ) -> Result<(), Self::RepoError>;

    async fn get_dates_in_range(&self, from: Date, to: Date) -> Result<Vec<Date>, Self::RepoError>;
}

#[async_trait::async_trait]
pub trait HealthRepo {
    type RepoError: std::error::Error + Send + Sync;

    async fn get_health(&self) -> Result<HealthParams, Self::RepoError>;
    async fn save(&self, params: &HealthParams) -> Result<(), Self::RepoError>;
}
