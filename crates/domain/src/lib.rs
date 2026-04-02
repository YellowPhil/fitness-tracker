mod ai;
pub mod excercise;
pub mod health;
pub mod traits;
pub mod types;

#[cfg(feature = "rusqlite")]
mod sql;

pub use ai::InferenceProvider;
