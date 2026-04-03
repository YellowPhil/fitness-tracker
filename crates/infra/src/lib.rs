pub mod bot;
pub mod mcp;
pub mod web;

mod repos;

pub use repos::excercies::{SqliteExcerciseDb, SqliteExcerciseRepo, SqliteExcerciseRepoError};
pub use repos::health::{SqliteHealthDb, SqliteHealthRepo, SqliteHealthRepoError};
pub use repos::workouts::{SqliteWorkoutDb, SqliteWorkoutRepo, SqliteWorkoutRepoError};
pub use web::{Databases, http_router, router};
