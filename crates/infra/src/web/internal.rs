use axum::extract::{Path, Query, State};
use axum::{Json, Router};
use domain::traits::ExcerciseRepo;
use domain::types::{Exercise, ExerciseKind, MuscleGroup, QueryType, UserId, WorkoutQuery};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use super::types::MuscleGroupReq;
use super::{ApiError, AppState};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/users/{user_id}/exercises",
            axum::routing::get(list_exercises),
        )
        .route(
            "/users/{user_id}/ai/workout-query",
            axum::routing::post(workout_query_tool),
        )
        .route(
            "/users/{user_id}/ai/exercise-list",
            axum::routing::post(exercise_list_tool),
        )
}

#[derive(Deserialize)]
struct ExerciseCatalogQuery {
    muscle_group: MuscleGroupReq,
}

#[derive(Deserialize)]
struct WorkoutQueryToolRequest {
    date: Option<String>,
    last_n: Option<usize>,
    muscle_group: MuscleGroupReq,
}

#[derive(Deserialize)]
struct ExerciseListToolRequest {
    muscle_group: MuscleGroupReq,
}

#[derive(Serialize)]
struct ExerciseCatalogItemResponse {
    exercise_id: String,
    name: String,
    kind: String,
    muscle_group: String,
}

#[derive(Serialize)]
struct ToolContentResponse {
    content: String,
}

#[instrument(skip(state, query), fields(user_id = user_id))]
async fn list_exercises(
    Path(user_id): Path<i64>,
    State(state): State<AppState>,
    Query(query): Query<ExerciseCatalogQuery>,
) -> Result<Json<Vec<ExerciseCatalogItemResponse>>, ApiError> {
    let user_id = UserId::new(user_id);
    let app = state.databases.gym_app(user_id);
    app.seed_built_in_excercises()
        .await
        .map_err(ApiError::internal)?;

    let muscle_group = MuscleGroup::from(query.muscle_group);
    let exercises = state
        .databases
        .exercise_db
        .for_user(user_id)
        .get_by_muscle_group(muscle_group)
        .await
        .map_err(ApiError::internal)?;

    let response = exercises
        .into_iter()
        .map(exercise_catalog_item_response)
        .collect();

    Ok(Json(response))
}

#[instrument(skip(state, body), fields(user_id = user_id))]
async fn workout_query_tool(
    Path(user_id): Path<i64>,
    State(state): State<AppState>,
    Json(body): Json<WorkoutQueryToolRequest>,
) -> Result<Json<ToolContentResponse>, ApiError> {
    let user_id = UserId::new(user_id);
    let muscle_group = MuscleGroup::from(body.muscle_group);
    let date = match body.date {
        Some(raw_date) => QueryType::OnDate(parse_date_yyyy_mm_dd(&raw_date)?),
        None => match body.last_n {
            Some(last_n) => QueryType::LastN(last_n),
            None => QueryType::Latest,
        },
    };

    let result = state
        .databases
        .gym_app(user_id)
        .query_workout_resource(WorkoutQuery {
            date,
            muscle_group: Some(muscle_group),
        })
        .await
        .map_err(ApiError::internal)?;

    let content = crate::ai::format::format_workouts(
        &result.workouts,
        &result.excercises,
        Some(muscle_group),
    );

    Ok(Json(ToolContentResponse { content }))
}

#[instrument(skip(state, body), fields(user_id = user_id))]
async fn exercise_list_tool(
    Path(user_id): Path<i64>,
    State(state): State<AppState>,
    Json(body): Json<ExerciseListToolRequest>,
) -> Result<Json<ToolContentResponse>, ApiError> {
    let user_id = UserId::new(user_id);
    let app = state.databases.gym_app(user_id);
    app.seed_built_in_excercises()
        .await
        .map_err(ApiError::internal)?;

    let muscle_group = MuscleGroup::from(body.muscle_group);
    let exercises = state
        .databases
        .exercise_db
        .for_user(user_id)
        .get_by_muscle_group(muscle_group)
        .await
        .map_err(ApiError::internal)?;

    let metadata = exercises.iter().map(Exercise::metadata).collect::<Vec<_>>();
    let content = crate::ai::format::format_exercises(&metadata, Some(muscle_group));

    Ok(Json(ToolContentResponse { content }))
}

fn exercise_catalog_item_response(exercise: Exercise) -> ExerciseCatalogItemResponse {
    let kind = match exercise.kind {
        ExerciseKind::Weighted => "Weighted",
        ExerciseKind::BodyWeight => "BodyWeight",
    }
    .to_string();

    ExerciseCatalogItemResponse {
        exercise_id: exercise.id.as_uuid().to_string(),
        name: exercise.name,
        kind,
        muscle_group: exercise.muscle_group.to_string(),
    }
}

fn parse_date_yyyy_mm_dd(input: &str) -> Result<time::Date, ApiError> {
    let format = time::format_description::parse_borrowed::<2>("[year]-[month]-[day]")
        .map_err(ApiError::validation)?;
    time::Date::parse(input, &format)
        .map_err(|_| ApiError::validation(format!("invalid date format: {input}")))
}
