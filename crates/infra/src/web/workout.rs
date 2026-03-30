use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::{Json, Router};
use domain::excercise::{ExcerciseId, LoadType, PerformedSet, Workout, WorkoutExercise, WorkoutId};
use domain::types::{Weight, WeightUnits};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use super::{ApiError, AppState, AuthUser, lock_dbs};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", axum::routing::get(list_workouts).post(create_workout))
        .route("/dates", axum::routing::get(get_workout_dates))
        .route(
            "/{workout_id}",
            axum::routing::get(get_workout)
                .delete(delete_workout)
                .patch(update_workout),
        )
        .route(
            "/{workout_id}/exercises",
            axum::routing::post(add_exercise_to_workout),
        )
        .route(
            "/{workout_id}/exercises/{exercise_id}",
            axum::routing::delete(remove_exercise_from_workout),
        )
        .route(
            "/{workout_id}/exercises/{exercise_id}/sets",
            axum::routing::post(add_set),
        )
        .route(
            "/{workout_id}/exercises/{exercise_id}/sets/{set_index}",
            axum::routing::delete(remove_set),
        )
}

#[derive(Serialize)]
struct WorkoutResponse {
    id: String,
    name: Option<String>,
    start_date: i64,
    end_date: Option<i64>,
    entries: Vec<WorkoutEntryResponse>,
}

#[derive(Serialize)]
struct WorkoutEntryResponse {
    excercise_id: String,
    notes: Option<String>,
    sets: Vec<SetResponse>,
}

#[derive(Serialize)]
struct SetResponse {
    reps: u32,
    load: LoadResponse,
}

#[derive(Serialize)]
#[serde(tag = "type")]
enum LoadResponse {
    #[serde(rename = "weighted")]
    Weighted { value: f64, units: String },
    #[serde(rename = "bodyweight")]
    BodyWeight,
}

