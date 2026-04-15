pub mod dispatcher;
pub mod event_bus;

use std::sync::Arc;

use anyhow::Context;
use domain::types::{MuscleGroup, UserId};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use time::OffsetDateTime;

use crate::repos::generation_jobs::{
    GenerationJob, GenerationJobListScope, PostgresGenerationJobDb, PostgresGenerationJobRepoError,
};

use dispatcher::GenerationDispatcher;

#[derive(Debug, Clone)]
pub struct GenerationRequest {
    pub muscle_groups: Vec<MuscleGroup>,
    pub max_exercise_count: usize,
    pub start_date: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct EnqueueGenerationResult {
    pub job: GenerationJob,
    pub deduplicated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationPayload {
    pub muscle_groups: Vec<String>,
    pub max_exercise_count: usize,
    pub start_date: OffsetDateTime,
}

#[derive(Debug, thiserror::Error)]
pub enum GenerationServiceError {
    #[error("generation jobs repository error: {0}")]
    Repo(#[from] PostgresGenerationJobRepoError),
    #[error("dispatch error: {0}")]
    Dispatch(#[from] anyhow::Error),
}

#[derive(Clone)]
pub struct GenerationService {
    generation_jobs_db: PostgresGenerationJobDb,
    dispatcher: Arc<dyn GenerationDispatcher>,
}

impl GenerationService {
    pub fn new(
        generation_jobs_db: PostgresGenerationJobDb,
        dispatcher: Arc<dyn GenerationDispatcher>,
    ) -> Self {
        Self {
            generation_jobs_db,
            dispatcher,
        }
    }

    pub async fn enqueue(
        &self,
        user_id: UserId,
        request: GenerationRequest,
    ) -> Result<EnqueueGenerationResult, GenerationServiceError> {
        let payload = build_payload(&request);
        let fingerprint = request_fingerprint(&payload);
        let repo = self.generation_jobs_db.for_user(user_id);
        let payload_json =
            serde_json::to_value(&payload).context("serialize generation payload")?;
        let (job, deduplicated) = repo
            .create_or_reuse_active_job(payload.start_date.date(), &fingerprint, &payload_json)
            .await?;
        self.dispatcher.dispatch(job.clone()).await?;
        Ok(EnqueueGenerationResult { job, deduplicated })
    }

    pub async fn get_job(
        &self,
        user_id: UserId,
        job_id: uuid::Uuid,
    ) -> Result<Option<GenerationJob>, GenerationServiceError> {
        let repo = self.generation_jobs_db.for_user(user_id);
        Ok(repo.get_job(job_id).await?)
    }

    pub async fn list_jobs(
        &self,
        user_id: UserId,
        limit: i64,
        scope: GenerationJobListScope,
    ) -> Result<Vec<GenerationJob>, GenerationServiceError> {
        let repo = self.generation_jobs_db.for_user(user_id);
        Ok(repo.list_jobs(limit, scope).await?)
    }
}

pub fn parse_generation_payload(value: &Value) -> anyhow::Result<GenerationPayload> {
    serde_json::from_value(value.clone()).context("deserialize generation payload")
}

fn build_payload(request: &GenerationRequest) -> GenerationPayload {
    let mut muscle_groups = request
        .muscle_groups
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    muscle_groups.sort();
    muscle_groups.dedup();
    GenerationPayload {
        muscle_groups,
        max_exercise_count: request.max_exercise_count,
        start_date: request.start_date,
    }
}

fn request_fingerprint(payload: &GenerationPayload) -> String {
    let material = format!(
        "v1|{}|{}|{}",
        payload.start_date.date(),
        payload.muscle_groups.join(","),
        payload.max_exercise_count
    );
    let mut hasher = Sha256::new();
    hasher.update(material.as_bytes());
    let digest = hasher.finalize();
    hex::encode(digest)
}
