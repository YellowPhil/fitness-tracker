use std::path::Path;

use domain::{
    excercise::{ExcerciseId, LoadType, PerformedSet, Workout, WorkoutExercise, WorkoutId},
    traits::WorkoutRepo,
    types::{Weight, WeightUnits},
};
use rusqlite::{Connection, OptionalExtension, params};
use time::OffsetDateTime;

#[derive(Debug)]
pub enum SqliteWorkoutRepoError {
    Sqlite(rusqlite::Error),
    InvalidWorkoutId(uuid::Error),
    InvalidExcerciseId(uuid::Error),
    InvalidTimestamp(i64),
    InvalidReps(i64),
    InvalidLoadType(i64),
    InvalidWeightUnits(i64),
    MissingWeightForWeightedSet,
    MissingWeightUnitsForWeightedSet,
}

impl std::fmt::Display for SqliteWorkoutRepoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sqlite(err) => write!(f, "sqlite error: {err}"),
            Self::InvalidWorkoutId(err) => write!(f, "invalid workout id: {err}"),
            Self::InvalidExcerciseId(err) => write!(f, "invalid excercise id: {err}"),
            Self::InvalidTimestamp(value) => write!(f, "invalid timestamp: {value}"),
            Self::InvalidReps(value) => write!(f, "invalid reps: {value}"),
            Self::InvalidLoadType(value) => write!(f, "invalid load type: {value}"),
            Self::InvalidWeightUnits(value) => write!(f, "invalid weight units: {value}"),
            Self::MissingWeightForWeightedSet => write!(f, "missing weight for weighted set"),
            Self::MissingWeightUnitsForWeightedSet => {
                write!(f, "missing weight units for weighted set")
            }
        }
    }
}

impl std::error::Error for SqliteWorkoutRepoError {}

impl From<rusqlite::Error> for SqliteWorkoutRepoError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

pub struct SqliteWorkoutRepo {
    connection: Connection,
}

impl SqliteWorkoutRepo {
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

    fn init_schema(connection: &Connection) -> Result<(), SqliteWorkoutRepoError> {
        connection.execute_batch(
            "
            PRAGMA foreign_keys = ON;

            CREATE TABLE IF NOT EXISTS workouts (
                id TEXT PRIMARY KEY NOT NULL,
                name TEXT,
                start_date INTEGER NOT NULL,
                end_date INTEGER
            );

            CREATE TABLE IF NOT EXISTS workout_exercises (
                workout_id TEXT NOT NULL,
                excercise_id TEXT NOT NULL,
                entry_order INTEGER NOT NULL,
                notes TEXT,
                PRIMARY KEY (workout_id, excercise_id),
                FOREIGN KEY (workout_id) REFERENCES workouts(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS performed_sets (
                workout_id TEXT NOT NULL,
                excercise_id TEXT NOT NULL,
                set_order INTEGER NOT NULL,
                reps INTEGER NOT NULL,
                load_type INTEGER NOT NULL,
                weight REAL,
                weight_units INTEGER,
                PRIMARY KEY (workout_id, excercise_id, set_order),
                FOREIGN KEY (workout_id, excercise_id)
                    REFERENCES workout_exercises(workout_id, excercise_id)
                    ON DELETE CASCADE
            );
            ",
        )?;

        Ok(())
    }

    fn build_workout(
        &self,
        id: String,
        name: Option<String>,
        start_date: i64,
        end_date: Option<i64>,
    ) -> Result<Workout, SqliteWorkoutRepoError> {
        let workout_id = WorkoutId::from_uuid(
            uuid::Uuid::parse_str(&id).map_err(SqliteWorkoutRepoError::InvalidWorkoutId)?,
        );

        Ok(Workout {
            id: workout_id,
            name,
            start_date: decode_timestamp(start_date)?,
            end_date: end_date.map(decode_timestamp).transpose()?,
            entries: self.load_workout_entries(&workout_id)?,
        })
    }

    fn load_workout_entries(
        &self,
        workout_id: &WorkoutId,
    ) -> Result<Vec<WorkoutExercise>, SqliteWorkoutRepoError> {
        let mut statement = self.connection.prepare(
            "
            SELECT excercise_id, notes
            FROM workout_exercises
            WHERE workout_id = ?1
            ORDER BY entry_order ASC
            ",
        )?;

        let rows = statement.query_map(params![workout_id.as_uuid().to_string()], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?))
        })?;
        let stored_rows = rows.collect::<Result<Vec<_>, _>>()?;

