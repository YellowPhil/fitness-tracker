use crate::types::{UserId, WorkoutId};

use super::types::{GenerationJob, GenerationJobListScope};

#[async_trait::async_trait]
pub trait GenerationDispatcher: Send + Sync {
    async fn dispatch(&self, job: GenerationJob) -> anyhow::Result<()>;
}

#[async_trait::async_trait]
pub trait GenerationJobRepo: Send + Sync {
    type RepoError: std::error::Error + Send + Sync;

    async fn create_or_reuse_active_job(
        &self,
        user_id: UserId,
        date: time::Date,
        request_fingerprint: &str,
        request_payload: &serde_json::Value,
    ) -> Result<(GenerationJob, bool), Self::RepoError>;

    async fn get_job(
        &self,
        user_id: UserId,
        id: uuid::Uuid,
    ) -> Result<Option<GenerationJob>, Self::RepoError>;

    async fn list_jobs(
        &self,
        user_id: UserId,
        limit: i64,
        scope: GenerationJobListScope,
    ) -> Result<Vec<GenerationJob>, Self::RepoError>;

    async fn mark_running(
        &self,
        user_id: UserId,
        id: uuid::Uuid,
    ) -> Result<Option<GenerationJob>, Self::RepoError>;

    async fn mark_completed(
        &self,
        user_id: UserId,
        id: uuid::Uuid,
        workout_id: WorkoutId,
    ) -> Result<GenerationJob, Self::RepoError>;

    async fn mark_failed(
        &self,
        user_id: UserId,
        id: uuid::Uuid,
        error_message: &str,
    ) -> Result<GenerationJob, Self::RepoError>;
}
