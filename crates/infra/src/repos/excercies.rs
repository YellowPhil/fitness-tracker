use std::path::Path;

use domain::{
    excercise::{Exercise, ExerciseId, ExerciseMetadata, MuscleGroup},
    traits::ExcerciseRepo,
    types::UserId,
};
use rusqlite::{Connection, OptionalExtension, ToSql, params, params_from_iter};

#[derive(Debug, thiserror::Error)]
pub enum SqliteExcerciseRepoError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("invalid secondary muscle group value: {0}")]
    InvalidSecondaryMuscleGroup(String),
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

impl ExcerciseRepo for SqliteExcerciseRepo<'_> {
    type RepoError = SqliteExcerciseRepoError;

    fn get_by_id(&self, id: &ExerciseId) -> Result<Option<Exercise>, Self::RepoError> {
        let mut stmt = self.connection.prepare(
            "SELECT id, name, kind, muscle_group, secondary_muscle_groups, source
             FROM excercises
             WHERE id = ?1 AND user_id = ?2",
        )?;

        Ok(stmt
            .query_row(params![id, self.user_id], |row| {
                Ok(Exercise {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    kind: row.get(2)?,
                    muscle_group: row.get(3)?,
                    secondary_muscle_groups: decode_secondary_muscle_groups(row.get(4)?),
                    source: row.get(5)?,
                })
            })
            .optional()?)
    }

    fn save(&self, exercise: &Exercise) -> Result<(), Self::RepoError> {
        self.connection.execute(
            "INSERT INTO excercises (id, user_id, name, kind, muscle_group, secondary_muscle_groups, source)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(id, user_id) DO UPDATE SET
                 name = excluded.name,
                 kind = excluded.kind,
                 muscle_group = excluded.muscle_group,
                 secondary_muscle_groups = excluded.secondary_muscle_groups,
                 source = excluded.source",
            params![
                exercise.id,
                self.user_id,
                exercise.name,
                exercise.kind,
                exercise.muscle_group,
                encode_secondary_muscle_groups(&exercise.secondary_muscle_groups),
                exercise.source,
            ],
        )?;
        Ok(())
    }

    fn get_all(&self) -> Result<Vec<Exercise>, Self::RepoError> {
        let mut stmt = self.connection.prepare(
            "SELECT id, name, kind, muscle_group, secondary_muscle_groups, source
             FROM excercises
             WHERE user_id = ?1
             ORDER BY name ASC",
        )?;

        let rows = stmt.query_map(params![self.user_id], |row| {
            Ok(Exercise {
                id: row.get(0)?,
                name: row.get(1)?,
                kind: row.get(2)?,
                muscle_group: row.get(3)?,
                secondary_muscle_groups: decode_secondary_muscle_groups(row.get(4)?),
                source: row.get(5)?,
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    fn get_metadata_by_ids(
        &self,
        ids: &[ExerciseId],
    ) -> Result<Vec<ExerciseMetadata>, Self::RepoError> {
        if ids.is_empty() {
            return Ok(vec![]);
        }

        let placeholders = std::iter::repeat_n("?", ids.len())
            .collect::<Vec<_>>()
            .join(", ");
        let query = format!(
            "SELECT id, name, muscle_group, secondary_muscle_groups
             FROM excercises
             WHERE user_id = ? AND id IN ({placeholders})
             ORDER BY name ASC"
        );

        let mut stmt = self.connection.prepare(&query)?;
        let mut sql_params: Vec<&dyn ToSql> = Vec::with_capacity(ids.len() + 1);
        sql_params.push(&self.user_id);
        for id in ids {
            sql_params.push(id);
        }

        let rows = stmt.query_map(params_from_iter(sql_params), |row| {
            Ok(ExerciseMetadata {
                id: row.get(0)?,
                name: row.get(1)?,
                muscle_group: row.get(2)?,
                secondary_muscle_groups: decode_secondary_muscle_groups(row.get(3)?),
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    fn delete(&self, id: &ExerciseId) -> Result<(), Self::RepoError> {
        self.connection.execute(
            "DELETE FROM excercises WHERE id = ?1 AND user_id = ?2",
            params![id, self.user_id],
        )?;
        Ok(())
    }
}

/// Comma-separated MuscleGroup integers → single TEXT column.
fn encode_secondary_muscle_groups(v: &Option<Vec<MuscleGroup>>) -> Option<String> {
    v.as_ref().map(|groups| {
        groups
            .iter()
            .map(|g| {
                use rusqlite::types::ToSql;
                match g.to_sql().unwrap() {
                    rusqlite::types::ToSqlOutput::Owned(rusqlite::types::Value::Integer(n)) => {
                        n.to_string()
                    }
                    _ => unreachable!(),
                }
            })
            .collect::<Vec<_>>()
            .join(",")
    })
}

/// TEXT column → Vec<MuscleGroup> using FromSql on individual values.
fn decode_secondary_muscle_groups(raw: Option<String>) -> Option<Vec<MuscleGroup>> {
    let raw = raw?;
    if raw.is_empty() {
        return Some(vec![]);
    }
    Some(
        raw.split(',')
            .filter_map(|s| {
                let n: i64 = s.parse().ok()?;
                use rusqlite::types::{FromSql, ValueRef};
                MuscleGroup::column_result(ValueRef::Integer(n)).ok()
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use domain::{
        excercise::{Exercise, ExerciseKind, ExerciseSource, MuscleGroup},
        traits::ExcerciseRepo,
        types::UserId,
    };

    use super::SqliteExcerciseDb;

    #[test]
    fn saves_and_loads_excercise() {
        let db = SqliteExcerciseDb::in_memory().expect("db should initialize");
        let repo = db.for_user(UserId::new(1));
        let excercise = Exercise {
            id: domain::excercise::ExerciseId::new(),
            name: "Bench Press".to_string(),
            kind: ExerciseKind::Weighted,
            muscle_group: MuscleGroup::Chest,
            secondary_muscle_groups: Some(vec![MuscleGroup::Arms]),
            source: ExerciseSource::BuiltIn,
        };

        repo.save(&excercise).expect("save should succeed");

        let loaded = repo
            .get_by_id(&excercise.id)
            .expect("read should succeed")
            .expect("excercise should exist");

        assert_eq!(loaded.name, excercise.name);
        assert_eq!(loaded.kind, excercise.kind);
        assert_eq!(loaded.muscle_group, excercise.muscle_group);
        assert_eq!(
            loaded.secondary_muscle_groups,
            excercise.secondary_muscle_groups
        );
        assert_eq!(loaded.source, excercise.source);
    }

    #[test]
    fn different_users_see_own_excercises() {
        let db = SqliteExcerciseDb::in_memory().expect("db should initialize");
        let repo_a = db.for_user(UserId::new(1));
        let repo_b = db.for_user(UserId::new(2));

        let excercise = Exercise {
            id: domain::excercise::ExerciseId::new(),
            name: "Bench Press".to_string(),
            kind: ExerciseKind::Weighted,
            muscle_group: MuscleGroup::Chest,
            secondary_muscle_groups: None,
            source: ExerciseSource::UserDefined,
        };

        repo_a.save(&excercise).expect("save should succeed");
        assert_eq!(repo_a.get_all().unwrap().len(), 1);
        assert_eq!(repo_b.get_all().unwrap().len(), 0);
    }
}
