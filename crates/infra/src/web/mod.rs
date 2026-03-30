pub mod excercise;
pub mod workout;

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use axum::Router;
use axum::extract::FromRequestParts;
use axum::http::StatusCode;
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};
use domain::types::UserId;

use crate::{SqliteExcerciseDb, SqliteExcerciseRepo, SqliteWorkoutDb, SqliteWorkoutRepo};

pub struct Databases {
    pub exercise_db: SqliteExcerciseDb,
    pub workout_db: SqliteWorkoutDb,
}

impl Databases {
    pub fn new(exercise_db: SqliteExcerciseDb, workout_db: SqliteWorkoutDb) -> Self {
        Self {
            exercise_db,
            workout_db,
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
}

pub type AppState = Arc<Mutex<Databases>>;

pub fn router(dbs: Databases) -> Router {
    let state: AppState = Arc::new(Mutex::new(dbs));

    Router::new()
        .nest("/api/exercises", excercise::routes())
        .nest("/api/workouts", workout::routes())
        .with_state(state)
}

pub struct AuthUser(pub UserId);

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = ApiError;

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
    pub fn internal(err: impl std::fmt::Display) -> Self {
        Self::Internal(err.to_string())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized"),
            Self::NotFound => (StatusCode::NOT_FOUND, "not found"),
            Self::Internal(ref e) => {
                return (StatusCode::INTERNAL_SERVER_ERROR, e.clone()).into_response();
            }
        };
        (status, msg).into_response()
    }
}

impl<T> From<PoisonError<T>> for ApiError {
    fn from(_: PoisonError<T>) -> Self {
        Self::Internal("database lock poisoned".into())
    }
}

pub fn lock_dbs(state: &AppState) -> Result<MutexGuard<'_, Databases>, ApiError> {
    state.lock().map_err(ApiError::from)
}
