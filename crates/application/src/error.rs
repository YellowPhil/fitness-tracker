#[derive(thiserror::Error, Debug)]
pub enum AppError<ExerciseError, WorkoutError> {
    #[error("excercise repository error: {0}")]
    ExcerciseRepo(ExerciseError),
    #[error("workout repository error: {0}")]
    WorkoutRepo(WorkoutError),
}
