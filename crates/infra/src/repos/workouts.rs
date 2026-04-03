use std::sync::{MutexGuard, PoisonError};

use domain::{
    excercise::{ExerciseId, LoadType, PerformedSet, Workout, WorkoutExercise, WorkoutId},
    traits::WorkoutRepo,
    types::{UserId, Weight, WeightUnits},
};
use postgres::{Client, Row};
use time::{Date, OffsetDateTime};

use super::{
    postgres::{SharedClient, connect},
    postgres_types::{PgLoadType, PgWeightUnits},
};

#[derive(Debug, thiserror::Error)]
pub enum PostgresWorkoutRepoError {
    #[error("postgres error: {0}")]
    Postgres(#[from] postgres::Error),
    #[error("postgres connection lock poisoned")]
    ConnectionPoisoned,
    #[error("weighted set missing weight value")]
    MissingWeightForWeightedSet,
    #[error("weighted set missing weight units")]
    MissingWeightUnitsForWeightedSet,
    #[error("invalid reps value from database: {0}")]
    InvalidReps(i32),
    #[error("value for {field} exceeds supported range: {value}")]
    ValueOutOfRange { field: &'static str, value: usize },
    #[error("count for {field} exceeds supported range: {value}")]
    CountOutOfRange { field: &'static str, value: i64 },
}

pub struct PostgresWorkoutDb {
    client: SharedClient,
}

pub struct PostgresWorkoutRepo<'db> {
    client: &'db SharedClient,
    user_id: UserId,
}

impl PostgresWorkoutDb {
    pub fn open(url: &str) -> Result<Self, PostgresWorkoutRepoError> {
        Ok(Self {
            client: connect(url)?,
        })
    }

    pub(crate) fn new(client: SharedClient) -> Self {
        Self { client }
    }

    pub fn for_user(&self, user_id: UserId) -> PostgresWorkoutRepo<'_> {
        PostgresWorkoutRepo {
            client: &self.client,
            user_id,
        }
    }
}

impl PostgresWorkoutRepo<'_> {
    fn client(&self) -> Result<MutexGuard<'_, Client>, PostgresWorkoutRepoError> {
        self.client
            .lock()
            .map_err(|_: PoisonError<MutexGuard<'_, Client>>| {
                PostgresWorkoutRepoError::ConnectionPoisoned
            })
    }

    fn build_workout(
        &self,
        id: WorkoutId,
        name: Option<String>,
        start_date: OffsetDateTime,
        end_date: Option<OffsetDateTime>,
    ) -> Result<Workout, PostgresWorkoutRepoError> {
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
    ) -> Result<Vec<WorkoutExercise>, PostgresWorkoutRepoError> {
        let mut client = self.client()?;
        let rows = client.query(
            "SELECT exercise_id, notes
             FROM workout_exercises
             WHERE workout_id = $1 AND user_id = $2
             ORDER BY entry_order ASC",
            &[workout_id.as_uuid(), &self.user_id.as_i64()],
        )?;
        drop(client);

        rows.into_iter()
            .map(|row| {
                let exercise_id = ExerciseId::from_uuid(row.get("exercise_id"));
                Ok(WorkoutExercise {
                    sets: self.load_performed_sets(workout_id, &exercise_id)?,
                    exercise_id,
                    notes: row.get("notes"),
                })
            })
            .collect()
    }

    fn load_performed_sets(
        &self,
        workout_id: &WorkoutId,
        exercise_id: &ExerciseId,
    ) -> Result<Vec<PerformedSet>, PostgresWorkoutRepoError> {
        let mut client = self.client()?;
        let rows = client.query(
            "SELECT reps, load_type, weight_value, weight_units
             FROM performed_sets
             WHERE workout_id = $1 AND user_id = $2 AND exercise_id = $3
             ORDER BY set_order ASC",
            &[workout_id.as_uuid(), &self.user_id.as_i64(), exercise_id.as_uuid()],
        )?;

        rows.into_iter()
            .map(stored_set_from_row)
            .map(|result| result.and_then(StoredSet::into_domain))
            .collect()
    }
}

