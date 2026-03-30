use axum::extract::State;
use axum::{Json, Router};
use domain::excercise::{Excercise, ExcerciseKind, ExcerciseSource, MuscleGroup};
use serde::{Deserialize, Serialize};

use super::{ApiError, AppState, AuthUser, lock_dbs};

pub fn routes() -> Router<AppState> {
    Router::new().route(
        "/",
        axum::routing::get(list_exercises).post(create_exercise),
    )
}

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct ExcerciseResponse {
    id: String,
    name: String,
    kind: String,
    muscle_group: String,
    secondary_muscle_groups: Option<Vec<String>>,
    source: String,
}

impl From<Excercise> for ExcerciseResponse {
    fn from(e: Excercise) -> Self {
        Self {
            id: e.id.as_uuid().to_string(),
            name: e.name,
            kind: match e.kind {
                ExcerciseKind::Weighted => "weighted",
                ExcerciseKind::BodyWeight => "bodyweight",
            }
            .into(),
            muscle_group: e.muscle_group.to_string(),
            secondary_muscle_groups: e
                .secondary_muscle_groups
                .map(|groups| groups.into_iter().map(|g| g.to_string()).collect()),
            source: match e.source {
                ExcerciseSource::BuiltIn => "builtin",
                ExcerciseSource::UserDefined => "user",
            }
            .into(),
        }
    }
}

#[derive(Deserialize)]
struct CreateExcerciseRequest {
    name: String,
    kind: String,
    muscle_group: String,
    secondary_muscle_groups: Option<Vec<String>>,
}

fn parse_muscle_group(s: &str) -> Result<MuscleGroup, ApiError> {
    match s.to_lowercase().as_str() {
        "chest" => Ok(MuscleGroup::Chest),
        "back" => Ok(MuscleGroup::Back),
        "arms" => Ok(MuscleGroup::Arms),
        "legs" => Ok(MuscleGroup::Legs),
        "core" => Ok(MuscleGroup::Core),
        _ => Err(ApiError::Internal(format!("unknown muscle group: {s}"))),
    }
}

fn parse_excercise_kind(s: &str) -> Result<ExcerciseKind, ApiError> {
    match s.to_lowercase().as_str() {
        "weighted" => Ok(ExcerciseKind::Weighted),
        "bodyweight" => Ok(ExcerciseKind::BodyWeight),
        _ => Err(ApiError::Internal(format!("unknown exercise kind: {s}"))),
    }
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

async fn list_exercises(
    user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<ExcerciseResponse>>, ApiError> {
    let dbs = lock_dbs(&state)?;
    let app = dbs.gym_app(user.0);
    let exercises = app.get_all_excercises().map_err(ApiError::internal)?;

    Ok(Json(exercises.into_iter().map(Into::into).collect()))
}

async fn create_exercise(
    user: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<CreateExcerciseRequest>,
) -> Result<axum::http::StatusCode, ApiError> {
    let kind = parse_excercise_kind(&body.kind)?;
    let muscle_group = parse_muscle_group(&body.muscle_group)?;
    let secondary = body
        .secondary_muscle_groups
        .map(|groups| {
            groups
                .iter()
                .map(|g| parse_muscle_group(g))
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?;

    let dbs = lock_dbs(&state)?;
    let app = dbs.gym_app(user.0);
    app.add_new_excercise(body.name, muscle_group, secondary, kind)
        .map_err(ApiError::internal)?;

    Ok(axum::http::StatusCode::CREATED)
}
