use domain::{
    traits::WorkoutRepo,
    types::{
        ExerciseId, LoadType, PerformedSet, Workout, WorkoutExercise, WorkoutId, WorkoutSource,
    },
    types::{UserId, Weight, WeightUnits},
};
use sqlx::{Pool, Postgres, Row, postgres::PgRow};
use time::{Date, OffsetDateTime};
use tracing::instrument;

use super::postgres_types::{PgLoadType, PgWeightUnits, PgWorkoutSource};

#[derive(Debug, thiserror::Error)]
pub enum PostgresWorkoutRepoError {
    #[error("postgres error: {0}")]
    Postgres(#[from] sqlx::Error),
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
    pool: Pool<Postgres>,
}

impl PostgresWorkoutDb {
    pub(crate) fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    pub fn for_user(&self, user_id: UserId) -> PostgresWorkoutRepo {
        PostgresWorkoutRepo {
            pool: self.pool.clone(),
            user_id,
        }
    }
}

pub struct PostgresWorkoutRepo {
    pool: Pool<Postgres>,
    user_id: UserId,
}

impl PostgresWorkoutRepo {
    #[instrument(level = "debug", skip(self, name, start_date, end_date, source), fields(workout_id = ?id), err)]
    async fn build_workout(
        &self,
        id: WorkoutId,
        name: Option<String>,
        start_date: OffsetDateTime,
        end_date: Option<OffsetDateTime>,
        source: WorkoutSource,
    ) -> Result<Workout, PostgresWorkoutRepoError> {
        Ok(Workout {
            entries: self.load_workout_entries(&id).await?,
            id,
            name,
            start_date,
            end_date,
            source,
        })
    }

    #[instrument(level = "debug", skip(self), fields(workout_id = ?workout_id), err)]
    async fn load_workout_entries(
        &self,
        workout_id: &WorkoutId,
    ) -> Result<Vec<WorkoutExercise>, PostgresWorkoutRepoError> {
        let rows = sqlx::query(
            "SELECT exercise_id, notes
             FROM workout_exercises
             WHERE workout_id = $1 AND user_id = $2
             ORDER BY entry_order ASC",
        )
        .bind(workout_id.as_uuid())
        .bind(self.user_id.as_i64())
        .fetch_all(&self.pool)
        .await?;

        let mut entries = Vec::with_capacity(rows.len());
        for row in rows {
            let exercise_id = ExerciseId::from_uuid(row.get("exercise_id"));
            let sets = self.load_performed_sets(workout_id, &exercise_id).await?;
            entries.push(WorkoutExercise {
                sets,
                exercise_id,
                notes: row.get("notes"),
            });
        }
        Ok(entries)
    }

    #[instrument(level = "debug", skip(self), fields(workout_id = ?workout_id, exercise_id = ?exercise_id), err)]
    async fn load_performed_sets(
        &self,
        workout_id: &WorkoutId,
        exercise_id: &ExerciseId,
    ) -> Result<Vec<PerformedSet>, PostgresWorkoutRepoError> {
        let rows = sqlx::query(
            "SELECT reps, load_type, weight_value, weight_units
             FROM performed_sets
             WHERE workout_id = $1 AND user_id = $2 AND exercise_id = $3
             ORDER BY set_order ASC",
        )
        .bind(workout_id.as_uuid())
        .bind(self.user_id.as_i64())
        .bind(exercise_id.as_uuid())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| stored_set_from_row(row).into_domain())
            .collect()
    }
}

#[async_trait::async_trait]
impl WorkoutRepo for PostgresWorkoutRepo {
    type RepoError = PostgresWorkoutRepoError;

    #[instrument(skip(self), fields(table = "workouts"), err)]
    async fn get_all(&self) -> Result<Vec<Workout>, Self::RepoError> {
        let rows = sqlx::query(
            "SELECT id, name, start_date, end_date, source
             FROM workouts
             WHERE user_id = $1
             ORDER BY start_date DESC",
        )
        .bind(self.user_id.as_i64())
        .fetch_all(&self.pool)
        .await?;

        let mut workouts = Vec::with_capacity(rows.len());
        for row in rows {
            let (id, name, start_date, end_date, source) = workout_header_from_row(row);
            workouts.push(
                self.build_workout(id, name, start_date, end_date, source)
                    .await?,
            );
        }
        Ok(workouts)
    }

