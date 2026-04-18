use domain::generation::GenerationJobRepo;
pub use domain::generation::{GenerationJob, GenerationJobListScope, GenerationJobStatus};
use domain::types::{UserId, WorkoutId};
use serde_json::Value;
use sqlx::{Pool, Postgres, Row};
use time::Date;
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum PostgresGenerationJobRepoError {
    #[error("postgres error: {0}")]
    Postgres(#[from] sqlx::Error),
    #[error("invalid generation_jobs.status value '{0}'")]
    InvalidStatus(String),
    #[error("job not found")]
    NotFound,
}

#[derive(Clone)]
pub struct PostgresGenerationJobDb {
    pool: Pool<Postgres>,
}

impl PostgresGenerationJobDb {
    pub(crate) fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    pub fn for_user(&self, user_id: UserId) -> PostgresGenerationJobRepo {
        PostgresGenerationJobRepo {
            pool: self.pool.clone(),
            user_id,
        }
    }
}

pub struct PostgresGenerationJobRepo {
    pool: Pool<Postgres>,
    user_id: UserId,
}

#[async_trait::async_trait]
impl GenerationJobRepo for PostgresGenerationJobRepo {
    type RepoError = PostgresGenerationJobRepoError;

    async fn create_or_reuse_active_job(
        &self,
        user_id: UserId,
        date: Date,
        request_fingerprint: &str,
        request_payload: &Value,
    ) -> Result<(GenerationJob, bool), Self::RepoError> {
        debug_assert_eq!(self.user_id, user_id);
        PostgresGenerationJobRepo::create_or_reuse_active_job(
            self,
            date,
            request_fingerprint,
            request_payload,
        )
        .await
    }

    async fn get_job(
        &self,
        user_id: UserId,
        id: Uuid,
    ) -> Result<Option<GenerationJob>, Self::RepoError> {
        debug_assert_eq!(self.user_id, user_id);
        PostgresGenerationJobRepo::get_job(self, id).await
    }

    async fn list_jobs(
        &self,
        user_id: UserId,
        limit: i64,
        scope: GenerationJobListScope,
    ) -> Result<Vec<GenerationJob>, Self::RepoError> {
        debug_assert_eq!(self.user_id, user_id);
        PostgresGenerationJobRepo::list_jobs(self, limit, scope).await
    }

    async fn mark_running(
        &self,
        user_id: UserId,
        id: Uuid,
    ) -> Result<Option<GenerationJob>, Self::RepoError> {
        debug_assert_eq!(self.user_id, user_id);
        PostgresGenerationJobRepo::mark_running(self, id).await
    }

    async fn mark_completed(
        &self,
        user_id: UserId,
        id: Uuid,
        workout_id: WorkoutId,
    ) -> Result<GenerationJob, Self::RepoError> {
        debug_assert_eq!(self.user_id, user_id);
        PostgresGenerationJobRepo::mark_completed(self, id, workout_id).await
    }

    async fn mark_failed(
        &self,
        user_id: UserId,
        id: Uuid,
        error_message: &str,
    ) -> Result<GenerationJob, Self::RepoError> {
        debug_assert_eq!(self.user_id, user_id);
        PostgresGenerationJobRepo::mark_failed(self, id, error_message).await
    }
}

impl PostgresGenerationJobRepo {
    pub async fn create_or_reuse_active_job(
        &self,
        date: Date,
        request_fingerprint: &str,
        request_payload: &Value,
    ) -> Result<(GenerationJob, bool), PostgresGenerationJobRepoError> {
        if let Some(existing) = self.find_active_job(date, request_fingerprint).await? {
            return Ok((existing, true));
        }

        let created = self
            .insert_job(date, request_fingerprint, request_payload)
            .await;

        match created {
            Ok(job) => Ok((job, false)),
            Err(PostgresGenerationJobRepoError::Postgres(sqlx::Error::Database(db_err)))
                if db_err.is_unique_violation() =>
            {
                let existing = self
                    .find_active_job(date, request_fingerprint)
                    .await?
                    .ok_or(PostgresGenerationJobRepoError::NotFound)?;
                Ok((existing, true))
            }
            Err(err) => Err(err),
        }
    }

    pub async fn get_job(
        &self,
        id: Uuid,
    ) -> Result<Option<GenerationJob>, PostgresGenerationJobRepoError> {
        let row = sqlx::query(
            "SELECT
                id,
                user_id,
                date,
                status,
                request_fingerprint,
                request_payload,
                workout_id,
                error,
                version,
                created_at,
                updated_at,
                queued_at,
                started_at,
                completed_at,
                failed_at
             FROM generation_jobs
             WHERE id = $1 AND user_id = $2",
        )
        .bind(id)
        .bind(self.user_id.as_i64())
        .fetch_optional(&self.pool)
        .await?;

        row.map(job_from_row).transpose()
    }

    pub async fn list_jobs(
        &self,
        limit: i64,
        scope: GenerationJobListScope,
    ) -> Result<Vec<GenerationJob>, PostgresGenerationJobRepoError> {
        let limit = limit.clamp(1, 100);
        let rows = match scope {
            GenerationJobListScope::All => {
                sqlx::query(
                    "SELECT
                        id,
                        user_id,
                        date,
                        status,
                        request_fingerprint,
                        request_payload,
                        workout_id,
                        error,
                        version,
                        created_at,
                        updated_at,
                        queued_at,
                        started_at,
                        completed_at,
                        failed_at
                     FROM generation_jobs
                     WHERE user_id = $1
                     ORDER BY created_at DESC
                     LIMIT $2",
                )
                .bind(self.user_id.as_i64())
                .bind(limit)
                .fetch_all(&self.pool)
                .await?
            }
            GenerationJobListScope::Active => {
                sqlx::query(
                    "SELECT
                        id,
                        user_id,
                        date,
                        status,
                        request_fingerprint,
                        request_payload,
                        workout_id,
                        error,
                        version,
                        created_at,
                        updated_at,
                        queued_at,
                        started_at,
                        completed_at,
                        failed_at
                     FROM generation_jobs
                     WHERE user_id = $1
                       AND status IN ('queued', 'running')
                     ORDER BY created_at DESC
                     LIMIT $2",
                )
                .bind(self.user_id.as_i64())
                .bind(limit)
                .fetch_all(&self.pool)
                .await?
            }
        };

        rows.into_iter().map(job_from_row).collect()
    }

    pub async fn mark_running(
        &self,
        id: Uuid,
    ) -> Result<Option<GenerationJob>, PostgresGenerationJobRepoError> {
        let row = sqlx::query(
            "UPDATE generation_jobs
             SET
                status = 'running',
                started_at = COALESCE(started_at, now()),
                updated_at = now(),
                version = version + 1
             WHERE id = $1
               AND user_id = $2
               AND status = 'queued'
             RETURNING
                id,
                user_id,
                date,
                status,
                request_fingerprint,
                request_payload,
                workout_id,
                error,
                version,
                created_at,
                updated_at,
                queued_at,
                started_at,
                completed_at,
                failed_at",
        )
        .bind(id)
        .bind(self.user_id.as_i64())
        .fetch_optional(&self.pool)
        .await?;

        row.map(job_from_row).transpose()
    }

    pub async fn mark_completed(
        &self,
        id: Uuid,
        workout_id: WorkoutId,
    ) -> Result<GenerationJob, PostgresGenerationJobRepoError> {
        let row = sqlx::query(
            "UPDATE generation_jobs
             SET
                status = 'completed',
                workout_id = $3,
                error = NULL,
                completed_at = COALESCE(completed_at, now()),
                updated_at = now(),
                version = version + 1
             WHERE id = $1
               AND user_id = $2
               AND status IN ('queued', 'running')
             RETURNING
                id,
                user_id,
                date,
                status,
                request_fingerprint,
                request_payload,
                workout_id,
                error,
                version,
                created_at,
                updated_at,
                queued_at,
                started_at,
                completed_at,
                failed_at",
        )
        .bind(id)
        .bind(self.user_id.as_i64())
        .bind(workout_id.as_uuid())
        .fetch_optional(&self.pool)
        .await?
        .ok_or(PostgresGenerationJobRepoError::NotFound)?;

        job_from_row(row)
    }

    pub async fn mark_failed(
        &self,
        id: Uuid,
        error_message: &str,
    ) -> Result<GenerationJob, PostgresGenerationJobRepoError> {
        let row = sqlx::query(
            "UPDATE generation_jobs
             SET
                status = 'failed',
                error = $3,
                failed_at = COALESCE(failed_at, now()),
                updated_at = now(),
                version = version + 1
             WHERE id = $1
               AND user_id = $2
               AND status IN ('queued', 'running')
             RETURNING
                id,
                user_id,
                date,
                status,
                request_fingerprint,
                request_payload,
                workout_id,
                error,
                version,
                created_at,
                updated_at,
                queued_at,
                started_at,
                completed_at,
                failed_at",
        )
        .bind(id)
        .bind(self.user_id.as_i64())
        .bind(error_message)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(PostgresGenerationJobRepoError::NotFound)?;

        job_from_row(row)
    }

    async fn find_active_job(
        &self,
        date: Date,
        request_fingerprint: &str,
    ) -> Result<Option<GenerationJob>, PostgresGenerationJobRepoError> {
        let row = sqlx::query(
            "SELECT
                id,
                user_id,
                date,
                status,
                request_fingerprint,
                request_payload,
                workout_id,
                error,
                version,
                created_at,
                updated_at,
                queued_at,
                started_at,
                completed_at,
                failed_at
             FROM generation_jobs
             WHERE user_id = $1
               AND date = $2
               AND request_fingerprint = $3
               AND status IN ('queued', 'running')
             ORDER BY created_at DESC
             LIMIT 1",
        )
        .bind(self.user_id.as_i64())
        .bind(date)
        .bind(request_fingerprint)
        .fetch_optional(&self.pool)
        .await?;

        row.map(job_from_row).transpose()
    }

    async fn insert_job(
        &self,
        date: Date,
        request_fingerprint: &str,
        request_payload: &Value,
    ) -> Result<GenerationJob, PostgresGenerationJobRepoError> {
        let row = sqlx::query(
            "INSERT INTO generation_jobs (
                id,
                user_id,
                date,
                status,
                request_fingerprint,
                request_payload,
                version,
                queued_at,
                created_at,
                updated_at
             )
             VALUES ($1, $2, $3, 'queued', $4, $5, 1, now(), now(), now())
             RETURNING
                id,
                user_id,
                date,
                status,
                request_fingerprint,
                request_payload,
                workout_id,
                error,
                version,
                created_at,
                updated_at,
                queued_at,
                started_at,
                completed_at,
                failed_at",
        )
        .bind(Uuid::new_v4())
        .bind(self.user_id.as_i64())
        .bind(date)
        .bind(request_fingerprint)
        .bind(request_payload)
        .fetch_one(&self.pool)
        .await?;

        job_from_row(row)
    }
}

fn job_from_row(
    row: sqlx::postgres::PgRow,
) -> Result<GenerationJob, PostgresGenerationJobRepoError> {
    let status: String = row.get("status");
    Ok(GenerationJob {
        id: row.get("id"),
        user_id: UserId::new(row.get("user_id")),
        date: row.get("date"),
        status: GenerationJobStatus::parse_api_str(status.as_str())
            .ok_or(PostgresGenerationJobRepoError::InvalidStatus(status))?,
        request_fingerprint: row.get("request_fingerprint"),
        request_payload: row.get("request_payload"),
        workout_id: row
            .get::<Option<Uuid>, _>("workout_id")
            .map(WorkoutId::from_uuid),
        error: row.get("error"),
        version: row.get("version"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        queued_at: row.get("queued_at"),
        started_at: row.get("started_at"),
        completed_at: row.get("completed_at"),
        failed_at: row.get("failed_at"),
    })
}
