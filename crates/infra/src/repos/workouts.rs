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
    #[error("invalid load type: {0}")]
    InvalidLoadType(i64),
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
                start_date TEXT NOT NULL,
                end_date TEXT,
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

impl SqliteWorkoutRepo<'_> {
    fn build_workout(
        &self,
        id: WorkoutId,
        name: Option<String>,
        start_date: OffsetDateTime,
        end_date: Option<OffsetDateTime>,
    ) -> Result<Workout, SqliteWorkoutRepoError> {
        Ok(Workout {
            entries: self.load_workout_entries(&id)?,
            id,
            name,
            start_date,
            end_date,
        })
    }

    fn load_workout_entries(
        &self,
        workout_id: &WorkoutId,
    ) -> Result<Vec<WorkoutExercise>, SqliteWorkoutRepoError> {
        let mut stmt = self.connection.prepare(
            "SELECT excercise_id, notes
             FROM workout_exercises
             WHERE workout_id = ?1 AND user_id = ?2
             ORDER BY entry_order ASC",
        )?;

        let rows = stmt.query_map(params![workout_id, self.user_id], |row| {
            Ok((row.get::<_, ExcerciseId>(0)?, row.get::<_, Option<String>>(1)?))
        })?;

        rows.collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(|(excercise_id, notes)| {
                Ok(WorkoutExercise {
                    sets: self.load_performed_sets(workout_id, &excercise_id)?,
                    excercise_id,
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
        let mut stmt = self.connection.prepare(
            "SELECT reps, load_type, weight, weight_units
             FROM performed_sets
             WHERE workout_id = ?1 AND user_id = ?2 AND excercise_id = ?3
             ORDER BY set_order ASC",
        )?;

        let rows = stmt.query_map(params![workout_id, self.user_id, excercise_id], |row| {
            Ok((
                row.get::<_, u32>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, Option<f64>>(2)?,
                row.get::<_, Option<WeightUnits>>(3)?,
            ))
        })?;

        rows.collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(|(reps, load_type, weight, weight_units)| {
                Ok(PerformedSet {
                    reps,
                    kind: decode_load_type(load_type, weight, weight_units)?,
                })
            })
            .collect()
    }
}

impl WorkoutRepo for SqliteWorkoutRepo<'_> {
    type RepoError = SqliteWorkoutRepoError;

    fn get_all(&self) -> Result<Vec<Workout>, Self::RepoError> {
        let mut stmt = self.connection.prepare(
            "SELECT id, name, start_date, end_date
             FROM workouts
             WHERE user_id = ?1
             ORDER BY start_date DESC",
        )?;

        let rows = stmt.query_map(params![self.user_id], |row| {
            Ok((
                row.get::<_, WorkoutId>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, OffsetDateTime>(2)?,
                row.get::<_, Option<OffsetDateTime>>(3)?,
            ))
        })?;

        rows.collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(|(id, name, sd, ed)| self.build_workout(id, name, sd, ed))
            .collect()
    }

    fn get_by_id(&self, id: &WorkoutId) -> Result<Option<Workout>, Self::RepoError> {
        let mut stmt = self.connection.prepare(
            "SELECT id, name, start_date, end_date
             FROM workouts
             WHERE id = ?1 AND user_id = ?2",
        )?;

        let row = stmt
            .query_row(params![id, self.user_id], |row| {
                Ok((
                    row.get::<_, WorkoutId>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, OffsetDateTime>(2)?,
                    row.get::<_, Option<OffsetDateTime>>(3)?,
                ))
            })
            .optional()?;

        row.map(|(id, name, sd, ed)| self.build_workout(id, name, sd, ed))
            .transpose()
    }

    fn save(&self, workout: &Workout) -> Result<(), Self::RepoError> {
        self.connection.execute(
            "INSERT INTO workouts (id, user_id, name, start_date, end_date)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(id, user_id) DO UPDATE SET
                 name = excluded.name,
                 start_date = excluded.start_date,
                 end_date = excluded.end_date",
            params![
                workout.id,
                self.user_id,
                workout.name,
                workout.start_date,
                workout.end_date,
            ],
        )?;

        self.connection.execute(
            "DELETE FROM performed_sets WHERE workout_id = ?1 AND user_id = ?2",
            params![workout.id, self.user_id],
        )?;
        self.connection.execute(
            "DELETE FROM workout_exercises WHERE workout_id = ?1 AND user_id = ?2",
            params![workout.id, self.user_id],
        )?;

        for (i, entry) in workout.entries.iter().enumerate() {
            self.connection.execute(
                "INSERT INTO workout_exercises (workout_id, user_id, excercise_id, entry_order, notes)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![workout.id, self.user_id, entry.excercise_id, i as i64, entry.notes],
            )?;

            for (j, set) in entry.sets.iter().enumerate() {
                let (lt, w, wu) = encode_load_type(&set.kind);
                self.connection.execute(
                    "INSERT INTO performed_sets
                     (workout_id, user_id, excercise_id, set_order, reps, load_type, weight, weight_units)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    params![workout.id, self.user_id, entry.excercise_id, j as i64, set.reps, lt, w, wu],
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
            "SELECT COUNT(*) FROM workout_exercises WHERE workout_id = ?1 AND user_id = ?2",
            params![workout_id, self.user_id],
            |row| row.get(0),
        )?;

        self.connection.execute(
            "INSERT INTO workout_exercises (workout_id, user_id, excercise_id, entry_order, notes)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![workout_id, self.user_id, exercise.excercise_id, entry_order, exercise.notes],
        )?;

        for (j, set) in exercise.sets.iter().enumerate() {
            let (lt, w, wu) = encode_load_type(&set.kind);
            self.connection.execute(
                "INSERT INTO performed_sets
                 (workout_id, user_id, excercise_id, set_order, reps, load_type, weight, weight_units)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![workout_id, self.user_id, exercise.excercise_id, j as i64, set.reps, lt, w, wu],
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
            "SELECT COUNT(*) FROM performed_sets
             WHERE workout_id = ?1 AND user_id = ?2 AND excercise_id = ?3",
            params![workout_id, self.user_id, exercise_id],
            |row| row.get(0),
        )?;

        let (lt, w, wu) = encode_load_type(&set.kind);
        self.connection.execute(
            "INSERT INTO performed_sets
             (workout_id, user_id, excercise_id, set_order, reps, load_type, weight, weight_units)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![workout_id, self.user_id, exercise_id, set_order, set.reps, lt, w, wu],
        )?;

        Ok(())
    }

    fn get_by_date(&self, date: time::Date) -> Result<Vec<Workout>, Self::RepoError> {
        let mut stmt = self.connection.prepare(
            "SELECT id, name, start_date, end_date
             FROM workouts
             WHERE user_id = ?1 AND DATE(start_date) = ?2
             ORDER BY start_date DESC",
        )?;

        let rows = stmt.query_map(params![self.user_id, date], |row| {
            Ok((
                row.get::<_, WorkoutId>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, OffsetDateTime>(2)?,
                row.get::<_, Option<OffsetDateTime>>(3)?,
            ))
        })?;

        rows.collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(|(id, name, sd, ed)| self.build_workout(id, name, sd, ed))
            .collect()
    }
}

/// LoadType spans multiple columns — still needs manual encode/decode.
fn encode_load_type(v: &LoadType) -> (i64, Option<f64>, Option<&WeightUnits>) {
    match v {
        LoadType::Weighted(w) => (1, Some(w.value), Some(&w.units)),
        LoadType::BodyWeight => (2, None, None),
    }
}

fn decode_load_type(
    lt: i64,
    w: Option<f64>,
    wu: Option<WeightUnits>,
) -> Result<LoadType, SqliteWorkoutRepoError> {
    match lt {
        1 => Ok(LoadType::Weighted(Weight {
            value: w.ok_or(SqliteWorkoutRepoError::MissingWeightForWeightedSet)?,
            units: wu.ok_or(SqliteWorkoutRepoError::MissingWeightUnitsForWeightedSet)?,
        })),
        2 => Ok(LoadType::BodyWeight),
        _ => Err(SqliteWorkoutRepoError::InvalidLoadType(lt)),
    }
}

#[cfg(test)]
mod tests {
    use domain::{
        excercise::{LoadType, PerformedSet, Workout, WorkoutExercise},
        traits::WorkoutRepo,
        types::{UserId, Weight, WeightUnits},
    };

    use super::SqliteWorkoutDb;

    #[test]
    fn saves_and_loads_workout_with_entries_and_sets() {
        let db = SqliteWorkoutDb::in_memory().expect("db should initialize");
        let repo = db.for_user(UserId::new(1));

        let eid = domain::excercise::ExcerciseId::new();
        let mut workout = Workout::new(Some("Push Day".to_string()));
        let mut entry = WorkoutExercise::new(eid);
        entry.notes = Some("Top set first".to_string());
        entry.add_set(PerformedSet {
            kind: LoadType::Weighted(Weight { value: 100.0, units: WeightUnits::Pounds }),
            reps: 5,
        });
        entry.add_set(PerformedSet { kind: LoadType::BodyWeight, reps: 12 });
        workout.entries.push(entry);

        repo.save(&workout).expect("save should succeed");

        let loaded = repo.get_by_id(&workout.id).unwrap().unwrap();
        assert_eq!(loaded.name, workout.name);
        assert_eq!(loaded.entries.len(), 1);
        assert_eq!(loaded.entries[0].excercise_id, eid);
        assert_eq!(loaded.entries[0].notes, Some("Top set first".to_string()));
        assert_eq!(loaded.entries[0].sets.len(), 2);
        assert_eq!(loaded.entries[0].sets[0].reps, 5);
        assert_eq!(loaded.entries[0].sets[1].reps, 12);
    }

    #[test]
    fn adds_exercise_and_set_to_existing_workout() {
        let db = SqliteWorkoutDb::in_memory().expect("db should initialize");
        let repo = db.for_user(UserId::new(1));

        let workout = Workout::new(Some("Leg Day".to_string()));
        let eid = domain::excercise::ExcerciseId::new();

        repo.save(&workout).unwrap();
        repo.add_exercise(&workout.id, &WorkoutExercise::new(eid)).unwrap();
        repo.add_set(
            &workout.id,
            &eid,
            &PerformedSet {
                kind: LoadType::Weighted(Weight { value: 140.0, units: WeightUnits::Pounds }),
                reps: 3,
            },
        )
        .unwrap();

        let loaded = repo.get_by_id(&workout.id).unwrap().unwrap();
        assert_eq!(loaded.entries.len(), 1);
        assert_eq!(loaded.entries[0].sets.len(), 1);
        assert_eq!(loaded.entries[0].sets[0].reps, 3);
    }

    #[test]
    fn different_users_see_own_workouts() {
        let db = SqliteWorkoutDb::in_memory().expect("db should initialize");
        let repo_a = db.for_user(UserId::new(1));
        let repo_b = db.for_user(UserId::new(2));

        let workout = Workout::new(Some("Push Day".to_string()));
        repo_a.save(&workout).unwrap();

        assert_eq!(repo_a.get_all().unwrap().len(), 1);
        assert_eq!(repo_b.get_all().unwrap().len(), 0);
    }
}
