use std::path::Path;

use domain::{
    excercise::{Excercise, ExcerciseId, ExcerciseKind, ExcerciseSource, MuscleGroup},
    traits::ExcerciseRepo,
};
use rusqlite::{Connection, OptionalExtension, params};

#[derive(Debug)]
pub enum SqliteExcerciseRepoError {
    Sqlite(rusqlite::Error),
    InvalidExcerciseId(uuid::Error),
    InvalidExcerciseKind(i64),
    InvalidExcerciseSource(i64),
    InvalidMuscleGroup(i64),
    InvalidSecondaryMuscleGroup(i64),
}

impl std::fmt::Display for SqliteExcerciseRepoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sqlite(err) => write!(f, "sqlite error: {err}"),
            Self::InvalidExcerciseId(err) => write!(f, "invalid excercise id: {err}"),
            Self::InvalidExcerciseKind(value) => write!(f, "invalid excercise kind: {value}"),
            Self::InvalidExcerciseSource(value) => {
                write!(f, "invalid excercise source: {value}")
            }
            Self::InvalidMuscleGroup(value) => write!(f, "invalid muscle group: {value}"),
            Self::InvalidSecondaryMuscleGroup(value) => {
                write!(f, "invalid secondary muscle group: {value}")
            }
        }
    }
}

impl std::error::Error for SqliteExcerciseRepoError {}

impl From<rusqlite::Error> for SqliteExcerciseRepoError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

pub struct SqliteExcerciseRepo {
    connection: Connection,
}

impl SqliteExcerciseRepo {
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

    fn init_schema(connection: &Connection) -> Result<(), SqliteExcerciseRepoError> {
        connection.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS excercises (
                id TEXT PRIMARY KEY NOT NULL,
                name TEXT NOT NULL,
                kind INTEGER NOT NULL,
                muscle_group INTEGER NOT NULL,
                secondary_muscle_groups TEXT,
                source INTEGER NOT NULL
            );
            ",
        )?;

        Ok(())
    }

    fn build_excercise(
        id: String,
        name: String,
        kind: i64,
        muscle_group: i64,
        secondary_muscle_groups: Option<String>,
        source: i64,
    ) -> Result<Excercise, SqliteExcerciseRepoError> {
        Ok(Excercise {
            id: ExcerciseId::from_uuid(
                uuid::Uuid::parse_str(&id).map_err(SqliteExcerciseRepoError::InvalidExcerciseId)?,
            ),
            name,
            kind: decode_excercise_kind(kind)?,
            muscle_group: decode_muscle_group(muscle_group)?,
            secondary_muscle_groups: decode_secondary_muscle_groups(secondary_muscle_groups)?,
            source: decode_excercise_source(source)?,
        })
    }
}

impl ExcerciseRepo for SqliteExcerciseRepo {
    type RepoError = SqliteExcerciseRepoError;

    fn get_by_id(&self, id: &ExcerciseId) -> Result<Option<Excercise>, Self::RepoError> {
        let mut statement = self.connection.prepare(
            "
            SELECT id, name, kind, muscle_group, secondary_muscle_groups, source
            FROM excercises
            WHERE id = ?1
            ",
        )?;

        let row = statement
            .query_row(params![id.as_uuid().to_string()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, i64>(5)?,
                ))
            })
            .optional()?;

        row.map(
            |(id, name, kind, muscle_group, secondary_muscle_groups, source)| {
                Self::build_excercise(
                    id,
                    name,
                    kind,
                    muscle_group,
                    secondary_muscle_groups,
                    source,
                )
            },
        )
        .transpose()
    }

    fn save(&self, exercise: &Excercise) -> Result<(), Self::RepoError> {
        let secondary_muscle_groups =
            encode_secondary_muscle_groups(&exercise.secondary_muscle_groups);

        self.connection.execute(
            "
            INSERT INTO excercises (id, name, kind, muscle_group, secondary_muscle_groups, source)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                kind = excluded.kind,
                muscle_group = excluded.muscle_group,
                secondary_muscle_groups = excluded.secondary_muscle_groups,
                source = excluded.source
            ",
            params![
                exercise.id.as_uuid().to_string(),
                &exercise.name,
                encode_excercise_kind(exercise.kind),
                encode_muscle_group(exercise.muscle_group),
                secondary_muscle_groups,
                encode_excercise_source(exercise.source),
            ],
        )?;

        Ok(())
    }

    fn get_all(&self) -> Result<Vec<Excercise>, Self::RepoError> {
        let mut statement = self.connection.prepare(
            "
            SELECT id, name, kind, muscle_group, secondary_muscle_groups, source
            FROM excercises
            ORDER BY name ASC
            ",
        )?;
        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, i64>(5)?,
            ))
        })?;

        let stored_rows = rows.collect::<Result<Vec<_>, _>>()?;

        stored_rows
            .into_iter()
            .map(
                |(id, name, kind, muscle_group, secondary_muscle_groups, source)| {
                    Self::build_excercise(
                        id,
                        name,
                        kind,
                        muscle_group,
                        secondary_muscle_groups,
                        source,
                    )
                },
            )
            .collect()
    }
}