impl WorkoutRepo for PostgresWorkoutRepo<'_> {
    type RepoError = PostgresWorkoutRepoError;

    fn get_all(&self) -> Result<Vec<Workout>, Self::RepoError> {
        let mut client = self.client()?;
        let rows = client.query(
            "SELECT id, name, start_date, end_date
             FROM workouts
             WHERE user_id = $1
             ORDER BY start_date DESC",
            &[&self.user_id.as_i64()],
        )?;
        drop(client);

        rows.into_iter()
            .map(workout_header_from_row)
            .map(|result| {
                result.and_then(|(id, name, start_date, end_date)| {
                    self.build_workout(id, name, start_date, end_date)
                })
            })
            .collect()
    }

    fn get_by_id(&self, id: &WorkoutId) -> Result<Option<Workout>, Self::RepoError> {
        let mut client = self.client()?;
        let row = client.query_opt(
            "SELECT id, name, start_date, end_date
             FROM workouts
             WHERE id = $1 AND user_id = $2",
            &[id.as_uuid(), &self.user_id.as_i64()],
        )?;
        drop(client);

        row.map(workout_header_from_row)
            .transpose()?
            .map(|(id, name, start_date, end_date)| self.build_workout(id, name, start_date, end_date))
            .transpose()
    }

    fn save(&self, workout: &Workout) -> Result<(), Self::RepoError> {
        let mut client = self.client()?;
        let mut tx = client.transaction()?;

        tx.execute(
            "INSERT INTO workouts (id, user_id, name, start_date, end_date)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (id, user_id) DO UPDATE SET
                name = EXCLUDED.name,
                start_date = EXCLUDED.start_date,
                end_date = EXCLUDED.end_date",
            &[
                workout.id.as_uuid(),
                &self.user_id.as_i64(),
                &workout.name,
                &workout.start_date,
                &workout.end_date,
            ],
        )?;

        tx.execute(
            "DELETE FROM performed_sets WHERE workout_id = $1 AND user_id = $2",
            &[workout.id.as_uuid(), &self.user_id.as_i64()],
        )?;
        tx.execute(
            "DELETE FROM workout_exercises WHERE workout_id = $1 AND user_id = $2",
            &[workout.id.as_uuid(), &self.user_id.as_i64()],
        )?;

        for (entry_order, entry) in workout.entries.iter().enumerate() {
            let entry_order = to_i32(entry_order, "entry_order")?;
            tx.execute(
                "INSERT INTO workout_exercises (workout_id, user_id, exercise_id, entry_order, notes)
                 VALUES ($1, $2, $3, $4, $5)",
                &[
                    workout.id.as_uuid(),
                    &self.user_id.as_i64(),
                    entry.exercise_id.as_uuid(),
                    &entry_order,
                    &entry.notes,
                ],
            )?;

            for (set_order, set) in entry.sets.iter().enumerate() {
                let stored = StoredSet::from_domain(set)?;
                let set_order = to_i32(set_order, "set_order")?;
                tx.execute(
                    "INSERT INTO performed_sets (
                        workout_id,
                        user_id,
                        exercise_id,
                        set_order,
                        reps,
                        load_type,
                        weight_value,
                        weight_units
                     )
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
                    &[
                        workout.id.as_uuid(),
                        &self.user_id.as_i64(),
                        entry.exercise_id.as_uuid(),
                        &set_order,
                        &stored.reps,
                        &stored.load_type,
                        &stored.weight_value,
                        &stored.weight_units,
                    ],
                )?;
            }
        }

        tx.commit()?;
        Ok(())
    }

    fn add_exercise(
        &self,
        workout_id: &WorkoutId,
        exercise: &WorkoutExercise,
    ) -> Result<(), Self::RepoError> {
        let mut client = self.client()?;
        let mut tx = client.transaction()?;
        let entry_order: i64 = tx.query_one(
            "SELECT COUNT(*) FROM workout_exercises WHERE workout_id = $1 AND user_id = $2",
            &[workout_id.as_uuid(), &self.user_id.as_i64()],
        )?
        .get(0);
        let entry_order = count_to_i32(entry_order, "entry_order")?;

        tx.execute(
            "INSERT INTO workout_exercises (workout_id, user_id, exercise_id, entry_order, notes)
             VALUES ($1, $2, $3, $4, $5)",
            &[
                workout_id.as_uuid(),
                &self.user_id.as_i64(),
                exercise.exercise_id.as_uuid(),
                &entry_order,
                &exercise.notes,
            ],
        )?;

        for (set_order, set) in exercise.sets.iter().enumerate() {
            let stored = StoredSet::from_domain(set)?;
            let set_order = to_i32(set_order, "set_order")?;
            tx.execute(
                "INSERT INTO performed_sets (
                    workout_id,
                    user_id,
                    exercise_id,
                    set_order,
                    reps,
                    load_type,
                    weight_value,
                    weight_units
                 )
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
                &[
                    workout_id.as_uuid(),
                    &self.user_id.as_i64(),
                    exercise.exercise_id.as_uuid(),
                    &set_order,
                    &stored.reps,
                    &stored.load_type,
                    &stored.weight_value,
                    &stored.weight_units,
                ],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    fn add_set(
        &self,
        workout_id: &WorkoutId,
        exercise_id: &ExerciseId,
        set: &PerformedSet,
    ) -> Result<(), Self::RepoError> {
        let stored = StoredSet::from_domain(set)?;
        let mut client = self.client()?;
        let set_order: i64 = client
            .query_one(
                "SELECT COUNT(*) FROM performed_sets
                 WHERE workout_id = $1 AND user_id = $2 AND exercise_id = $3",
                &[workout_id.as_uuid(), &self.user_id.as_i64(), exercise_id.as_uuid()],
            )?
            .get(0);
        let set_order = count_to_i32(set_order, "set_order")?;

        client.execute(
            "INSERT INTO performed_sets (
                workout_id,
                user_id,
                exercise_id,
                set_order,
                reps,
                load_type,
                weight_value,
                weight_units
             )
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            &[
                workout_id.as_uuid(),
                &self.user_id.as_i64(),
                exercise_id.as_uuid(),
                &set_order,
                &stored.reps,
                &stored.load_type,
                &stored.weight_value,
                &stored.weight_units,
            ],
        )?;

        Ok(())
    }

    fn get_by_date(&self, date: Date) -> Result<Vec<Workout>, Self::RepoError> {
        let mut client = self.client()?;
        let rows = client.query(
            "SELECT id, name, start_date, end_date
             FROM workouts
             WHERE user_id = $1 AND start_date::date = $2
             ORDER BY start_date DESC",
            &[&self.user_id.as_i64(), &date],
        )?;
        drop(client);

        rows.into_iter()
            .map(workout_header_from_row)
            .map(|result| {
                result.and_then(|(id, name, start_date, end_date)| {
                    self.build_workout(id, name, start_date, end_date)
                })
            })
            .collect()
    }

    fn get_latest(&self) -> Result<Option<Workout>, Self::RepoError> {
        let mut client = self.client()?;
        let row = client.query_opt(
            "SELECT id, name, start_date, end_date
             FROM workouts
             WHERE user_id = $1
             ORDER BY start_date DESC
             LIMIT 1",
            &[&self.user_id.as_i64()],
        )?;
        drop(client);

        row.map(workout_header_from_row)
            .transpose()?
            .map(|(id, name, start_date, end_date)| self.build_workout(id, name, start_date, end_date))
            .transpose()
    }

    fn get_last_n(&self, n: usize) -> Result<Vec<Workout>, Self::RepoError> {
        let limit = to_i64(n, "limit")?;
        let mut client = self.client()?;
        let rows = client.query(
            "SELECT id, name, start_date, end_date
             FROM workouts
             WHERE user_id = $1
             ORDER BY start_date DESC
             LIMIT $2",
            &[&self.user_id.as_i64(), &limit],
        )?;
        drop(client);

        rows.into_iter()
            .map(workout_header_from_row)
            .map(|result| {
                result.and_then(|(id, name, start_date, end_date)| {
                    self.build_workout(id, name, start_date, end_date)
                })
            })
            .collect()
    }

    fn delete(&self, id: &WorkoutId) -> Result<(), Self::RepoError> {
        let mut client = self.client()?;
        client.execute(
            "DELETE FROM workouts WHERE id = $1 AND user_id = $2",
            &[id.as_uuid(), &self.user_id.as_i64()],
        )?;
        Ok(())
    }

    fn update_name(&self, id: &WorkoutId, name: Option<&str>) -> Result<(), Self::RepoError> {
        let mut client = self.client()?;
        client.execute(
            "UPDATE workouts SET name = $3 WHERE id = $1 AND user_id = $2",
            &[id.as_uuid(), &self.user_id.as_i64(), &name],
        )?;
        Ok(())
    }

    fn remove_exercise(
        &self,
        workout_id: &WorkoutId,
        exercise_id: &ExerciseId,
    ) -> Result<(), Self::RepoError> {
        let mut client = self.client()?;
        let mut tx = client.transaction()?;
        let entry_order: Option<i32> = tx
            .query_opt(
                "SELECT entry_order
                 FROM workout_exercises
                 WHERE workout_id = $1 AND user_id = $2 AND exercise_id = $3",
                &[workout_id.as_uuid(), &self.user_id.as_i64(), exercise_id.as_uuid()],
            )?
            .map(|row| row.get("entry_order"));

        let Some(entry_order) = entry_order else {
            tx.commit()?;
            return Ok(());
        };

        tx.execute(
            "DELETE FROM workout_exercises
             WHERE workout_id = $1 AND user_id = $2 AND exercise_id = $3",
            &[workout_id.as_uuid(), &self.user_id.as_i64(), exercise_id.as_uuid()],
        )?;
        tx.execute(
            "UPDATE workout_exercises
             SET entry_order = entry_order - 1
             WHERE workout_id = $1 AND user_id = $2 AND entry_order > $3",
            &[workout_id.as_uuid(), &self.user_id.as_i64(), &entry_order],
        )?;
        tx.commit()?;
        Ok(())
    }

    fn remove_exercise_from_all(&self, exercise_id: &ExerciseId) -> Result<(), Self::RepoError> {
        let mut client = self.client()?;
        client.execute(
            "DELETE FROM workout_exercises WHERE exercise_id = $1 AND user_id = $2",
            &[exercise_id.as_uuid(), &self.user_id.as_i64()],
        )?;
        Ok(())
    }

    fn remove_set(
        &self,
        workout_id: &WorkoutId,
        exercise_id: &ExerciseId,
        set_index: usize,
    ) -> Result<(), Self::RepoError> {
        let set_order = to_i32(set_index, "set_order")?;
        let mut client = self.client()?;
        let mut tx = client.transaction()?;
        let changed = tx.execute(
            "DELETE FROM performed_sets
             WHERE workout_id = $1 AND user_id = $2 AND exercise_id = $3 AND set_order = $4",
            &[
                workout_id.as_uuid(),
                &self.user_id.as_i64(),
                exercise_id.as_uuid(),
                &set_order,
            ],
        )?;

        if changed > 0 {
            tx.execute(
                "UPDATE performed_sets
                 SET set_order = set_order - 1
                 WHERE workout_id = $1 AND user_id = $2 AND exercise_id = $3 AND set_order > $4",
                &[
                    workout_id.as_uuid(),
                    &self.user_id.as_i64(),
                    exercise_id.as_uuid(),
                    &set_order,
                ],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    fn get_dates_in_range(&self, from: Date, to: Date) -> Result<Vec<Date>, Self::RepoError> {
        let mut client = self.client()?;
        let rows = client.query(
            "SELECT DISTINCT start_date::date AS workout_date
             FROM workouts
             WHERE user_id = $1 AND start_date::date >= $2 AND start_date::date <= $3
             ORDER BY workout_date ASC",
            &[&self.user_id.as_i64(), &from, &to],
        )?;

        Ok(rows
            .into_iter()
            .map(|row| row.get("workout_date"))
            .collect())
    }
}

#[derive(Debug, Clone)]
struct StoredSet {
    reps: i32,
    load_type: PgLoadType,
    weight_value: Option<f64>,
    weight_units: Option<PgWeightUnits>,
}

impl StoredSet {
    fn from_domain(value: &PerformedSet) -> Result<Self, PostgresWorkoutRepoError> {
        let reps = i32::try_from(value.reps).map_err(|_| {
            PostgresWorkoutRepoError::ValueOutOfRange {
                field: "reps",
                value: value.reps as usize,
            }
        })?;

        let (load_type, weight_value, weight_units) = match &value.kind {
            LoadType::Weighted(weight) => (
                PgLoadType::Weighted,
                Some(weight.value),
                Some(PgWeightUnits::from(weight.units)),
            ),
            LoadType::BodyWeight => (PgLoadType::BodyWeight, None, None),
        };

        Ok(Self {
            reps,
            load_type,
            weight_value,
            weight_units,
        })
    }

    fn into_domain(self) -> Result<PerformedSet, PostgresWorkoutRepoError> {
        let reps = u32::try_from(self.reps)
            .map_err(|_| PostgresWorkoutRepoError::InvalidReps(self.reps))?;
        let kind = match self.load_type {
            PgLoadType::Weighted => LoadType::Weighted(Weight::new(
                self.weight_value
                    .ok_or(PostgresWorkoutRepoError::MissingWeightForWeightedSet)?,
                WeightUnits::from(
                    self.weight_units
                        .ok_or(PostgresWorkoutRepoError::MissingWeightUnitsForWeightedSet)?,
                ),
            )),
            PgLoadType::BodyWeight => LoadType::BodyWeight,
        };

        Ok(PerformedSet { reps, kind })
    }
}

fn stored_set_from_row(row: Row) -> Result<StoredSet, PostgresWorkoutRepoError> {
    Ok(StoredSet {
        reps: row.get("reps"),
        load_type: row.get("load_type"),
        weight_value: row.get("weight_value"),
        weight_units: row.get("weight_units"),
    })
}

fn workout_header_from_row(
    row: Row,
) -> Result<(WorkoutId, Option<String>, OffsetDateTime, Option<OffsetDateTime>), PostgresWorkoutRepoError>
{
    Ok((
        WorkoutId::from_uuid(row.get("id")),
        row.get("name"),
        row.get("start_date"),
        row.get("end_date"),
    ))
}

fn to_i32(value: usize, field: &'static str) -> Result<i32, PostgresWorkoutRepoError> {
    i32::try_from(value).map_err(|_| PostgresWorkoutRepoError::ValueOutOfRange { field, value })
}

fn to_i64(value: usize, field: &'static str) -> Result<i64, PostgresWorkoutRepoError> {
    i64::try_from(value).map_err(|_| PostgresWorkoutRepoError::ValueOutOfRange { field, value })
}

fn count_to_i32(value: i64, field: &'static str) -> Result<i32, PostgresWorkoutRepoError> {
    i32::try_from(value).map_err(|_| PostgresWorkoutRepoError::CountOutOfRange { field, value })
}

#[cfg(test)]
mod tests {
    use domain::{
        excercise::LoadType,
        types::{Weight, WeightUnits},
    };

    use crate::repos::postgres_types::{PgLoadType, PgWeightUnits};

    use super::StoredSet;

    #[test]
    fn stored_set_round_trips_weighted_loads() {
        let stored = StoredSet::from_domain(&domain::excercise::PerformedSet {
            reps: 8,
            kind: LoadType::Weighted(Weight::new(185.0, WeightUnits::Pounds)),
        })
        .expect("set should convert to storage");

        assert_eq!(stored.load_type, PgLoadType::Weighted);
        assert_eq!(stored.weight_units, Some(PgWeightUnits::Pounds));

        let restored = stored.into_domain().expect("set should convert from storage");
        match restored.kind {
            LoadType::Weighted(weight) => {
                assert_eq!(weight.value, 185.0);
                assert_eq!(weight.units, WeightUnits::Pounds);
            }
            LoadType::BodyWeight => panic!("expected weighted set"),
        }
        assert_eq!(restored.reps, 8);
    }

    #[test]
    fn stored_set_round_trips_bodyweight_loads() {
        let stored = StoredSet::from_domain(&domain::excercise::PerformedSet {
            reps: 15,
            kind: LoadType::BodyWeight,
        })
        .expect("set should convert to storage");

        assert_eq!(stored.load_type, PgLoadType::BodyWeight);
        assert_eq!(stored.weight_value, None);
        assert_eq!(stored.weight_units, None);

        let restored = stored.into_domain().expect("set should convert from storage");
        assert!(matches!(restored.kind, LoadType::BodyWeight));
        assert_eq!(restored.reps, 15);
    }
}
