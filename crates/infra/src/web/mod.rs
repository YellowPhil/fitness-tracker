pub mod excercise;
pub mod profile;
pub mod telegram_auth;
pub mod workout;

use std::path::Path;
use std::sync::Arc;

use anyhow::Context;
use axum::Router;
use axum::extract::FromRequestParts;
use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
use axum::http::request::Parts;
use axum::http::{HeaderName, HeaderValue, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use domain::types::UserId;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing::{debug, error, instrument, warn};

use crate::{
    PostgresExcerciseDb, PostgresHealthDb, PostgresWorkoutDb,
};

pub struct Databases {
    pub exercise_db: PostgresExcerciseDb,
    pub workout_db: PostgresWorkoutDb,
    pub health_db: PostgresHealthDb,
}

impl Databases {
    pub fn new(
        exercise_db: PostgresExcerciseDb,
        workout_db: PostgresWorkoutDb,
        health_db: PostgresHealthDb,
    ) -> Self {
        Self {
            exercise_db,
            workout_db,
            health_db,
        }
    }

    pub async fn connect(postgres_url: &str) -> anyhow::Result<Self> {
        let pool = crate::repos::postgres::connect(postgres_url)
            .await
            .context("connect postgres")?;

        Ok(Self::new(
            PostgresExcerciseDb::new(pool.clone()),
            PostgresWorkoutDb::new(pool.clone()),
            PostgresHealthDb::new(pool),
        ))
    }

    pub fn gym_app(
        &self,
        user_id: UserId,
    ) -> application::GymApp<crate::PostgresExcerciseRepo, crate::PostgresWorkoutRepo> {
        application::GymApp::new(
            self.exercise_db.for_user(user_id),
            self.workout_db.for_user(user_id),
        )
    }

    pub fn health_app(&self, user_id: UserId) -> application::HealthApp<crate::PostgresHealthRepo> {
        application::HealthApp::new(self.health_db.for_user(user_id))
    }
}

/// HTTP-layer state: databases plus Telegram bot token for `initData` validation.
pub struct InnerState {
    pub databases: Arc<Databases>,
    /// When `Some`, API requires `Authorization: tma <initData>` validated with this token.
    pub bot_token: Option<String>,
    /// When `true` and `bot_token` is `None`, accept legacy `x-user-id` (local dev only).
    pub dev_skip_auth: bool,
    /// OpenAI API key for AI workout generation (`POST /api/workouts/generate`).
    pub openai_api_key: Option<String>,
    /// When `Some`, only the listed Telegram user IDs may access the API.
    /// When `None`, every authenticated user is allowed.
    pub allowed_user_ids: Option<Vec<i64>>,
}

pub type AppState = Arc<InnerState>;

/// JSON API under `/api/*`
pub fn router(
    dbs: Arc<Databases>,
    bot_token: Option<String>,
    dev_skip_auth: bool,
    openai_api_key: Option<String>,
    allowed_user_ids: Option<Vec<i64>>,
) -> Router<()> {
    let state: AppState = Arc::new(InnerState {
        databases: dbs,
        bot_token,
        dev_skip_auth,
        openai_api_key,
        allowed_user_ids,
    });

    Router::new()
        .nest("/api/exercises", excercise::routes())
        .nest("/api/workouts", workout::routes())
        .nest("/api/profile", profile::routes())
        .with_state(state)
}

/// JSON API under `/api/*` plus the built SPA from `web/dist` when `web/dist/index.html` exists.
///
/// When `frontend_url` is set (production: frontend on a different origin), a CORS layer is added
/// allowing that origin to call the API.
pub fn http_router(
    dbs: Arc<Databases>,
    frontend_url: Option<&str>,
    bot_token: Option<String>,
    dev_skip_auth: bool,
    openai_api_key: Option<String>,
    allowed_user_ids: Option<Vec<i64>>,
) -> Router<()> {
    let api = router(
        dbs,
        bot_token,
        dev_skip_auth,
        openai_api_key,
        allowed_user_ids,
    );
    let dist = Path::new("web/dist");

    let mut router = if dist.join("index.html").exists() {
        Router::new()
            .merge(api)
            .fallback_service(ServeDir::new("web/dist"))
    } else {
        api
    };

    router = router.route("/health", get(|| async { StatusCode::OK }));
    router = router.layer(TraceLayer::new_for_http());

    if let Some(origin) = frontend_url {
        let cors = CorsLayer::new()
            .allow_origin(
                origin
                    .parse::<HeaderValue>()
                    .expect("FRONTEND_URL must be a valid header value"),
            )
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::PATCH,
                Method::DELETE,
            ])
            .allow_headers([
                CONTENT_TYPE,
                AUTHORIZATION,
                HeaderName::from_static("x-user-id"),
            ]);
        router = router.layer(cors);
    }

    router
}

pub struct AuthUser(pub UserId);

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = ApiError;

    #[instrument(skip(parts, state))]
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let user_id: i64 = if state.dev_skip_auth {
            parts
                .headers
                .get("x-user-id")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<i64>().ok())
                .ok_or(ApiError::Unauthorized)?
        } else if let Some(ref token) = state.bot_token {
            let init_data = parts
                .headers
                .get(AUTHORIZATION)
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.strip_prefix("tma "))
                .ok_or(ApiError::Unauthorized)?;

            let tg_user = telegram_auth::validate_init_data_default(init_data, token)
                .map_err(|_| ApiError::Unauthorized)?;

            tg_user.id
        } else {
            return Err(ApiError::Unauthorized);
        };

        if let Some(ref allowed) = state.allowed_user_ids {
            if !allowed.contains(&user_id) {
                warn!(user_id, "rejected: user not in allowlist");
                return Err(ApiError::Forbidden);
            }
        }

        Ok(AuthUser(UserId::new(user_id)))
    }
}

pub enum ApiError {
    Unauthorized,
    /// The user is authenticated but not in the allowlist.
    Forbidden,
    NotFound,
    /// Service unavailable (e.g. AI features not configured).
    ServiceUnavailable(&'static str),
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
                warn!(
                    "responding unauthorized (missing or invalid Telegram initData / credentials)"
                );
                (StatusCode::UNAUTHORIZED, "unauthorized")
            }
            Self::Forbidden => {
                warn!("responding forbidden (user not in allowlist)");
                (StatusCode::FORBIDDEN, "forbidden")
            }
            Self::NotFound => {
                debug!("responding not found");
                (StatusCode::NOT_FOUND, "not found")
            }
            Self::ServiceUnavailable(reason) => {
                warn!(%reason, "service unavailable");
                (StatusCode::SERVICE_UNAVAILABLE, reason)
            }
            Self::Internal(ref e) => {
                return (StatusCode::INTERNAL_SERVER_ERROR, e.clone()).into_response();
            }
        };
        (status, msg).into_response()
    }
}