        stored_rows
            .into_iter()
            .map(|(excercise_id, notes)| {
                let excercise_id = ExcerciseId::from_uuid(
                    uuid::Uuid::parse_str(&excercise_id)
                        .map_err(SqliteWorkoutRepoError::InvalidExcerciseId)?,
                );

                Ok(WorkoutExercise {
                    excercise_id,
                    sets: self.load_performed_sets(workout_id, &excercise_id)?,
                    notes,
                })
            })
            .collect()
    }

    fn load_performed_sets(
        &self,
        workout_id: &WorkoutId,
        excercise_id: &ExcerciseId,
    ) -> Result<Vec<PerformedSet>, SqliteWorkoutRepoError> {
        let mut statement = self.connection.prepare(
            "
            SELECT reps, load_type, weight, weight_units
            FROM performed_sets
            WHERE workout_id = ?1 AND excercise_id = ?2
            ORDER BY set_order ASC
            ",
        )?;

        let rows = statement.query_map(
            params![
                workout_id.as_uuid().to_string(),
                excercise_id.as_uuid().to_string()
            ],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Option<f64>>(2)?,
                    row.get::<_, Option<i64>>(3)?,
                ))
            },
        )?;
        let stored_rows = rows.collect::<Result<Vec<_>, _>>()?;

        stored_rows
            .into_iter()
            .map(|(reps, load_type, weight, weight_units)| {
                Ok(PerformedSet {
                    reps: decode_reps(reps)?,
                    kind: decode_load_type(load_type, weight, weight_units)?,
                })
            })
            .collect()
    }
}

impl WorkoutRepo for SqliteWorkoutRepo {
    type RepoError = SqliteWorkoutRepoError;

