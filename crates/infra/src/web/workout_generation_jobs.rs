use std::convert::Infallible;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::{Json, Router};
use domain::types::MuscleGroup;
use futures::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tokio_stream::wrappers::BroadcastStream;
use tracing::{instrument, warn};

use crate::generation::{EnqueueGenerationResult, GenerationRequest};
use crate::repos::generation_jobs::{GenerationJob, GenerationJobListScope};

use super::types::MuscleGroupReq;
use super::{ApiError, AppState, AuthUser};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", axum::routing::get(list_generation_jobs))
        .route("/stream", axum::routing::get(stream_generation_jobs))
        .route("/{job_id}", axum::routing::get(get_generation_job))
}

#[derive(Deserialize)]
pub struct GenerateWorkoutRequest {
    muscle_groups: Vec<MuscleGroupReq>,
    max_exercise_count: usize,
    #[serde(default, with = "time::serde::rfc3339::option")]
    date: Option<OffsetDateTime>,
}

#[derive(Serialize)]
pub struct EnqueueGenerationResponse {
    job: GenerationJobResponse,
    deduplicated: bool,
}

#[derive(Serialize)]
pub struct GenerationJobListResponse {
    jobs: Vec<GenerationJobResponse>,
}

#[derive(Serialize)]
pub struct GenerationJobEnvelope {
    job: GenerationJobResponse,
}

#[derive(Serialize)]
pub struct GenerationSnapshotResponse {
    jobs: Vec<GenerationJobResponse>,
}

#[derive(Serialize)]
pub struct GenerationJobResponse {
    id: String,
    status: String,
    date: String,
    request_fingerprint: String,
    workout_id: Option<String>,
    error: Option<String>,
    version: i64,
    created_at: String,
    updated_at: String,
    queued_at: String,
    started_at: Option<String>,
    completed_at: Option<String>,
    failed_at: Option<String>,
}

#[derive(Deserialize)]
pub struct ListGenerationJobsQuery {
    limit: Option<i64>,
    status: Option<String>,
}

#[instrument(
    skip(state, user, body),
    fields(
        user_id = user.0.as_i64(),
        muscle_group_count = body.muscle_groups.len(),
        max_exercise_count = body.max_exercise_count
    )
)]
pub async fn create_generation_job(
    user: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<GenerateWorkoutRequest>,
) -> Result<(StatusCode, Json<EnqueueGenerationResponse>), ApiError> {
    if body.muscle_groups.is_empty() {
        return Err(ApiError::validation("muscle_groups must not be empty"));
    }
    if body.max_exercise_count == 0 {
        return Err(ApiError::validation(
            "max_exercise_count must be at least 1",
        ));
    }

    let muscle_groups = body
        .muscle_groups
        .into_iter()
        .map(MuscleGroup::from)
        .collect::<Vec<_>>();

    let start_date = body.date.unwrap_or_else(OffsetDateTime::now_utc);
    let request = GenerationRequest {
        muscle_groups,
        max_exercise_count: body.max_exercise_count,
        start_date,
    };

    let result = state
        .generation_service
        .enqueue(user.0, request)
        .await
        .map_err(ApiError::internal)?;

    Ok((
        StatusCode::ACCEPTED,
        Json(EnqueueGenerationResponse::from(result)),
    ))
}

#[instrument(skip(state, user, query), fields(user_id = user.0.as_i64()))]
async fn list_generation_jobs(
    user: AuthUser,
    State(state): State<AppState>,
    Query(query): Query<ListGenerationJobsQuery>,
) -> Result<Json<GenerationJobListResponse>, ApiError> {
    let scope = match query.status.as_deref() {
        Some("active") => GenerationJobListScope::Active,
        Some("all") | None => GenerationJobListScope::All,
        Some(_) => return Err(ApiError::validation("status must be 'active' or 'all'")),
    };
    let limit = query.limit.unwrap_or(20).clamp(1, 100);

    let jobs = state
        .generation_service
        .list_jobs(user.0, limit, scope)
        .await
        .map_err(ApiError::internal)?;

    Ok(Json(GenerationJobListResponse {
        jobs: jobs.into_iter().map(GenerationJobResponse::from).collect(),
    }))
}

#[instrument(skip(state, user), fields(user_id = user.0.as_i64(), job_id = %job_id))]
async fn get_generation_job(
    user: AuthUser,
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> Result<Json<GenerationJobEnvelope>, ApiError> {
    let job_id = uuid::Uuid::parse_str(&job_id)
        .map_err(|e| ApiError::validation(format!("invalid uuid: {e}")))?;

    let job = state
        .generation_service
        .get_job(user.0, job_id)
        .await
        .map_err(ApiError::internal)?
        .ok_or(ApiError::NotFound)?;

    Ok(Json(GenerationJobEnvelope {
        job: GenerationJobResponse::from(job),
    }))
}

#[instrument(skip(state, user), fields(user_id = user.0.as_i64()))]
async fn stream_generation_jobs(
    user: AuthUser,
    State(state): State<AppState>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, ApiError> {
    let jobs = state
        .generation_service
        .list_jobs(user.0, 20, GenerationJobListScope::All)
        .await
        .map_err(ApiError::internal)?;

    let snapshot = GenerationSnapshotResponse {
        jobs: jobs.into_iter().map(GenerationJobResponse::from).collect(),
    };

    let snapshot_event = Event::default()
        .event("snapshot")
        .data(serde_json::to_string(&snapshot).map_err(ApiError::internal)?);

    let receiver = state.generation_event_bus.subscribe(user.0);
    let updates = BroadcastStream::new(receiver).filter_map(|item| async move {
        match item {
            Ok(job) => {
                let payload = GenerationJobEnvelope {
                    job: GenerationJobResponse::from(job),
                };
                match serde_json::to_string(&payload) {
                    Ok(data) => Some(Ok(Event::default().event("job.updated").data(data))),
                    Err(err) => {
                        warn!(error = %err, "failed to serialize generation update");
                        None
                    }
                }
            }
            Err(err) => {
                warn!(error = %err, "generation stream lagged");
                None
            }
        }
    });

    let stream = tokio_stream::once(Ok(snapshot_event)).chain(updates);
    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

impl From<EnqueueGenerationResult> for EnqueueGenerationResponse {
    fn from(value: EnqueueGenerationResult) -> Self {
        Self {
            job: GenerationJobResponse::from(value.job),
            deduplicated: value.deduplicated,
        }
    }
}

impl From<GenerationJob> for GenerationJobResponse {
    fn from(job: GenerationJob) -> Self {
        Self {
            id: job.id.to_string(),
            status: job.status.as_str().to_string(),
            date: job.date.to_string(),
            request_fingerprint: job.request_fingerprint,
            workout_id: job.workout_id.map(|id| id.as_uuid().to_string()),
            error: job.error,
            version: job.version,
            created_at: job
                .created_at
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_default(),
            updated_at: job
                .updated_at
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_default(),
            queued_at: job
                .queued_at
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_default(),
            started_at: job.started_at.and_then(|ts| {
                ts.format(&time::format_description::well_known::Rfc3339)
                    .ok()
            }),
            completed_at: job.completed_at.and_then(|ts| {
                ts.format(&time::format_description::well_known::Rfc3339)
                    .ok()
            }),
            failed_at: job.failed_at.and_then(|ts| {
                ts.format(&time::format_description::well_known::Rfc3339)
                    .ok()
            }),
        }
    }
}
