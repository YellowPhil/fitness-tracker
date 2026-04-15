pub mod ai;
pub mod bot;
pub mod generation;
pub mod grpc;
pub mod web;

mod repos;

pub use repos::excercies::{
    PostgresExcerciseDb, PostgresExcerciseRepo, PostgresExcerciseRepoError,
};
pub use repos::generation_jobs::{
    GenerationJob, GenerationJobListScope, GenerationJobStatus, PostgresGenerationJobDb,
    PostgresGenerationJobRepo, PostgresGenerationJobRepoError,
};
pub use repos::health::{PostgresHealthDb, PostgresHealthRepo, PostgresHealthRepoError};
pub use repos::preferences::{
    PostgresPreferencesDb, PostgresPreferencesRepo, PostgresPreferencesRepoError,
};
pub use repos::workouts::{PostgresWorkoutDb, PostgresWorkoutRepo, PostgresWorkoutRepoError};
pub use web::{Databases, http_router, router};
