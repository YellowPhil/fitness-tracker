use axum::extract::State;
use axum::{Json, Router};
use domain::{
    health::HealthParams,
    types::{Height, HeightUnits, Weight, WeightUnits},
};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use super::{ApiError, AppState, AuthUser, lock_dbs};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", axum::routing::get(get_profile).put(update_profile))
        .route("/weight", axum::routing::patch(update_weight))
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

#[derive(Deserialize)]
struct UpdateProfileRequest {
    weight_value: f64,
    weight_units: String,
    height_value: f64,
    height_units: String,
    age: u32,
}

#[derive(Deserialize)]
struct UpdateWeightRequest {
    value: f64,
    units: String,
}

fn parse_weight_units(s: &str) -> Result<WeightUnits, ApiError> {
    match s {
        "kg" => Ok(WeightUnits::Kilograms),
        "lbs" => Ok(WeightUnits::Pounds),
        _ => Err(ApiError::validation(format!("unknown weight units: {s}"))),
    }
}

fn parse_height_units(s: &str) -> Result<HeightUnits, ApiError> {
    match s {
        "cm" => Ok(HeightUnits::Centimeters),
        "in" => Ok(HeightUnits::Inches),
        _ => Err(ApiError::validation(format!("unknown height units: {s}"))),
    }
}

#[instrument(skip(state, user), fields(user_id = user.0.as_i64()))]
async fn get_profile(
    user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<ProfileResponse>, ApiError> {
    let dbs = lock_dbs(&state)?;
    let app = dbs.health_app(user.0);
    let params = app.get_profile().map_err(ApiError::internal)?;
    Ok(Json(params.into()))
}

#[instrument(skip(state, user, body), fields(user_id = user.0.as_i64()))]
async fn update_profile(
    user: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<UpdateProfileRequest>,
) -> Result<Json<ProfileResponse>, ApiError> {
    let weight_units = parse_weight_units(&body.weight_units)?;
    let height_units = parse_height_units(&body.height_units)?;

    let params = HealthParams::new(
        Height::new(body.height_value, height_units),
        Weight::new(body.weight_value, weight_units),
        body.age,
    );

    let dbs = lock_dbs(&state)?;
    let app = dbs.health_app(user.0);
    let saved = app.update_profile(params).map_err(ApiError::internal)?;
    Ok(Json(saved.into()))
}

#[instrument(skip(state, user, body), fields(user_id = user.0.as_i64()))]
async fn update_weight(
    user: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<UpdateWeightRequest>,
) -> Result<Json<ProfileResponse>, ApiError> {
    let units = parse_weight_units(&body.units)?;
    let weight = Weight::new(body.value, units);

    let dbs = lock_dbs(&state)?;
    let app = dbs.health_app(user.0);
    let updated = app.update_weight(weight).map_err(ApiError::internal)?;
    Ok(Json(updated.into()))
}
