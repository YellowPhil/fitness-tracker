use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::{Json, Router};
use domain::types::{
    ExerciseId, LoadType, MuscleGroup, PerformedSet, Workout, WorkoutExercise, WorkoutId,
};
use domain::types::Weight;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tracing::instrument;

use crate::ai::WorkoutGenerator;

use super::types::{MuscleGroupReq, Name, WeightUnitsReq};
use super::{ApiError, AppState, AuthUser};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", axum::routing::get(list_workouts).post(create_workout))
        .route("/dates", axum::routing::get(get_workout_dates))
        .route("/generate", axum::routing::post(generate_workout_ai))
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
            axum::routing::put(update_set).delete(remove_set),
        )
}

#[derive(Serialize)]
struct WorkoutResponse {
    id: String,
    name: Option<String>,
    start_date: i64,
    end_date: Option<i64>,
    /// Wire value from `WorkoutSource::as_api_str` (`domain::excercise::workout_source`).
    source: String,
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
            source: w.source.as_api_str().to_string(),
            entries: w.entries.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<WorkoutExercise> for WorkoutEntryResponse {
    fn from(e: WorkoutExercise) -> Self {
        Self {
            excercise_id: e.exercise_id.as_uuid().to_string(),
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
    name: Option<Name>,
    #[serde(default, with = "time::serde::rfc3339::option")]
    date: Option<OffsetDateTime>,
}

#[derive(Deserialize)]
struct GenerateWorkoutRequest {
    /// Muscle group names as returned by the API (e.g. `"Chest"`, `"Legs"`).
    muscle_groups: Vec<MuscleGroupReq>,
    max_exercise_count: usize,
    #[serde(default, with = "time::serde::rfc3339::option")]
    date: Option<OffsetDateTime>,
}

#[derive(Deserialize)]
struct ListWorkoutsQuery {
    date: Option<String>,
}

#[derive(Deserialize)]
struct UpdateWorkoutRequest {
    name: Option<Name>,
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
    Weighted { value: f64, units: WeightUnitsReq },
    #[serde(rename = "bodyweight")]
    BodyWeight,
}

impl From<LoadRequest> for LoadType {
    fn from(req: LoadRequest) -> Self {
        match req {
            LoadRequest::Weighted { value, units } => {
                LoadType::Weighted(Weight::new(value, units.into()))
            }
            LoadRequest::BodyWeight => LoadType::BodyWeight,
        }
    }
}

fn parse_uuid(s: &str) -> Result<uuid::Uuid, ApiError> {
    uuid::Uuid::parse_str(s).map_err(|e| ApiError::validation(format!("invalid uuid: {e}")))
}

fn parse_date(s: &str) -> Result<time::Date, ApiError> {
    let format = time::format_description::parse_borrowed::<2>("[year]-[month]-[day]").unwrap();
    time::Date::parse(s, &format)
        .map_err(|_| ApiError::validation(format!("invalid date format: {s}")))
}

#[instrument(
    skip(state, user, query),
    fields(user_id = user.0.as_i64(), filter_date = ?query.date)
)]
async fn list_workouts(
    user: AuthUser,
    State(state): State<AppState>,
    Query(query): Query<ListWorkoutsQuery>,
) -> Result<Json<Vec<WorkoutResponse>>, ApiError> {
    let date = query.date.as_deref().map(parse_date).transpose()?;
    let app = state.databases.gym_app(user.0);
    let workouts = match date {
        Some(d) => app
            .get_workout_by_date(d)
            .await
            .map_err(ApiError::internal)?,
        None => app.get_all_workouts().await.map_err(ApiError::internal)?,
    };
    Ok(Json(workouts.into_iter().map(Into::into).collect()))
}

#[instrument(skip(state, user, body), fields(user_id = user.0.as_i64()))]
async fn create_workout(
    user: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<CreateWorkoutRequest>,
) -> Result<(StatusCode, Json<WorkoutResponse>), ApiError> {
    let workout = state
        .databases
        .gym_app(user.0)
        .create_new_workout(body.name.map(String::from), body.date)
        .await
        .map_err(ApiError::internal)?;
    Ok((StatusCode::CREATED, Json(workout.into())))
}

async fn generate_workout_ai(
    user: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<GenerateWorkoutRequest>,
) -> Result<(StatusCode, Json<WorkoutResponse>), ApiError> {
    let api_key = state
        .openai_api_key
        .clone()
        .filter(|s| !s.trim().is_empty())
        .ok_or(ApiError::ServiceUnavailable(
            "OPENAI_API_KEY not configured",
        ))?;

    if body.muscle_groups.is_empty() {
        return Err(ApiError::validation("muscle_groups must not be empty"));
    }
    if body.max_exercise_count == 0 {
        return Err(ApiError::validation(
            "max_exercise_count must be at least 1",
        ));
    }

    let muscle_groups: Vec<MuscleGroup> =
        body.muscle_groups.into_iter().map(MuscleGroup::from).collect();

    let dbs = Arc::clone(&state.databases);
    let app = state.databases.gym_app(user.0);
    app.seed_built_in_excercises()
        .await
        .map_err(ApiError::internal)?;

    let generator = WorkoutGenerator::new(dbs, user.0, api_key);
    let start_date = body.date.unwrap_or_else(OffsetDateTime::now_utc);
    let date = start_date.date();

    let generated = generator
        .generate_workout(date, &muscle_groups, body.max_exercise_count)
        .await
        .map_err(ApiError::internal)?;

    let workout = Workout::ai_generated(generated.name, start_date, generated.exercises);

    state
        .databases
        .gym_app(user.0)
        .save_workout(&workout)
        .await
        .map_err(ApiError::internal)?;

    Ok((StatusCode::CREATED, Json(workout.into())))
}

#[instrument(
    skip(state, user),
    fields(user_id = user.0.as_i64(), workout_id = %workout_id)
)]
async fn get_workout(
    user: AuthUser,
    State(state): State<AppState>,
    Path(workout_id): Path<String>,
) -> Result<Json<WorkoutResponse>, ApiError> {
    let id = WorkoutId::from_uuid(parse_uuid(&workout_id)?);
    let workout = state
        .databases
        .gym_app(user.0)
        .get_workout_by_id(&id)
        .await
        .map_err(ApiError::internal)?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(workout.into()))
}