fn encode_excercise_kind(value: ExcerciseKind) -> i64 {
    match value {
        ExcerciseKind::Weighted => 1,
        ExcerciseKind::BodyWeight => 2,
    }
}

fn decode_excercise_kind(value: i64) -> Result<ExcerciseKind, SqliteExcerciseRepoError> {
    match value {
        1 => Ok(ExcerciseKind::Weighted),
        2 => Ok(ExcerciseKind::BodyWeight),
        _ => Err(SqliteExcerciseRepoError::InvalidExcerciseKind(value)),
    }
}

fn encode_excercise_source(value: ExcerciseSource) -> i64 {
    match value {
        ExcerciseSource::BuiltIn => 1,
        ExcerciseSource::UserDefined => 2,
    }
}

fn decode_excercise_source(value: i64) -> Result<ExcerciseSource, SqliteExcerciseRepoError> {
    match value {
        1 => Ok(ExcerciseSource::BuiltIn),
        2 => Ok(ExcerciseSource::UserDefined),
        _ => Err(SqliteExcerciseRepoError::InvalidExcerciseSource(value)),
    }
}

fn encode_muscle_group(value: MuscleGroup) -> i64 {
    match value {
        MuscleGroup::Chest => 1,
        MuscleGroup::Back => 2,
        MuscleGroup::Arms => 3,
        MuscleGroup::Legs => 4,
        MuscleGroup::Core => 5,
    }
}

fn decode_muscle_group(value: i64) -> Result<MuscleGroup, SqliteExcerciseRepoError> {
    match value {
        1 => Ok(MuscleGroup::Chest),
        2 => Ok(MuscleGroup::Back),
        3 => Ok(MuscleGroup::Arms),
        4 => Ok(MuscleGroup::Legs),
        5 => Ok(MuscleGroup::Core),
        _ => Err(SqliteExcerciseRepoError::InvalidMuscleGroup(value)),
    }
}

fn encode_secondary_muscle_groups(value: &Option<Vec<MuscleGroup>>) -> Option<String> {
    value.as_ref().map(|muscle_groups| {
        muscle_groups
            .iter()
            .map(|group| encode_muscle_group(*group).to_string())
            .collect::<Vec<_>>()
            .join(",")
    })
}

fn decode_secondary_muscle_groups(
    value: Option<String>,
) -> Result<Option<Vec<MuscleGroup>>, SqliteExcerciseRepoError> {
    let Some(value) = value else {
        return Ok(None);
    };

    if value.is_empty() {
        return Ok(Some(vec![]));
    }

    value
        .split(',')
        .map(|raw| {
            raw.parse::<i64>()
                .map_err(|_| SqliteExcerciseRepoError::InvalidSecondaryMuscleGroup(-1))
                .and_then(|parsed| {
                    decode_muscle_group(parsed)
                        .map_err(|_| SqliteExcerciseRepoError::InvalidSecondaryMuscleGroup(parsed))
                })
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Some)
}

#[cfg(test)]
mod tests {
    use domain::{
        excercise::{Excercise, ExcerciseKind, ExcerciseSource, MuscleGroup},
        traits::ExcerciseRepo,
    };

    use super::SqliteExcerciseRepo;

    #[test]
    fn saves_and_loads_excercise() {
        let repo = SqliteExcerciseRepo::in_memory().expect("repo should initialize");
        let excercise = Excercise {
            id: domain::excercise::ExcerciseId::new(),
            name: "Bench Press".to_string(),
            kind: ExcerciseKind::Weighted,
            muscle_group: MuscleGroup::Chest,
            secondary_muscle_groups: Some(vec![MuscleGroup::Arms]),
            source: ExcerciseSource::BuiltIn,
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
}
