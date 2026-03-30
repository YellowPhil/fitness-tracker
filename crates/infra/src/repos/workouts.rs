use std::path::Path;

use domain::{
    excercise::{ExcerciseId, LoadType, PerformedSet, Workout, WorkoutExercise, WorkoutId},
    traits::WorkoutRepo,
    types::{UserId, Weight, WeightUnits},
};
use rusqlite::{Connection, OptionalExtension, params};
use time::OffsetDateTime;

#[derive(Debug, thiserror::Error)]
pub enum SqliteWorkoutRepoError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("invalid workout id: {0}")]
    InvalidWorkoutId(uuid::Error),
    #[error("invalid excercise id: {0}")]
    InvalidExcerciseId(uuid::Error),
    #[error("invalid timestamp: {0}")]
    InvalidTimestamp(i64),
    #[error("invalid reps: {0}")]
    InvalidReps(i64),
    #[error("invalid load type: {0}")]
    InvalidLoadType(i64),
    #[error("invalid weight units: {0}")]
    InvalidWeightUnits(i64),
    #[error("missing weight for weighted set")]
    MissingWeightForWeightedSet,
    #[error("missing weight units for weighted set")]
    MissingWeightUnitsForWeightedSet,
}

pub struct SqliteWorkoutDb {
    connection: Connection,
}

pub struct SqliteWorkoutRepo<'db> {
    connection: &'db Connection,
    user_id: UserId,
}

impl SqliteWorkoutDb {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, SqliteWorkoutRepoError> {
        let connection = Connection::open(path)?;
        Self::init_schema(&connection)?;
        Ok(Self { connection })
    }

    pub fn in_memory() -> Result<Self, SqliteWorkoutRepoError> {
        let connection = Connection::open_in_memory()?;
        Self::init_schema(&connection)?;
        Ok(Self { connection })
    }

    pub fn for_user(&self, user_id: UserId) -> SqliteWorkoutRepo<'_> {
        SqliteWorkoutRepo {
            connection: &self.connection,
            user_id,
        }
    }

    fn init_schema(connection: &Connection) -> Result<(), SqliteWorkoutRepoError> {
        connection.execute_batch(
            "
            PRAGMA foreign_keys = ON;

            CREATE TABLE IF NOT EXISTS workouts (
                id TEXT NOT NULL,
                user_id INTEGER NOT NULL,
                name TEXT,
                start_date INTEGER NOT NULL,
                end_date INTEGER,
                PRIMARY KEY (id, user_id)
            );

            CREATE TABLE IF NOT EXISTS workout_exercises (
                workout_id TEXT NOT NULL,
                user_id INTEGER NOT NULL,
                excercise_id TEXT NOT NULL,
                entry_order INTEGER NOT NULL,
                notes TEXT,
                PRIMARY KEY (workout_id, user_id, excercise_id),
                FOREIGN KEY (workout_id, user_id) REFERENCES workouts(id, user_id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS performed_sets (
                workout_id TEXT NOT NULL,
                user_id INTEGER NOT NULL,
                excercise_id TEXT NOT NULL,
                set_order INTEGER NOT NULL,
                reps INTEGER NOT NULL,
                load_type INTEGER NOT NULL,
                weight REAL,
                weight_units INTEGER,
                PRIMARY KEY (workout_id, user_id, excercise_id, set_order),
                FOREIGN KEY (workout_id, user_id, excercise_id)
                    REFERENCES workout_exercises(workout_id, user_id, excercise_id)
                    ON DELETE CASCADE
            );
            ",
        )?;

        Ok(())
    }
}
