pub mod bot;
pub mod ai;
pub mod web;

mod repos;

pub use repos::excercies::{PostgresExcerciseDb, PostgresExcerciseRepo, PostgresExcerciseRepoError};
pub use repos::health::{PostgresHealthDb, PostgresHealthRepo, PostgresHealthRepoError};
pub use repos::workouts::{PostgresWorkoutDb, PostgresWorkoutRepo, PostgresWorkoutRepoError};
pub use web::{Databases, http_router, router};
