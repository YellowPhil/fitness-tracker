use std::path::Path;

use domain::{
    excercise::{Excercise, ExcerciseId, ExcerciseKind, ExcerciseSource, MuscleGroup},
    traits::ExcerciseRepo,
    types::UserId,
};
use rusqlite::{Connection, OptionalExtension, params};

#[derive(Debug, thiserror::Error)]
pub enum SqliteExcerciseRepoError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("invalid excercise id: {0}")]
    InvalidExcerciseId(#[from] uuid::Error),
    #[error("invalid excercise kind: {0}")]
    InvalidExcerciseKind(i64),
    #[error("invalid excercise source: {0}")]
    InvalidExcerciseSource(i64),
    #[error("invalid muscle group: {0}")]
    InvalidMuscleGroup(i64),
    #[error("invalid secondary muscle group: {0}")]
    InvalidSecondaryMuscleGroup(i64),
}
pub struct SqliteExcerciseDb {
    connection: Connection,
}

pub struct SqliteExcerciseRepo<'db> {
    connection: &'db Connection,
    user_id: UserId,
}

impl SqliteExcerciseDb {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, SqliteExcerciseRepoError> {
        let connection = Connection::open(path)?;
        Self::init_schema(&connection)?;
        Ok(Self { connection })
    }

    pub fn in_memory() -> Result<Self, SqliteExcerciseRepoError> {
        let connection = Connection::open_in_memory()?;
        Self::init_schema(&connection)?;
        Ok(Self { connection })
    }

    pub fn for_user(&self, user_id: UserId) -> SqliteExcerciseRepo<'_> {
        SqliteExcerciseRepo {
            connection: &self.connection,
            user_id,
        }
    }

    fn init_schema(connection: &Connection) -> Result<(), SqliteExcerciseRepoError> {
        connection.execute_batch(
            "CREATE TABLE IF NOT EXISTS excercises (
                id TEXT NOT NULL,
                user_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                kind INTEGER NOT NULL,
                muscle_group INTEGER NOT NULL,
                secondary_muscle_groups TEXT,
                source INTEGER NOT NULL,
                PRIMARY KEY (id, user_id)
            );",
        )?;

        Ok(())
    }
}