impl From<Workout> for WorkoutResponse {
    fn from(w: Workout) -> Self {
        Self {
            id: w.id.as_uuid().to_string(),
            name: w.name,
            start_date: w.start_date.unix_timestamp(),
            end_date: w.end_date.map(|d| d.unix_timestamp()),
            entries: w.entries.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<WorkoutExercise> for WorkoutEntryResponse {
    fn from(e: WorkoutExercise) -> Self {
        Self {
            excercise_id: e.excercise_id.as_uuid().to_string(),
            notes: e.notes,
            sets: e.sets.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<PerformedSet> for SetResponse {
    fn from(s: PerformedSet) -> Self {
        Self {
            reps: s.reps,
            load: match s.kind {
                LoadType::Weighted(w) => LoadResponse::Weighted {
                    value: w.value,
                    units: w.units.to_string(),
                },
                LoadType::BodyWeight => LoadResponse::BodyWeight,
            },
        }
    }
}

#[derive(Deserialize)]
struct CreateWorkoutRequest {
    name: Option<String>,
    date: Option<OffsetDateTime>,
}

#[derive(Deserialize)]
struct ListWorkoutsQuery {
    date: Option<String>,
}

#[derive(Deserialize)]
struct UpdateWorkoutRequest {
    name: Option<String>,
}

#[derive(Deserialize)]
struct WorkoutDatesQuery {
    from: String,
    to: String,
}

#[derive(Serialize)]
struct WorkoutDatesResponse {
    dates: Vec<String>,
}

#[derive(Deserialize)]
struct AddExerciseRequest {
    excercise_id: String,
}

#[derive(Deserialize)]
struct AddSetRequest {
    reps: u32,
    load: LoadRequest,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum LoadRequest {
    #[serde(rename = "weighted")]
    Weighted { value: f64, units: String },
    #[serde(rename = "bodyweight")]
    BodyWeight,
}

fn parse_uuid(s: &str) -> Result<uuid::Uuid, ApiError> {
    uuid::Uuid::parse_str(s).map_err(|e| ApiError::Internal(format!("invalid uuid: {e}")))
}

fn parse_date(s: &str) -> Result<time::Date, ApiError> {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 3 {
        return Err(ApiError::Internal(format!(
            "invalid date format, expected YYYY-MM-DD: {s}"
        )));
    }
    let year: i32 = parts[0]
        .parse()
        .map_err(|_| ApiError::Internal(format!("invalid year in date: {s}")))?;
    let month: u8 = parts[1]
        .parse()
        .map_err(|_| ApiError::Internal(format!("invalid month in date: {s}")))?;
    let day: u8 = parts[2]
        .parse()
        .map_err(|_| ApiError::Internal(format!("invalid day in date: {s}")))?;

    let month = time::Month::try_from(month)
        .map_err(|_| ApiError::Internal(format!("month out of range: {s}")))?;

    time::Date::from_calendar_date(year, month, day)
        .map_err(|e| ApiError::Internal(format!("invalid date: {e}")))
}

fn parse_weight_units(s: &str) -> Result<WeightUnits, ApiError> {
    match s.to_lowercase().as_str() {
        "kg" | "kilograms" => Ok(WeightUnits::Kilograms),
        "lbs" | "pounds" => Ok(WeightUnits::Pounds),
        _ => Err(ApiError::Internal(format!("unknown weight units: {s}"))),
    }
}

fn load_request_to_domain(req: LoadRequest) -> Result<LoadType, ApiError> {
    match req {
        LoadRequest::Weighted { value, units } => Ok(LoadType::Weighted(Weight::new(
            value,
            parse_weight_units(&units)?,
        ))),
        LoadRequest::BodyWeight => Ok(LoadType::BodyWeight),
    }
}

async fn list_workouts(
    user: AuthUser,
    State(state): State<AppState>,
    Query(query): Query<ListWorkoutsQuery>,
) -> Result<Json<Vec<WorkoutResponse>>, ApiError> {
    let dbs = lock_dbs(&state)?;
    let app = dbs.gym_app(user.0);

    let workouts = match query.date {
        Some(ref date_str) => {
            let date = parse_date(date_str)?;
            app.get_workout_by_date(date).map_err(ApiError::internal)?
        }
        None => app.get_all_workouts().map_err(ApiError::internal)?,
    };

    Ok(Json(workouts.into_iter().map(Into::into).collect()))
}

async fn create_workout(
    user: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<CreateWorkoutRequest>,
) -> Result<(StatusCode, Json<WorkoutResponse>), ApiError> {
    let dbs = lock_dbs(&state)?;
    let app = dbs.gym_app(user.0);
    let workout = app
        .create_new_workout(body.name, body.date)
        .map_err(ApiError::internal)?;

    Ok((StatusCode::CREATED, Json(workout.into())))
}

async fn get_workout(
    user: AuthUser,
    State(state): State<AppState>,
    Path(workout_id): Path<String>,
) -> Result<Json<WorkoutResponse>, ApiError> {
    let id = WorkoutId::from_uuid(parse_uuid(&workout_id)?);

    let dbs = lock_dbs(&state)?;
    let app = dbs.gym_app(user.0);
    let workout = app
        .get_workout_by_id(&id)
        .map_err(ApiError::internal)?
        .ok_or(ApiError::NotFound)?;

    Ok(Json(workout.into()))
}

async fn delete_workout(
    user: AuthUser,
    State(state): State<AppState>,
    Path(workout_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let id = WorkoutId::from_uuid(parse_uuid(&workout_id)?);
    let dbs = lock_dbs(&state)?;
    let app = dbs.gym_app(user.0);
    app.delete_workout(&id).map_err(ApiError::internal)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn update_workout(
    user: AuthUser,
    State(state): State<AppState>,
    Path(workout_id): Path<String>,
    Json(body): Json<UpdateWorkoutRequest>,
) -> Result<Json<WorkoutResponse>, ApiError> {
    let id = WorkoutId::from_uuid(parse_uuid(&workout_id)?);
    let dbs = lock_dbs(&state)?;
    let app = dbs.gym_app(user.0);
    app
        .update_workout_name(&id, body.name.as_deref())
        .map_err(ApiError::internal)?;
    let workout = app
        .get_workout_by_id(&id)
        .map_err(ApiError::internal)?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(workout.into()))
}

async fn remove_exercise_from_workout(
    user: AuthUser,
    State(state): State<AppState>,
    Path((workout_id, exercise_id)): Path<(String, String)>,
) -> Result<StatusCode, ApiError> {
    let wid = WorkoutId::from_uuid(parse_uuid(&workout_id)?);
    let eid = ExcerciseId::from_uuid(parse_uuid(&exercise_id)?);
    let dbs = lock_dbs(&state)?;
    let app = dbs.gym_app(user.0);
    app
        .remove_excercise_from_workout(&wid, &eid)
        .map_err(ApiError::internal)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn remove_set(
    user: AuthUser,
    State(state): State<AppState>,
    Path((workout_id, exercise_id, set_index)): Path<(String, String, usize)>,
) -> Result<StatusCode, ApiError> {
    let wid = WorkoutId::from_uuid(parse_uuid(&workout_id)?);
    let eid = ExcerciseId::from_uuid(parse_uuid(&exercise_id)?);
    let dbs = lock_dbs(&state)?;
    let app = dbs.gym_app(user.0);
    app
        .remove_set_from_workout(&wid, &eid, set_index)
        .map_err(ApiError::internal)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_workout_dates(
    user: AuthUser,
    State(state): State<AppState>,
    Query(query): Query<WorkoutDatesQuery>,
) -> Result<Json<WorkoutDatesResponse>, ApiError> {
    let from = parse_date(&query.from)?;
    let to = parse_date(&query.to)?;
    let dbs = lock_dbs(&state)?;
    let app = dbs.gym_app(user.0);
    let dates = app
        .get_workout_dates_in_range(from, to)
        .map_err(ApiError::internal)?;
    Ok(Json(WorkoutDatesResponse {
        dates: dates.iter().map(|d| d.to_string()).collect(),
    }))
}

async fn add_exercise_to_workout(
    user: AuthUser,
    State(state): State<AppState>,
    Path(workout_id): Path<String>,
    Json(body): Json<AddExerciseRequest>,
) -> Result<StatusCode, ApiError> {
    let wid = WorkoutId::from_uuid(parse_uuid(&workout_id)?);
    let eid = ExcerciseId::from_uuid(parse_uuid(&body.excercise_id)?);

    let dbs = lock_dbs(&state)?;
    let app = dbs.gym_app(user.0);
    app.add_excercise_to_workout(&wid, eid)
        .map_err(ApiError::internal)?;

    Ok(StatusCode::CREATED)
}

async fn add_set(
    user: AuthUser,
    State(state): State<AppState>,
    Path((workout_id, exercise_id)): Path<(String, String)>,
    Json(body): Json<AddSetRequest>,
) -> Result<StatusCode, ApiError> {
    let wid = WorkoutId::from_uuid(parse_uuid(&workout_id)?);
    let eid = ExcerciseId::from_uuid(parse_uuid(&exercise_id)?);
    let load = load_request_to_domain(body.load)?;

    let dbs = lock_dbs(&state)?;
    let app = dbs.gym_app(user.0);
    app.add_set_for_excercise(
        &wid,
        &eid,
        PerformedSet {
            kind: load,
            reps: body.reps,
        },
    )
    .map_err(ApiError::internal)?;

    Ok(StatusCode::CREATED)
}
