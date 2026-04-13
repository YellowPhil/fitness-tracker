use domain::{
    preferences::{TrainingGoal, WorkoutPreferences, WorkoutSplit},
    traits::PreferencesRepo,
    types::UserId,
};
use sqlx::{Pool, Postgres, Row, postgres::PgRow};
use tracing::instrument;

#[derive(Debug, thiserror::Error)]
pub enum PostgresPreferencesRepoError {
    #[error("postgres error: {0}")]
    Postgres(#[from] sqlx::Error),
    #[error("invalid value for {field}: {value}")]
    InvalidEnumValue { field: &'static str, value: String },
    #[error("invalid numeric value for {field}: {value}")]
    InvalidNumericValue { field: &'static str, value: i32 },
}

pub struct PostgresPreferencesDb {
    pool: Pool<Postgres>,
}

impl PostgresPreferencesDb {
    pub(crate) fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    pub fn for_user(&self, user_id: UserId) -> PostgresPreferencesRepo {
        PostgresPreferencesRepo {
            pool: self.pool.clone(),
            user_id,
        }
    }
}

pub struct PostgresPreferencesRepo {
    pool: Pool<Postgres>,
    user_id: UserId,
}

#[async_trait::async_trait]
impl PreferencesRepo for PostgresPreferencesRepo {
    type RepoError = PostgresPreferencesRepoError;

    #[instrument(skip(self), fields(table = "workout_preferences"), err)]
    async fn get_preferences(&self) -> Result<WorkoutPreferences, Self::RepoError> {
        let row = sqlx::query(
            "SELECT
                max_sets_per_exercise,
                preferred_split,
                training_goal,
                session_duration_minutes,
                notes
             FROM workout_preferences
             WHERE user_id = $1",
        )
        .bind(self.user_id.as_i64())
        .fetch_optional(&self.pool)
        .await?;

        row.map(preferences_from_row)
            .transpose()
            .map(|item| item.unwrap_or_default())
    }

    #[instrument(skip(self, preferences), fields(table = "workout_preferences"), err)]
    async fn save(&self, preferences: &WorkoutPreferences) -> Result<(), Self::RepoError> {
        sqlx::query(
            "INSERT INTO workout_preferences (
                user_id,
                max_sets_per_exercise,
                preferred_split,
                training_goal,
                session_duration_minutes,
                notes
             )
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (user_id) DO UPDATE SET
                max_sets_per_exercise = EXCLUDED.max_sets_per_exercise,
                preferred_split = EXCLUDED.preferred_split,
                training_goal = EXCLUDED.training_goal,
                session_duration_minutes = EXCLUDED.session_duration_minutes,
                notes = EXCLUDED.notes",
        )
        .bind(self.user_id.as_i64())
        .bind(preferences.max_sets_per_exercise.map(i16::from))
        .bind(preferences.preferred_split.map(WorkoutSplit::as_api_str))
        .bind(preferences.training_goal.map(TrainingGoal::as_api_str))
        .bind(preferences.session_duration_minutes.map(i32::from))
        .bind(preferences.notes.as_deref())
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

fn preferences_from_row(row: PgRow) -> Result<WorkoutPreferences, PostgresPreferencesRepoError> {
    let max_sets_per_exercise = row
        .get::<Option<i16>, _>("max_sets_per_exercise")
        .map(|value| {
            u8::try_from(value).map_err(|_| PostgresPreferencesRepoError::InvalidNumericValue {
                field: "max_sets_per_exercise",
                value: i32::from(value),
            })
        })
        .transpose()?;

    let preferred_split = row
        .get::<Option<String>, _>("preferred_split")
        .map(|value| {
            WorkoutSplit::parse_api_str(&value).ok_or_else(|| {
                PostgresPreferencesRepoError::InvalidEnumValue {
                    field: "preferred_split",
                    value,
                }
            })
        })
        .transpose()?;

    let training_goal = row
        .get::<Option<String>, _>("training_goal")
        .map(|value| {
            TrainingGoal::parse_api_str(&value).ok_or_else(|| {
                PostgresPreferencesRepoError::InvalidEnumValue {
                    field: "training_goal",
                    value,
                }
            })
        })
        .transpose()?;

    let session_duration_minutes = row
        .get::<Option<i32>, _>("session_duration_minutes")
        .map(|value| {
            u16::try_from(value).map_err(|_| PostgresPreferencesRepoError::InvalidNumericValue {
                field: "session_duration_minutes",
                value,
            })
        })
        .transpose()?;

    Ok(WorkoutPreferences {
        max_sets_per_exercise,
        preferred_split,
        training_goal,
        session_duration_minutes,
        notes: row.get("notes"),
    })
}
