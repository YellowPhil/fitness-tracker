use axum::extract::State;
use axum::{Json, Router};
use domain::preferences::{TrainingGoal, WorkoutPreferences, WorkoutSplit};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use super::{ApiError, AppState, AuthUser};

pub fn routes() -> Router<AppState> {
    Router::new().route(
        "/",
        axum::routing::get(get_preferences).put(update_preferences),
    )
}

#[derive(Deserialize)]
struct UpdatePreferencesRequest {
    max_sets_per_exercise: Option<u8>,
    preferred_split: Option<String>,
    training_goal: Option<String>,
    session_duration_minutes: Option<u16>,
    notes: Option<String>,
}

#[derive(Serialize)]
struct PreferencesResponse {
    max_sets_per_exercise: Option<u8>,
    preferred_split: Option<String>,
    training_goal: Option<String>,
    session_duration_minutes: Option<u16>,
    notes: Option<String>,
}

impl From<WorkoutPreferences> for PreferencesResponse {
    fn from(value: WorkoutPreferences) -> Self {
        Self {
            max_sets_per_exercise: value.max_sets_per_exercise,
            preferred_split: value
                .preferred_split
                .map(|item| item.as_api_str().to_string()),
            training_goal: value
                .training_goal
                .map(|item| item.as_api_str().to_string()),
            session_duration_minutes: value.session_duration_minutes,
            notes: value.notes,
        }
    }
}

#[instrument(skip(state, user), fields(user_id = user.0.as_i64()))]
async fn get_preferences(
    user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<PreferencesResponse>, ApiError> {
    let preferences = state
        .databases
        .preferences_app(user.0)
        .get_preferences()
        .await
        .map_err(ApiError::internal)?;
    Ok(Json(preferences.into()))
}

#[instrument(skip(state, user, body), fields(user_id = user.0.as_i64()))]
async fn update_preferences(
    user: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<UpdatePreferencesRequest>,
) -> Result<Json<PreferencesResponse>, ApiError> {
    let preferred_split = parse_optional_split(body.preferred_split.as_deref())?;
    let training_goal = parse_optional_goal(body.training_goal.as_deref())?;

    let preferences = WorkoutPreferences {
        max_sets_per_exercise: body.max_sets_per_exercise,
        preferred_split,
        training_goal,
        session_duration_minutes: body.session_duration_minutes,
        notes: body.notes,
    };

    let saved = state
        .databases
        .preferences_app(user.0)
        .update_preferences(preferences)
        .await
        .map_err(ApiError::internal)?;

    Ok(Json(saved.into()))
}

fn parse_optional_split(value: Option<&str>) -> Result<Option<WorkoutSplit>, ApiError> {
    value
        .map(|item| {
            WorkoutSplit::parse_api_str(item).ok_or_else(|| {
                ApiError::validation(format!(
                    "invalid preferred_split '{item}' (expected one of: FullBody, PushPullLegs, UpperLower)"
                ))
            })
        })
        .transpose()
}

fn parse_optional_goal(value: Option<&str>) -> Result<Option<TrainingGoal>, ApiError> {
    value
        .map(|item| {
            TrainingGoal::parse_api_str(item).ok_or_else(|| {
                ApiError::validation(format!(
                    "invalid training_goal '{item}' (expected one of: Strength, Hypertrophy, Endurance)"
                ))
            })
        })
        .transpose()
}