#[instrument(
    skip(state, user),
    fields(user_id = user.0.as_i64(), workout_id = %workout_id)
)]
async fn delete_workout(
    user: AuthUser,
    State(state): State<AppState>,
    Path(workout_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let id = WorkoutId::from_uuid(parse_uuid(&workout_id)?);
    state
        .databases
        .gym_app(user.0)
        .delete_workout(&id)
        .await
        .map_err(ApiError::internal)?;
    Ok(StatusCode::NO_CONTENT)
}

#[instrument(
    skip(state, user, body),
    fields(user_id = user.0.as_i64(), workout_id = %workout_id)
)]
async fn update_workout(
    user: AuthUser,
    State(state): State<AppState>,
    Path(workout_id): Path<String>,
    Json(body): Json<UpdateWorkoutRequest>,
) -> Result<Json<WorkoutResponse>, ApiError> {
    let id = WorkoutId::from_uuid(parse_uuid(&workout_id)?);
    let app = state.databases.gym_app(user.0);
    let name = body.name.map(String::from);
    app.update_workout_name(&id, name.as_deref())
        .await
        .map_err(ApiError::internal)?;
    let workout = app
        .get_workout_by_id(&id)
        .await
        .map_err(ApiError::internal)?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(workout.into()))
}

#[instrument(
    skip(state, user),
    fields(
        user_id = user.0.as_i64(),
        workout_id = %workout_id,
        exercise_id = %exercise_id
    )
)]
async fn remove_exercise_from_workout(
    user: AuthUser,
    State(state): State<AppState>,
    Path((workout_id, exercise_id)): Path<(String, String)>,
) -> Result<StatusCode, ApiError> {
    let wid = WorkoutId::from_uuid(parse_uuid(&workout_id)?);
    let eid = ExerciseId::from_uuid(parse_uuid(&exercise_id)?);
    state
        .databases
        .gym_app(user.0)
        .remove_excercise_from_workout(&wid, &eid)
        .await
        .map_err(ApiError::internal)?;
    Ok(StatusCode::NO_CONTENT)
}

