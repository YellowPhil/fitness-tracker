use std::sync::Arc;

use anyhow::Context;
use async_openai::types::chat::{ChatCompletionTool, ChatCompletionTools, FunctionObjectArgs};

use domain::traits::ExcerciseRepo;
use domain::types::{MuscleGroup, UserId};

use tracing::instrument;

use crate::ai::{dto, format};
use crate::Databases;

use super::constants::{EXERCISE_LIST_TOOL, WORKOUT_QUERY_TOOL};

#[instrument(skip(databases), fields(user_id = user_id.as_i64()), err)]
pub(super) async fn execute_query_workouts(
    databases: Arc<Databases>,
    user_id: UserId,
    arguments_str: &str,
) -> anyhow::Result<String> {
    let arguments = serde_json::from_str::<dto::QueryWorkoutsRequest>(arguments_str)
        .with_context(|| "Invalid arguments for workout query tool")?;

    let date = match arguments.date {
        Some(date) => domain::types::QueryType::OnDate(date),
        None => match arguments.last_n {
            Some(count) => domain::types::QueryType::LastN(count),
            None => domain::types::QueryType::Latest,
        },
    };

    let result = databases
        .gym_app(user_id)
        .query_workout_resource(domain::types::WorkoutQuery {
            date,
            muscle_group: Some(arguments.muscle_group),
        })
        .await
        .with_context(|| "Failed to query workouts")?;

    Ok(format::format_workouts(
        &result.workouts,
        &result.excercises,
        Some(arguments.muscle_group),
    ))
}

#[instrument(skip(databases), fields(user_id = user_id.as_i64()), err)]
pub(super) async fn execute_list_exercises(
    databases: Arc<Databases>,
    user_id: UserId,
    arguments_str: &str,
) -> anyhow::Result<String> {
    let arguments = serde_json::from_str::<dto::ListExercisesRequest>(arguments_str)
        .with_context(|| "Invalid arguments for exercise list tool")?;

    let result = databases
        .exercise_db
        .for_user(user_id)
        .get_by_muscle_group(arguments.muscle_group)
        .await
        .with_context(|| "Failed to query exercises")?;

    let metadata = result
        .iter()
        .map(domain::types::Exercise::metadata)
        .collect::<Vec<_>>();

    Ok(format::format_exercises(
        &metadata,
        Some(arguments.muscle_group),
    ))
}

pub(super) fn exercise_query_tool() -> ChatCompletionTools {
    let muscle_groups = MuscleGroup::all()
        .map(|group| group.to_string())
        .collect::<Vec<_>>();

    ChatCompletionTools::Function(ChatCompletionTool {
        function: FunctionObjectArgs::default()
            .name(EXERCISE_LIST_TOOL)
            .description("Query existing exercises by muscle group.")
            .parameters(serde_json::json!({
                "type": "object",
                "properties": {
                    "muscle_group": {
                        "type": "string",
                        "enum": muscle_groups,
                        "description": "Muscle group filter."
                    },
                },
                "required": ["muscle_group"],
                "additionalProperties": false,
            }))
            .strict(true)
            .build()
            .unwrap(),
    })
}

pub(super) fn workout_query_tool() -> ChatCompletionTools {
    let muscle_groups = MuscleGroup::all()
        .map(|group| group.to_string())
        .collect::<Vec<_>>();

    ChatCompletionTools::Function(ChatCompletionTool {
        function: FunctionObjectArgs::default()
            .name(WORKOUT_QUERY_TOOL)
            .description("Query workouts by date, recent count, and optional muscle group. If both `date` and `last_n` are omitted, returns the latest workout.")
            .parameters(serde_json::json!({
                "type": "object",
                "properties": {
                    "date": {
                        "type": "string",
                        "format": "date",
                        "description": "Workout date in ISO 8601 (YYYY-MM-DD) format. Mutually exclusive with `last_n`."
                    },
                    "last_n": {
                        "type": "integer",
                        "minimum": 1,
                        "description": "Return the most recent N workouts. Mutually exclusive with `date`."
                    },
                    "muscle_group": {
                        "type": "string",
                        "enum": muscle_groups,
                        "description": "Muscle group filter."
                    },
                },
                "required": ["muscle_group"],
                "additionalProperties": false,
            }))
            .strict(false)
            .build()
            .unwrap(),
    })
}

pub(super) fn health_query_tool() -> ChatCompletionTools {
    ChatCompletionTools::Function(ChatCompletionTool {
        function: FunctionObjectArgs::default()
            .name("health_query")
            .description("Query users health information that includes weight, height, and age.")
            .parameters(serde_json::json!({
                "type": "object",
                "properties": { },
                "required": [],
                "additionalProperties": false,
            }))
            .build()
            .unwrap(),
    })
}
