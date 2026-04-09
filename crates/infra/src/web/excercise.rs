use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{Json, Router};
use domain::types::{Exercise, ExerciseId, ExerciseKind, ExerciseSource, MuscleGroup};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use super::types::{ExerciseKindReq, MuscleGroupReq, Name};
use super::{ApiError, AppState, AuthUser};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/",
            axum::routing::get(list_exercises).post(create_exercise),
        )
        .route("/{exercise_id}", axum::routing::delete(delete_exercise))
}

#[derive(Deserialize)]
struct CreateExcerciseRequest {
    name: Name,
    kind: ExerciseKindReq,
    muscle_group: MuscleGroupReq,
    secondary_muscle_groups: Option<Vec<MuscleGroupReq>>,
}

#[derive(Serialize)]
struct ExcerciseResponse {
    id: String,
    name: String,
    kind: String,
    muscle_group: String,
    secondary_muscle_groups: Option<Vec<String>>,
    source: String,
}

impl From<Exercise> for ExcerciseResponse {
    fn from(e: Exercise) -> Self {
        Self {
            id: e.id.as_uuid().to_string(),
            name: e.name,
            kind: match e.kind {
                ExerciseKind::Weighted => "weighted",
                ExerciseKind::BodyWeight => "bodyweight",
            }
            .into(),
            muscle_group: e.muscle_group.to_string(),
            secondary_muscle_groups: e
                .secondary_muscle_groups
                .map(|groups| groups.into_iter().map(|g| g.to_string()).collect()),
            source: match e.source {
                ExerciseSource::BuiltIn => "builtin",
                ExerciseSource::UserDefined => "user",
            }
            .into(),
        }
    }
}

fn parse_uuid(s: &str) -> Result<uuid::Uuid, ApiError> {
    uuid::Uuid::parse_str(s).map_err(|e| ApiError::validation(format!("invalid uuid: {e}")))
}

#[instrument(skip(state, user), fields(user_id = user.0.as_i64()))]
async fn list_exercises(
    user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<ExcerciseResponse>>, ApiError> {
    let app = state.databases.gym_app(user.0);
    app.seed_built_in_excercises()
        .await
        .map_err(ApiError::internal)?;
    let exercises = app.get_all_excercises().await.map_err(ApiError::internal)?;

    Ok(Json(exercises.into_iter().map(Into::into).collect()))
}

#[instrument(skip(state, user, body), fields(user_id = user.0.as_i64()))]
async fn create_exercise(
    user: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<CreateExcerciseRequest>,
) -> Result<StatusCode, ApiError> {
    let kind = ExerciseKind::from(body.kind);
    let muscle_group = MuscleGroup::from(body.muscle_group);
    let secondary = body
        .secondary_muscle_groups
        .map(|groups| groups.into_iter().map(MuscleGroup::from).collect());

    state
        .databases
        .gym_app(user.0)
        .add_new_excercise(String::from(body.name), muscle_group, secondary, kind)
        .await
        .map_err(ApiError::internal)?;

    Ok(StatusCode::CREATED)
}

#[instrument(
    skip(state, user),
    fields(user_id = user.0.as_i64(), exercise_id = %exercise_id)
)]
async fn delete_exercise(
    user: AuthUser,
    State(state): State<AppState>,
    Path(exercise_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let id = ExerciseId::from_uuid(parse_uuid(&exercise_id)?);
    state
        .databases
        .gym_app(user.0)
        .delete_excercise(&id)
        .await
        .map_err(ApiError::internal)?;
    Ok(StatusCode::NO_CONTENT)
}