    fn get_all(&self) -> Result<Vec<Workout>, Self::RepoError> {
        let mut statement = self.connection.prepare(
            "
            SELECT id, name, start_date, end_date
            FROM workouts
            ORDER BY start_date DESC
            "
        )?;

        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, Option<i64>>(3)?,
            ))
        })?;
        let stored_rows = rows.collect::<Result<Vec<_>, _>>()?;

        stored_rows
            .into_iter()
            .map(|(id, name, start_date, end_date)| {
                self.build_workout(id, name, start_date, end_date)
            })
            .collect()
    }

    fn get_by_id(&self, id: &WorkoutId) -> Result<Option<Workout>, Self::RepoError> {
        let mut statement = self.connection.prepare(
            "
            SELECT id, name, start_date, end_date
            FROM workouts
            WHERE id = ?1
            ",
        )?;

        let row = statement
            .query_row(params![id.as_uuid().to_string()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, Option<i64>>(3)?,
                ))
            })
            .optional()?;

        row.map(|(id, name, start_date, end_date)| {
            self.build_workout(id, name, start_date, end_date)
        })
        .transpose()
    }

    fn save(&self, workout: &Workout) -> Result<(), Self::RepoError> {
        self.connection.execute(
            "
            INSERT INTO workouts (id, name, start_date, end_date)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                start_date = excluded.start_date,
                end_date = excluded.end_date
            ",
            params![
                workout.id.as_uuid().to_string(),
                workout.name.as_deref(),
                workout.start_date.unix_timestamp(),
                workout.end_date.map(|date| date.unix_timestamp()),
            ],
        )?;

        self.connection.execute(
            "DELETE FROM performed_sets WHERE workout_id = ?1",
            params![workout.id.as_uuid().to_string()],
        )?;
        self.connection.execute(
            "DELETE FROM workout_exercises WHERE workout_id = ?1",
            params![workout.id.as_uuid().to_string()],
        )?;

        for (entry_index, entry) in workout.entries.iter().enumerate() {
            self.connection.execute(
                "
                INSERT INTO workout_exercises (workout_id, excercise_id, entry_order, notes)
                VALUES (?1, ?2, ?3, ?4)
                ",
                params![
                    workout.id.as_uuid().to_string(),
                    entry.excercise_id.as_uuid().to_string(),
                    entry_index as i64,
                    entry.notes.as_deref(),
                ],
            )?;

            for (set_index, set) in entry.sets.iter().enumerate() {
                let (load_type, weight, weight_units) = encode_load_type(&set.kind);

                self.connection.execute(
                    "
                    INSERT INTO performed_sets (
                        workout_id,
                        excercise_id,
                        set_order,
                        reps,
                        load_type,
                        weight,
                        weight_units
                    )
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                    ",
                    params![
                        workout.id.as_uuid().to_string(),
                        entry.excercise_id.as_uuid().to_string(),
                        set_index as i64,
                        set.reps as i64,
                        load_type,
                        weight,
                        weight_units,
                    ],
                )?;
            }
        }

        Ok(())
    }

    fn add_exercise(
        &self,
        workout_id: &WorkoutId,
        exercise: &WorkoutExercise,
    ) -> Result<(), Self::RepoError> {
        let entry_order: i64 = self.connection.query_row(
            "
            SELECT COUNT(*)
            FROM workout_exercises
            WHERE workout_id = ?1
            ",
            params![workout_id.as_uuid().to_string()],
            |row| row.get(0),
        )?;

        self.connection.execute(
            "
            INSERT INTO workout_exercises (workout_id, excercise_id, entry_order, notes)
            VALUES (?1, ?2, ?3, ?4)
            ",
            params![
                workout_id.as_uuid().to_string(),
                exercise.excercise_id.as_uuid().to_string(),
                entry_order,
                exercise.notes.as_deref(),
            ],
        )?;

        for (set_index, set) in exercise.sets.iter().enumerate() {
            let (load_type, weight, weight_units) = encode_load_type(&set.kind);

            self.connection.execute(
                "
                INSERT INTO performed_sets (
                    workout_id,
                    excercise_id,
                    set_order,
                    reps,
                    load_type,
                    weight,
                    weight_units
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                ",
                params![
                    workout_id.as_uuid().to_string(),
                    exercise.excercise_id.as_uuid().to_string(),
                    set_index as i64,
                    set.reps as i64,
                    load_type,
                    weight,
                    weight_units,
                ],
            )?;
        }

        Ok(())
    }

    fn add_set(
        &self,
        workout_id: &WorkoutId,
        exercise_id: &ExcerciseId,
        set: &PerformedSet,
    ) -> Result<(), Self::RepoError> {
        let set_order: i64 = self.connection.query_row(
            "
            SELECT COUNT(*)
            FROM performed_sets
            WHERE workout_id = ?1 AND excercise_id = ?2
            ",
            params![
                workout_id.as_uuid().to_string(),
                exercise_id.as_uuid().to_string()
            ],
            |row| row.get(0),
        )?;
        let (load_type, weight, weight_units) = encode_load_type(&set.kind);

        self.connection.execute(
            "
            INSERT INTO performed_sets (
                workout_id,
                excercise_id,
                set_order,
                reps,
                load_type,
                weight,
                weight_units
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ",
            params![
                workout_id.as_uuid().to_string(),
                exercise_id.as_uuid().to_string(),
                set_order,
                set.reps as i64,
                load_type,
                weight,
                weight_units,
            ],
        )?;

        Ok(())
    }
}

fn decode_timestamp(value: i64) -> Result<OffsetDateTime, SqliteWorkoutRepoError> {
    OffsetDateTime::from_unix_timestamp(value)
        .map_err(|_| SqliteWorkoutRepoError::InvalidTimestamp(value))
}

fn decode_reps(value: i64) -> Result<u32, SqliteWorkoutRepoError> {
    u32::try_from(value).map_err(|_| SqliteWorkoutRepoError::InvalidReps(value))
}

fn encode_load_type(value: &LoadType) -> (i64, Option<f64>, Option<i64>) {
    match value {
        LoadType::Weighted(load) => (1, Some(load.value), Some(encode_weight_units(load.units))),
        LoadType::BodyWeight => (2, None, None),
    }
}

fn decode_load_type(
    load_type: i64,
    weight: Option<f64>,
    weight_units: Option<i64>,
) -> Result<LoadType, SqliteWorkoutRepoError> {
    match load_type {
        1 => Ok(LoadType::Weighted(Weight {
            value: weight.ok_or(SqliteWorkoutRepoError::MissingWeightForWeightedSet)?,
            units: decode_weight_units(
                    weight_units.ok_or(SqliteWorkoutRepoError::MissingWeightUnitsForWeightedSet)?,
                )?,
            }),
        ),
        2 => Ok(LoadType::BodyWeight),
        _ => Err(SqliteWorkoutRepoError::InvalidLoadType(load_type)),
    }
}

fn encode_weight_units(value: WeightUnits) -> i64 {
    match value {
        WeightUnits::Kilograms => 1,
        WeightUnits::Pounds => 2,
    }
}

fn decode_weight_units(value: i64) -> Result<WeightUnits, SqliteWorkoutRepoError> {
    match value {
        1 => Ok(WeightUnits::Kilograms),
        2 => Ok(WeightUnits::Pounds),
        _ => Err(SqliteWorkoutRepoError::InvalidWeightUnits(value)),
    }
}

#[cfg(test)]
mod tests {
    use domain::{
        excercise::{LoadType, PerformedSet, Workout, WorkoutExercise},
        traits::WorkoutRepo,
        types::{Weight, WeightUnits},
    };

    use super::SqliteWorkoutRepo;

    #[test]
    fn saves_and_loads_workout_with_entries_and_sets() {
        let repo = SqliteWorkoutRepo::in_memory().expect("repo should initialize");
        let excercise_id = domain::excercise::ExcerciseId::new();
        let mut workout = Workout::new(Some("Push Day".to_string()));
        let mut entry = WorkoutExercise::new(excercise_id);
        entry.notes = Some("Top set first".to_string());
        entry.add_set(PerformedSet {
            kind: LoadType::Weighted(Weight {
                value: 100.0,
                units: WeightUnits::Pounds,
            }),
            reps: 5,
        });
        entry.add_set(PerformedSet {
            kind: LoadType::BodyWeight,
            reps: 12,
        });
        workout.entries.push(entry);

        repo.save(&workout).expect("save should succeed");

        let loaded = repo
            .get_by_id(&workout.id)
            .expect("read should succeed")
            .expect("workout should exist");

        assert_eq!(loaded.name, workout.name);
        assert_eq!(loaded.entries.len(), 1);
        assert_eq!(loaded.entries[0].excercise_id, excercise_id);
        assert_eq!(loaded.entries[0].notes, Some("Top set first".to_string()));
        assert_eq!(loaded.entries[0].sets.len(), 2);
        assert_eq!(loaded.entries[0].sets[0].reps, 5);
        assert_eq!(loaded.entries[0].sets[1].reps, 12);
    }

    #[test]
    fn adds_exercise_and_set_to_existing_workout() {
        let repo = SqliteWorkoutRepo::in_memory().expect("repo should initialize");
        let workout = Workout::new(Some("Leg Day".to_string()));
        let excercise_id = domain::excercise::ExcerciseId::new();

        repo.save(&workout).expect("save should succeed");
        repo.add_exercise(&workout.id, &WorkoutExercise::new(excercise_id))
            .expect("exercise add should succeed");
        repo.add_set(
            &workout.id,
            &excercise_id,
            &PerformedSet {
                kind: LoadType::Weighted(Weight {
                    value: 140.0,
                    units: WeightUnits::Pounds,
                }),
                reps: 3,
            },
        )
        .expect("set add should succeed");

        let loaded = repo
            .get_by_id(&workout.id)
            .expect("read should succeed")
            .expect("workout should exist");

        assert_eq!(loaded.entries.len(), 1);
        assert_eq!(loaded.entries[0].sets.len(), 1);
        assert_eq!(loaded.entries[0].sets[0].reps, 3);
    }
}