#[instrument(
    skip(state, user, body),
    fields(
        user_id = user.0.as_i64(),
        workout_id = %workout_id,
        exercise_id = %exercise_id,
        set_index = set_index
    )
)]
async fn update_set(
    user: AuthUser,
    State(state): State<AppState>,
    Path((workout_id, exercise_id, set_index)): Path<(String, String, usize)>,
    Json(body): Json<AddSetRequest>,
) -> Result<StatusCode, ApiError> {
    let wid = WorkoutId::from_uuid(parse_uuid(&workout_id)?);
    let eid = ExerciseId::from_uuid(parse_uuid(&exercise_id)?);
    let load = LoadType::from(body.load);
    state
        .databases
        .gym_app(user.0)
        .update_set_in_workout(
            &wid,
            &eid,
            set_index,
            PerformedSet {
                kind: load,
                reps: body.reps,
            },
        )
        .await
        .map_err(ApiError::internal)?;
    Ok(StatusCode::NO_CONTENT)
}

#[instrument(
    skip(state, user),
    fields(
        user_id = user.0.as_i64(),
        workout_id = %workout_id,
        exercise_id = %exercise_id,
        set_index = set_index
    )
)]
async fn remove_set(
    user: AuthUser,
    State(state): State<AppState>,
    Path((workout_id, exercise_id, set_index)): Path<(String, String, usize)>,
) -> Result<StatusCode, ApiError> {
    let wid = WorkoutId::from_uuid(parse_uuid(&workout_id)?);
    let eid = ExerciseId::from_uuid(parse_uuid(&exercise_id)?);
    state
        .databases
        .gym_app(user.0)
        .remove_set_from_workout(&wid, &eid, set_index)
        .await
        .map_err(ApiError::internal)?;
    Ok(StatusCode::NO_CONTENT)
}

#[instrument(
    skip(state, user, query),
    fields(user_id = user.0.as_i64(), from = %query.from, to = %query.to)
)]
async fn get_workout_dates(
    user: AuthUser,
    State(state): State<AppState>,
    Query(query): Query<WorkoutDatesQuery>,
) -> Result<Json<WorkoutDatesResponse>, ApiError> {
    let from = parse_date(&query.from)?;
    let to = parse_date(&query.to)?;
    let dates = state
        .databases
        .gym_app(user.0)
        .get_workout_dates_in_range(from, to)
        .await
        .map_err(ApiError::internal)?;
    Ok(Json(WorkoutDatesResponse {
        dates: dates.iter().map(|d| d.to_string()).collect(),
    }))
}

#[instrument(
    skip(state, user, body),
    fields(user_id = user.0.as_i64(), workout_id = %workout_id)
)]
async fn add_exercise_to_workout(
    user: AuthUser,
    State(state): State<AppState>,
    Path(workout_id): Path<String>,
    Json(body): Json<AddExerciseRequest>,
) -> Result<StatusCode, ApiError> {
    let wid = WorkoutId::from_uuid(parse_uuid(&workout_id)?);
    let eid = ExerciseId::from_uuid(parse_uuid(&body.excercise_id)?);

    state
        .databases
        .gym_app(user.0)
        .add_excercise_to_workout(&wid, eid)
        .await
        .map_err(ApiError::internal)?;

    Ok(StatusCode::CREATED)
}

#[instrument(
    skip(state, user, body),
    fields(
        user_id = user.0.as_i64(),
        workout_id = %workout_id,
        exercise_id = %exercise_id
    )
)]
async fn add_set(
    user: AuthUser,
    State(state): State<AppState>,
    Path((workout_id, exercise_id)): Path<(String, String)>,
    Json(body): Json<AddSetRequest>,
) -> Result<StatusCode, ApiError> {
    let wid = WorkoutId::from_uuid(parse_uuid(&workout_id)?);
    let eid = ExerciseId::from_uuid(parse_uuid(&exercise_id)?);
    let load = LoadType::from(body.load);

    state
        .databases
        .gym_app(user.0)
        .add_set_for_excercise(
            &wid,
            &eid,
            PerformedSet {
                kind: load,
                reps: body.reps,
            },
        )
        .await
        .map_err(ApiError::internal)?;

    Ok(StatusCode::CREATED)
}
