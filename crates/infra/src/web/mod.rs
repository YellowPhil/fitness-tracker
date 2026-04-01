pub mod excercise;
pub mod profile;
pub mod workout;

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use std::path::Path;

use axum::Router;
use axum::extract::FromRequestParts;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use axum::http::StatusCode;
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};
use domain::types::UserId;
use tracing::{debug, error, instrument, warn};

use crate::{
    SqliteExcerciseDb, SqliteExcerciseRepo, SqliteHealthDb, SqliteHealthRepo, SqliteWorkoutDb,
    SqliteWorkoutRepo,
};

pub struct Databases {
    pub exercise_db: SqliteExcerciseDb,
    pub workout_db: SqliteWorkoutDb,
    pub health_db: SqliteHealthDb,
}

impl Databases {
    pub fn new(
        exercise_db: SqliteExcerciseDb,
        workout_db: SqliteWorkoutDb,
        health_db: SqliteHealthDb,
    ) -> Self {
        Self {
            exercise_db,
            workout_db,
            health_db,
        }
    }

    pub fn gym_app(
        &self,
        user_id: UserId,
    ) -> application::GymApp<SqliteExcerciseRepo<'_>, SqliteWorkoutRepo<'_>> {
        application::GymApp::new(
            self.exercise_db.for_user(user_id),
            self.workout_db.for_user(user_id),
        )
    }

    pub fn health_app(
        &self,
        user_id: UserId,
    ) -> application::HealthApp<SqliteHealthRepo<'_>> {
        application::HealthApp::new(self.health_db.for_user(user_id))
    }
}

pub type AppState = Arc<Mutex<Databases>>;

pub fn router(dbs: Databases) -> Router<()> {
    let state: AppState = Arc::new(Mutex::new(dbs));

    Router::new()
        .nest("/api/exercises", excercise::routes())
        .nest("/api/workouts", workout::routes())
        .nest("/api/profile", profile::routes())
        .with_state(state)
}

/// JSON API under `/api/*` plus the built SPA from `web/dist` when `web/dist/index.html` exists.
pub fn http_router(dbs: Databases) -> Router<()> {
    let api = router(dbs).layer(TraceLayer::new_for_http());
    let dist = Path::new("web/dist");
    if dist.join("index.html").exists() {
        Router::new()
            .merge(api)
            .fallback_service(ServeDir::new("web/dist"))
    } else {
        api
    }
}

pub struct AuthUser(pub UserId);

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = ApiError;

    #[instrument(skip(parts, _state))]
    async fn from_request_parts(
        parts: &mut Parts,
        _state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get("x-user-id")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<i64>().ok())
            .ok_or(ApiError::Unauthorized)?;

        Ok(AuthUser(UserId::new(header)))
    }
}

pub enum ApiError {
    Unauthorized,
    NotFound,
    Internal(String),
}

impl ApiError {
    /// Repository or application failure surfaced as HTTP 500.
    pub fn internal(err: impl std::fmt::Display) -> Self {
        error!(error = %err, "internal error");
        Self::Internal(err.to_string())
    }

    /// Client-side parse / validation issues (still HTTP 500 for this API).
    pub fn validation(err: impl std::fmt::Display) -> Self {
        warn!(error = %err, "request validation failed");
        Self::Internal(err.to_string())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            Self::Unauthorized => {
                warn!("responding unauthorized (missing or invalid x-user-id)");
                (StatusCode::UNAUTHORIZED, "unauthorized")
            }
            Self::NotFound => {
                debug!("responding not found");
                (StatusCode::NOT_FOUND, "not found")
            }
            Self::Internal(ref e) => {
                return (StatusCode::INTERNAL_SERVER_ERROR, e.clone()).into_response();
            }
        };
        (status, msg).into_response()
    }
}

impl<T> From<PoisonError<T>> for ApiError {
    fn from(err: PoisonError<T>) -> Self {
        error!(error = %err, "database mutex poisoned");
        Self::Internal("database lock poisoned".into())
    }
}

pub fn lock_dbs(state: &AppState) -> Result<MutexGuard<'_, Databases>, ApiError> {
    state.lock().map_err(ApiError::from)
}