    #[instrument(skip(self), fields(table = "workouts", workout_id = ?id), err)]
    async fn get_by_id(&self, id: &WorkoutId) -> Result<Option<Workout>, Self::RepoError> {
        let row = sqlx::query(
            "SELECT id, name, start_date, end_date, source
             FROM workouts
             WHERE id = $1 AND user_id = $2",
        )
        .bind(id.as_uuid())
        .bind(self.user_id.as_i64())
        .fetch_optional(&self.pool)
        .await?;

        match row {
            None => Ok(None),
            Some(r) => {
                let (id, name, start_date, end_date, source) = workout_header_from_row(r);
                Ok(Some(
                    self.build_workout(id, name, start_date, end_date, source)
                        .await?,
                ))
            }
        }
    }

    #[instrument(skip(self, workout), fields(table = "workouts", workout_id = ?workout.id, entry_count = workout.entries.len()), err)]
    async fn save(&self, workout: &Workout) -> Result<(), Self::RepoError> {
        let mut tx = self.pool.begin().await?;
        let pg_source = PgWorkoutSource::from(workout.source);

        sqlx::query(
            "INSERT INTO workouts (id, user_id, name, start_date, end_date, source)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (id, user_id) DO UPDATE SET
                name = EXCLUDED.name,
                start_date = EXCLUDED.start_date,
                end_date = EXCLUDED.end_date,
                source = EXCLUDED.source",
        )
        .bind(workout.id.as_uuid())
        .bind(self.user_id.as_i64())
        .bind(&workout.name)
        .bind(workout.start_date)
        .bind(workout.end_date)
        .bind(pg_source)
        .execute(&mut *tx)
        .await?;

        sqlx::query("DELETE FROM performed_sets WHERE workout_id = $1 AND user_id = $2")
            .bind(workout.id.as_uuid())
            .bind(self.user_id.as_i64())
            .execute(&mut *tx)
            .await?;

        sqlx::query("DELETE FROM workout_exercises WHERE workout_id = $1 AND user_id = $2")
            .bind(workout.id.as_uuid())
            .bind(self.user_id.as_i64())
            .execute(&mut *tx)
            .await?;

        for (entry_order, entry) in workout.entries.iter().enumerate() {
            let entry_order = to_i32(entry_order, "entry_order")?;
            sqlx::query(
                "INSERT INTO workout_exercises (workout_id, user_id, exercise_id, entry_order, notes)
                 VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(workout.id.as_uuid())
            .bind(self.user_id.as_i64())
            .bind(entry.exercise_id.as_uuid())
            .bind(entry_order)
            .bind(&entry.notes)
            .execute(&mut *tx)
            .await?;

            for (set_order, set) in entry.sets.iter().enumerate() {
                let stored = StoredSet::from_domain(set)?;
                let set_order = to_i32(set_order, "set_order")?;
                sqlx::query(
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
                )
                .bind(workout.id.as_uuid())
                .bind(self.user_id.as_i64())
                .bind(entry.exercise_id.as_uuid())
                .bind(set_order)
                .bind(stored.reps)
                .bind(stored.load_type)
                .bind(stored.weight_value)
                .bind(stored.weight_units)
                .execute(&mut *tx)
                .await?;
            }
        }

        tx.commit().await?;
        Ok(())
    }

    #[instrument(skip(self, exercise), fields(table = "workouts", workout_id = ?workout_id), err)]
    async fn add_exercise(
        &self,
        workout_id: &WorkoutId,
        exercise: &WorkoutExercise,
    ) -> Result<(), Self::RepoError> {
        let mut tx = self.pool.begin().await?;

        let entry_order: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM workout_exercises WHERE workout_id = $1 AND user_id = $2",
        )
        .bind(workout_id.as_uuid())
        .bind(self.user_id.as_i64())
        .fetch_one(&mut *tx)
        .await?;
        let entry_order = count_to_i32(entry_order, "entry_order")?;

        sqlx::query(
            "INSERT INTO workout_exercises (workout_id, user_id, exercise_id, entry_order, notes)
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(workout_id.as_uuid())
        .bind(self.user_id.as_i64())
        .bind(exercise.exercise_id.as_uuid())
        .bind(entry_order)
        .bind(&exercise.notes)
        .execute(&mut *tx)
        .await?;

        for (set_order, set) in exercise.sets.iter().enumerate() {
            let stored = StoredSet::from_domain(set)?;
            let set_order = to_i32(set_order, "set_order")?;
            sqlx::query(
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
            )
            .bind(workout_id.as_uuid())
            .bind(self.user_id.as_i64())
            .bind(exercise.exercise_id.as_uuid())
            .bind(set_order)
            .bind(stored.reps)
            .bind(stored.load_type)
            .bind(stored.weight_value)
            .bind(stored.weight_units)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    #[instrument(skip(self, set), fields(table = "performed_sets", workout_id = ?workout_id, exercise_id = ?exercise_id), err)]
    async fn add_set(
        &self,
        workout_id: &WorkoutId,
        exercise_id: &ExerciseId,
        set: &PerformedSet,
    ) -> Result<(), Self::RepoError> {
        let stored = StoredSet::from_domain(set)?;

        let set_order: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM performed_sets
             WHERE workout_id = $1 AND user_id = $2 AND exercise_id = $3",
        )
        .bind(workout_id.as_uuid())
        .bind(self.user_id.as_i64())
        .bind(exercise_id.as_uuid())
        .fetch_one(&self.pool)
        .await?;
        let set_order = count_to_i32(set_order, "set_order")?;

        sqlx::query(
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
        )
        .bind(workout_id.as_uuid())
        .bind(self.user_id.as_i64())
        .bind(exercise_id.as_uuid())
        .bind(set_order)
        .bind(stored.reps)
        .bind(stored.load_type)
        .bind(stored.weight_value)
        .bind(stored.weight_units)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    #[instrument(skip(self), fields(table = "workouts", date = %date), err)]
    async fn get_by_date(&self, date: Date) -> Result<Vec<Workout>, Self::RepoError> {
        let rows = sqlx::query(
            "SELECT id, name, start_date, end_date, source
             FROM workouts
             WHERE user_id = $1 AND start_date::date = $2
             ORDER BY start_date DESC",
        )
        .bind(self.user_id.as_i64())
        .bind(date)
        .fetch_all(&self.pool)
        .await?;

        let mut workouts = Vec::with_capacity(rows.len());
        for row in rows {
            let (id, name, start_date, end_date, source) = workout_header_from_row(row);
            workouts.push(
                self.build_workout(id, name, start_date, end_date, source)
                    .await?,
            );
        }
        Ok(workouts)
    }

    #[instrument(skip(self), fields(table = "workouts"), err)]
    async fn get_latest(&self) -> Result<Option<Workout>, Self::RepoError> {
        let row = sqlx::query(
            "SELECT id, name, start_date, end_date, source
             FROM workouts
             WHERE user_id = $1
             ORDER BY start_date DESC
             LIMIT 1",
        )
        .bind(self.user_id.as_i64())
        .fetch_optional(&self.pool)
        .await?;

        match row {
            None => Ok(None),
            Some(r) => {
                let (id, name, start_date, end_date, source) = workout_header_from_row(r);
                Ok(Some(
                    self.build_workout(id, name, start_date, end_date, source)
                        .await?,
                ))
            }
        }
    }

    #[instrument(skip(self), fields(table = "workouts", n = n), err)]
    async fn get_last_n(&self, n: usize) -> Result<Vec<Workout>, Self::RepoError> {
        let limit = to_i64(n, "limit")?;
        let rows = sqlx::query(
            "SELECT id, name, start_date, end_date, source
             FROM workouts
             WHERE user_id = $1
             ORDER BY start_date DESC
             LIMIT $2",
        )
        .bind(self.user_id.as_i64())
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut workouts = Vec::with_capacity(rows.len());
        for row in rows {
            let (id, name, start_date, end_date, source) = workout_header_from_row(row);
            workouts.push(
                self.build_workout(id, name, start_date, end_date, source)
                    .await?,
            );
        }
        Ok(workouts)
    }

    #[instrument(skip(self), fields(table = "workouts", workout_id = ?id), err)]
    async fn delete(&self, id: &WorkoutId) -> Result<(), Self::RepoError> {
        sqlx::query("DELETE FROM workouts WHERE id = $1 AND user_id = $2")
            .bind(id.as_uuid())
            .bind(self.user_id.as_i64())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    #[instrument(skip(self, name), fields(table = "workouts", workout_id = ?id), err)]
    async fn update_name(&self, id: &WorkoutId, name: Option<&str>) -> Result<(), Self::RepoError> {
        sqlx::query("UPDATE workouts SET name = $3 WHERE id = $1 AND user_id = $2")
            .bind(id.as_uuid())
            .bind(self.user_id.as_i64())
            .bind(name)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    #[instrument(skip(self), fields(table = "workouts", workout_id = ?workout_id, exercise_id = ?exercise_id), err)]
    async fn remove_exercise(
        &self,
        workout_id: &WorkoutId,
        exercise_id: &ExerciseId,
    ) -> Result<(), Self::RepoError> {
        let mut tx = self.pool.begin().await?;

        let entry_order: Option<i32> = sqlx::query_scalar(
            "SELECT entry_order
             FROM workout_exercises
             WHERE workout_id = $1 AND user_id = $2 AND exercise_id = $3",
        )
        .bind(workout_id.as_uuid())
        .bind(self.user_id.as_i64())
        .bind(exercise_id.as_uuid())
        .fetch_optional(&mut *tx)
        .await?;

        let Some(entry_order) = entry_order else {
            tx.commit().await?;
            return Ok(());
        };

        sqlx::query(
            "DELETE FROM workout_exercises
             WHERE workout_id = $1 AND user_id = $2 AND exercise_id = $3",
        )
        .bind(workout_id.as_uuid())
        .bind(self.user_id.as_i64())
        .bind(exercise_id.as_uuid())
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            "UPDATE workout_exercises
             SET entry_order = entry_order - 1
             WHERE workout_id = $1 AND user_id = $2 AND entry_order > $3",
        )
        .bind(workout_id.as_uuid())
        .bind(self.user_id.as_i64())
        .bind(entry_order)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    #[instrument(skip(self), fields(table = "workout_exercises", exercise_id = ?exercise_id), err)]
    async fn remove_exercise_from_all(
        &self,
        exercise_id: &ExerciseId,
    ) -> Result<(), Self::RepoError> {
        sqlx::query("DELETE FROM workout_exercises WHERE exercise_id = $1 AND user_id = $2")
            .bind(exercise_id.as_uuid())
            .bind(self.user_id.as_i64())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    #[instrument(skip(self, set), fields(table = "performed_sets", workout_id = ?workout_id, exercise_id = ?exercise_id, set_index = set_index), err)]
    async fn update_set(
        &self,
        workout_id: &WorkoutId,
        exercise_id: &ExerciseId,
        set_index: usize,
        set: &PerformedSet,
    ) -> Result<(), Self::RepoError> {
        let set_order = to_i32(set_index, "set_order")?;
        let stored = StoredSet::from_domain(set)?;

        sqlx::query(
            "UPDATE performed_sets
             SET reps = $5, load_type = $6, weight_value = $7, weight_units = $8
             WHERE workout_id = $1 AND user_id = $2 AND exercise_id = $3 AND set_order = $4",
        )
        .bind(workout_id.as_uuid())
        .bind(self.user_id.as_i64())
        .bind(exercise_id.as_uuid())
        .bind(set_order)
        .bind(stored.reps)
        .bind(stored.load_type)
        .bind(stored.weight_value)
        .bind(stored.weight_units)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    #[instrument(skip(self), fields(table = "performed_sets", workout_id = ?workout_id, exercise_id = ?exercise_id, set_index = set_index), err)]
    async fn remove_set(
        &self,
        workout_id: &WorkoutId,
        exercise_id: &ExerciseId,
        set_index: usize,
    ) -> Result<(), Self::RepoError> {
        let set_order = to_i32(set_index, "set_order")?;
        let mut tx = self.pool.begin().await?;

        let result = sqlx::query(
            "DELETE FROM performed_sets
             WHERE workout_id = $1 AND user_id = $2 AND exercise_id = $3 AND set_order = $4",
        )
        .bind(workout_id.as_uuid())
        .bind(self.user_id.as_i64())
        .bind(exercise_id.as_uuid())
        .bind(set_order)
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() > 0 {
            sqlx::query(
                "UPDATE performed_sets
                 SET set_order = set_order - 1
                 WHERE workout_id = $1 AND user_id = $2 AND exercise_id = $3 AND set_order > $4",
            )
            .bind(workout_id.as_uuid())
            .bind(self.user_id.as_i64())
            .bind(exercise_id.as_uuid())
            .bind(set_order)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    #[instrument(skip(self), fields(table = "workouts", from = %from, to = %to), err)]
    async fn get_dates_in_range(&self, from: Date, to: Date) -> Result<Vec<Date>, Self::RepoError> {
        let rows = sqlx::query(
            "SELECT DISTINCT start_date::date AS workout_date
             FROM workouts
             WHERE user_id = $1 AND start_date::date >= $2 AND start_date::date <= $3
             ORDER BY workout_date ASC",
        )
        .bind(self.user_id.as_i64())
        .bind(from)
        .bind(to)
        .fetch_all(&self.pool)
        .await?;

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
        let reps =
            i32::try_from(value.reps).map_err(|_| PostgresWorkoutRepoError::ValueOutOfRange {
                field: "reps",
                value: value.reps as usize,
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

fn stored_set_from_row(row: PgRow) -> StoredSet {
    StoredSet {
        reps: row.get("reps"),
        load_type: row.get("load_type"),
        weight_value: row.get("weight_value"),
        weight_units: row.get("weight_units"),
    }
}

fn workout_header_from_row(
    row: PgRow,
) -> (
    WorkoutId,
    Option<String>,
    OffsetDateTime,
    Option<OffsetDateTime>,
    WorkoutSource,
) {
    let pg_source: PgWorkoutSource = row.get("source");
    (
        WorkoutId::from_uuid(row.get("id")),
        row.get("name"),
        row.get("start_date"),
        row.get("end_date"),
        pg_source.into(),
    )
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
        types::LoadType,
        types::{Weight, WeightUnits},
    };

    use crate::repos::postgres_types::{PgLoadType, PgWeightUnits};

    use super::StoredSet;

    #[test]
    fn stored_set_round_trips_weighted_loads() {
        let stored = StoredSet::from_domain(&domain::types::PerformedSet {
            reps: 8,
            kind: LoadType::Weighted(Weight::new(185.0, WeightUnits::Pounds)),
        })
        .expect("set should convert to storage");

        assert_eq!(stored.load_type, PgLoadType::Weighted);
        assert_eq!(stored.weight_units, Some(PgWeightUnits::Pounds));

        let restored = stored
            .into_domain()
            .expect("set should convert from storage");
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
        let stored = StoredSet::from_domain(&domain::types::PerformedSet {
            reps: 15,
            kind: LoadType::BodyWeight,
        })
        .expect("set should convert to storage");

        assert_eq!(stored.load_type, PgLoadType::BodyWeight);
        assert_eq!(stored.weight_value, None);
        assert_eq!(stored.weight_units, None);

        let restored = stored
            .into_domain()
            .expect("set should convert from storage");
        assert!(matches!(restored.kind, LoadType::BodyWeight));
        assert_eq!(restored.reps, 15);
    }
}
