use axum::extract::State;
use axum::{Json, Router};
use domain::{
    health::HealthParams,
    types::{Height, Weight},
};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use super::types::{HeightUnitsReq, WeightUnitsReq};
use super::{ApiError, AppState, AuthUser};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", axum::routing::get(get_profile).put(update_profile))
        .route("/weight", axum::routing::patch(update_weight))
}

#[derive(Deserialize)]
struct UpdateProfileRequest {
    weight_value: f64,
    weight_units: WeightUnitsReq,
    height_value: f64,
    height_units: HeightUnitsReq,
    age: u32,
}

#[derive(Deserialize)]
struct UpdateWeightRequest {
    value: f64,
    units: WeightUnitsReq,
}

#[derive(Serialize)]
struct ProfileResponse {
    weight_value: f64,
    weight_units: String,
    height_value: f64,
    height_units: String,
    age: u32,
}

impl From<HealthParams> for ProfileResponse {
    fn from(p: HealthParams) -> Self {
        Self {
            weight_value: p.weight.value,
            weight_units: p.weight.units.to_string(),
            height_value: p.height.value,
            height_units: p.height.units.to_string(),
            age: p.age,
        }
    }
}

#[instrument(skip(state, user), fields(user_id = user.0.as_i64()))]
async fn get_profile(
    user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<ProfileResponse>, ApiError> {
    let params = state
        .databases
        .health_app(user.0)
        .get_profile()
        .await
        .map_err(ApiError::internal)?;
    Ok(Json(params.into()))
}

#[instrument(skip(state, user, body), fields(user_id = user.0.as_i64()))]
async fn update_profile(
    user: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<UpdateProfileRequest>,
) -> Result<Json<ProfileResponse>, ApiError> {
    let params = HealthParams::new(
        Height::new(body.height_value, body.height_units.into()),
        Weight::new(body.weight_value, body.weight_units.into()),
        body.age,
    );

    let saved = state
        .databases
        .health_app(user.0)
        .update_profile(params)
        .await
        .map_err(ApiError::internal)?;
    Ok(Json(saved.into()))
}

#[instrument(skip(state, user, body), fields(user_id = user.0.as_i64()))]
async fn update_weight(
    user: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<UpdateWeightRequest>,
) -> Result<Json<ProfileResponse>, ApiError> {
    let weight = Weight::new(body.value, body.units.into());

    let updated = state
        .databases
        .health_app(user.0)
        .update_weight(weight)
        .await
        .map_err(ApiError::internal)?;
    Ok(Json(updated.into()))
}
