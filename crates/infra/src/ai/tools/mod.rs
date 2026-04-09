use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use anyhow::Context;
use async_openai::types::chat::{
    ChatCompletionMessageToolCall, ChatCompletionMessageToolCalls,
    ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage,
    ChatCompletionRequestSystemMessage, ChatCompletionRequestToolMessage,
    ChatCompletionRequestUserMessage, ChatCompletionTool, ChatCompletionToolChoiceOption,
    ChatCompletionTools, CreateChatCompletionRequestArgs, FunctionObjectArgs, ResponseFormat,
    ResponseFormatJsonSchema, ToolChoiceOptions,
};

use domain::{
    types::{
        Exercise, ExerciseId, ExerciseKind, LoadType, MuscleGroup, PerformedSet, WorkoutExercise,
    },
    traits::ExcerciseRepo,
    types::{UserId, Weight, WeightUnits},
};

use tracing::{debug, info, instrument};

use crate::Databases;

const MODEL: &str = "gpt-5-mini";
const MAX_TOKENS: u32 = 8192;
const WORKOUT_QUERY_TOOL: &str = "workout_query";
const EXERCISE_LIST_TOOL: &str = "exercise_list";

const SYSTEM_PROMPT: &str = include_str!("workout-programmer.md");

/// Result of [`WorkoutGenerator::generate_workout`]: a name and exercises ready to attach to a workout.
#[derive(Debug, Clone)]
pub struct GeneratedWorkout {
    pub name: Option<String>,
    pub exercises: Vec<WorkoutExercise>,
}

pub struct WorkoutGenerator {
    databases: Arc<Databases>,
    user_id: UserId,
    api_key: String,
    known_tools: HashSet<String>,
}

impl WorkoutGenerator {
    pub fn new(databases: Arc<Databases>, user_id: UserId, api_key: String) -> Self {
        Self {
            databases,
            user_id,
            api_key,
            known_tools: HashSet::from([
                WORKOUT_QUERY_TOOL.to_string(),
                EXERCISE_LIST_TOOL.to_string(),
            ]),
        }
    }

    /// Generates a workout plan. `max_exercise_count` is both communicated to the model and enforced
    /// via the response JSON schema (`maxItems` on `exercises`).
    #[instrument(
        skip(self, muscle_groups),
        fields(user_id = self.user_id.as_i64(), date = %date, max_exercises = max_exercise_count),
        err
    )]
    pub async fn generate_workout(
        &self,
        date: time::Date,
        muscle_groups: &[MuscleGroup],
        max_exercise_count: usize,
    ) -> anyhow::Result<GeneratedWorkout> {
        if muscle_groups.is_empty() {
            anyhow::bail!("muscle_groups must not be empty");
        }
        if max_exercise_count == 0 {
            anyhow::bail!("max_exercise_count must be at least 1");
        }

        let loaded_exercises =
            load_exercises_for_muscle_groups(&self.databases, self.user_id, muscle_groups).await?;
        if loaded_exercises.is_empty() {
            anyhow::bail!("No exercises found for the selected muscle groups");
        }
        debug!(
            count = loaded_exercises.len(),
            "exercises loaded for muscle groups"
        );

        let exercise_names_sorted = sorted_exercise_names(&loaded_exercises);
        let exercises_by_name = exercises_by_lowercase_name(&loaded_exercises);

        let user_content = build_user_message_content(
            date,
            muscle_groups,
            &loaded_exercises,
            &exercise_names_sorted,
            max_exercise_count,
        );

        let client = async_openai::Client::with_config(
            async_openai::config::OpenAIConfig::new().with_api_key(&self.api_key),
        );

        let initial_messages: Vec<ChatCompletionRequestMessage> = vec![
            ChatCompletionRequestSystemMessage::from(SYSTEM_PROMPT).into(),
            ChatCompletionRequestUserMessage::from(user_content.as_str()).into(),
        ];

        let initial_request = CreateChatCompletionRequestArgs::default()
            .max_completion_tokens(MAX_TOKENS)
            .model(MODEL)
            .messages(initial_messages.clone())
            .tools(vec![workout_query_tool(), exercise_query_tool()])
            .tool_choice(ChatCompletionToolChoiceOption::Mode(
                ToolChoiceOptions::Required,
            ))
            .build()?;

        debug!("sending initial request to model");
        let response_message = client
            .chat()
            .create(initial_request)
            .await
            .with_context(|| "Failed to generate response from OpenAI")?
            .choices
            .first()
            .with_context(|| "No response from OpenAI")?
            .message
            .clone();

        let follow_up_messages = match response_message.tool_calls {
            None => {
                let assistant_text = response_message.content.clone().unwrap_or_default();
                debug!(
                    content_len = assistant_text.len(),
                    "model skipped tool calls, appending response to follow-up context"
                );
                let assistant_msg: ChatCompletionRequestMessage =
                    ChatCompletionRequestAssistantMessageArgs::default()
                        .content(assistant_text)
                        .build()
                        .expect("assistant message with content is always valid")
                        .into();
                let mut msgs = initial_messages;
                msgs.push(assistant_msg);
                msgs
            }
            Some(ref tool_calls) => {
                debug!(call_count = tool_calls.len(), "model requested tool calls");
                let tool_responses = self.execute_tool_calls(tool_calls).await?;
                if tool_responses.is_empty() {
                    anyhow::bail!("Model requested tools but none could be executed");
                }
                build_follow_up_messages(initial_messages, tool_responses)
            }
        };

        let schema = workout_response_schema(&exercise_names_sorted, max_exercise_count);
        let follow_up_request = CreateChatCompletionRequestArgs::default()
            .max_completion_tokens(MAX_TOKENS)
            .model(MODEL)
            .messages(follow_up_messages)
            .response_format(ResponseFormat::JsonSchema {
                json_schema: ResponseFormatJsonSchema {
                    description: Some(format!(
                        "Structured workout plan: exercise names must match the allowed list exactly; at most {max_exercise_count} exercises."
                    )),
                    name: "workout_plan".into(),
                    schema: Some(schema),
                    strict: Some(true),
                },
            })
            .build()?;

        let follow_up_choice = client
            .chat()
            .create(follow_up_request)
            .await
            .with_context(|| "Failed to generate structured workout from OpenAI")?
            .choices
            .into_iter()
            .next()
            .with_context(|| "No response from OpenAI")?;

        debug!(finish_reason = ?follow_up_choice.finish_reason, "received structured response from model");

        if let Some(refusal) = &follow_up_choice.message.refusal {
            anyhow::bail!("Model refused to generate workout: {refusal}");
        }

        let content = follow_up_choice
            .message
            .content
            .clone()
            .with_context(|| "OpenAI returned no message content and no refusal")?;

        debug!(
            content_preview = &content[..content.len().min(300)],
            "raw structured response from model"
        );

        let parsed: super::dto::AiWorkoutResponse = serde_json::from_str(content.trim())
            .with_context(|| format!("Failed to parse workout JSON from model: {content}"))?;

        let mut parsed = parsed;
        let name = parsed.workout_name.take();
        let workout_exercises = resolve_workout(parsed, &exercises_by_name)?;
        if workout_exercises.len() > max_exercise_count {
            anyhow::bail!(
                "Model returned {} exercises, exceeding max_exercise_count of {}",
                workout_exercises.len(),
                max_exercise_count
            );
        }

        info!(
            name = ?name,
            exercise_count = workout_exercises.len(),
            "workout plan generated"
        );
        Ok(GeneratedWorkout {
            name,
            exercises: workout_exercises,
        })
    }

    /// Iterates over tool calls from the model, dispatches each known tool, and returns
    /// `(tool_call, result)` pairs. Unknown tools receive an error JSON payload so the
    /// conversation history stays valid for the API.
    #[instrument(skip(self, tool_calls), fields(count = tool_calls.len()), err)]
    async fn execute_tool_calls(
        &self,
        tool_calls: &[ChatCompletionMessageToolCalls],
    ) -> anyhow::Result<Vec<(ChatCompletionMessageToolCall, String)>> {
        let mut responses = Vec::new();

        for tool_call_enum in tool_calls {
            if let ChatCompletionMessageToolCalls::Function(tool_call) = tool_call_enum {
                let name = &tool_call.function.name;
                let result = if self.known_tools.contains(name) {
                    debug!(tool = %name, "executing tool call");
                    self.call_tool(name, &tool_call.function.arguments)
                        .await
                        .unwrap_or_else(|e| e.to_string())
                } else {
                    debug!(tool = %name, "unknown tool call, returning error payload");
                    serde_json::json!({ "error": "unknown_tool", "name": name }).to_string()
                };
                responses.push((tool_call.clone(), result));
            }
        }

        Ok(responses)
    }

    #[instrument(skip(self, arguments), fields(tool = tool), err)]
    async fn call_tool(&self, tool: &str, arguments: &str) -> anyhow::Result<String> {
        let databases = Arc::clone(&self.databases);
        let user_id = self.user_id;

        match tool {
            WORKOUT_QUERY_TOOL => execute_query_workouts(databases, user_id, arguments).await,
            EXERCISE_LIST_TOOL => execute_list_exercises(databases, user_id, arguments).await,
            _ => anyhow::bail!("Unknown tool: {}", tool),
        }
    }
}

#[instrument(skip(databases, muscle_groups), fields(user_id = user_id.as_i64()), err)]
async fn load_exercises_for_muscle_groups(
    databases: &Arc<Databases>,
    user_id: UserId,
    muscle_groups: &[MuscleGroup],
) -> anyhow::Result<Vec<Exercise>> {
    let mut by_id: HashMap<ExerciseId, Exercise> = HashMap::new();
    let repo = databases.exercise_db.for_user(user_id);
    for &mg in muscle_groups {
        let list = repo
            .get_by_muscle_group(mg)
            .await
            .with_context(|| format!("Failed to load exercises for muscle group {mg}"))?;
        for e in list {
            by_id.insert(e.id, e);
        }
    }
    Ok(by_id.into_values().collect())
}

fn sorted_exercise_names(exercises: &[Exercise]) -> Vec<String> {
    let mut names: Vec<String> = exercises.iter().map(|e| e.name.clone()).collect();
    names.sort();
    names
}

fn exercises_by_lowercase_name(exercises: &[Exercise]) -> HashMap<String, Exercise> {
    let mut map = HashMap::new();
    for e in exercises {
        let key = e.name.to_lowercase();
        map.entry(key).or_insert_with(|| e.clone());
    }
    map
}

fn build_user_message_content(
    date: time::Date,
    muscle_groups: &[MuscleGroup],
    exercises: &[Exercise],
    exercise_names: &[String],
    max_exercise_count: usize,
) -> String {
    let groups = muscle_groups
        .iter()
        .map(|g| g.to_string())
        .collect::<Vec<_>>()
        .join(", ");

    let mut weighted_names: Vec<&str> = exercises
        .iter()
        .filter(|e| matches!(e.kind, ExerciseKind::Weighted))
        .map(|e| e.name.as_str())
        .collect();
    weighted_names.sort_unstable();

    let mut bodyweight_names: Vec<&str> = exercises
        .iter()
        .filter(|e| matches!(e.kind, ExerciseKind::BodyWeight))
        .map(|e| e.name.as_str())
        .collect();
    bodyweight_names.sort_unstable();

    let weighted_section = if weighted_names.is_empty() {
        String::new()
    } else {
        format!(
            "Weighted exercises (MUST have a non-null weight_kg in every set):\n{}\n\n",
            weighted_names.join("\n")
        )
    };

    let bodyweight_section = if bodyweight_names.is_empty() {
        String::new()
    } else {
        format!(
            "Bodyweight exercises (set weight_kg to null in every set):\n{}\n\n",
            bodyweight_names.join("\n")
        )
    };

    let all_names = exercise_names.join("\n");

    format!(
        "Target workout date: {date}\n\
         Muscle groups: {groups}\n\
         Maximum number of exercises (hard cap): {max_exercise_count}\n\n\
         {weighted_section}\
         {bodyweight_section}\
         Allowed exercise names (use ONLY these exact strings in your final JSON output):\n\
         {all_names}\n\n\
         Rules for weight_kg:\n\
         - Weighted exercises: weight_kg MUST be a positive number (kg). \
           If no historical data is available, prescribe a conservative but realistic working weight \
           (e.g. Deadlift 60–100 kg, Barbell Row 40–70 kg, Bench Press 40–80 kg, Overhead Press 30–60 kg).\n\
         - Bodyweight exercises: weight_kg MUST be null.\n\n\
         Generate a workout plan for the given date. \
         The `exercises` array must contain at most {max_exercise_count} entries. \
         Use the tools to check past workouts before prescribing loads."
    )
}

/// JSON Schema for structured outputs (`strict` mode).
fn workout_response_schema(
    exercise_names: &[String],
    max_exercise_count: usize,
) -> serde_json::Value {
    let enum_names: Vec<serde_json::Value> = exercise_names
        .iter()
        .cloned()
        .map(serde_json::Value::String)
        .collect();

    serde_json::json!({
        "type": "object",
        "properties": {
            "workout_name": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
            "exercises": {
                "type": "array",
                "maxItems": max_exercise_count,
                "items": {
                    "type": "object",
                    "properties": {
                        "exercise_name": {
                            "type": "string",
                            "enum": enum_names
                        },
                        "notes": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
                        "sets": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "reps": { "type": "integer", "minimum": 1 },
                                    "weight_kg": { "anyOf": [{ "type": "number" }, { "type": "null" }] }
                                },
                                "required": ["reps", "weight_kg"],
                                "additionalProperties": false
                            }
                        }
                    },
                    "required": ["exercise_name", "sets", "notes"],
                    "additionalProperties": false
                }
            }
        },
        "required": ["workout_name", "exercises"],
        "additionalProperties": false
    })
}

fn resolve_workout(
    response: super::dto::AiWorkoutResponse,
    exercises_by_name: &HashMap<String, Exercise>,
) -> anyhow::Result<Vec<WorkoutExercise>> {
    let mut out = Vec::with_capacity(response.exercises.len());

    for entry in response.exercises {
        let key = entry.exercise_name.to_lowercase();
        let exercise = exercises_by_name
            .get(&key)
            .with_context(|| format!("Unknown exercise name: {}", entry.exercise_name))?;

        let mut sets = Vec::with_capacity(entry.sets.len());
        for s in entry.sets {
            let kind = match exercise.kind {
                ExerciseKind::BodyWeight => LoadType::BodyWeight,
                ExerciseKind::Weighted => match s.weight_kg {
                    Some(w) if w > 0.0 => {
                        LoadType::Weighted(Weight::new(w, WeightUnits::Kilograms))
                    }
                    _ => LoadType::BodyWeight,
                },
            };

            sets.push(PerformedSet { kind, reps: s.reps });
        }

        out.push(WorkoutExercise {
            exercise_id: exercise.id,
            sets,
            notes: entry.notes,
        });
    }

    Ok(out)
}

/// Constructs the full message history for the follow-up request:
/// `[system, user, assistant_msg_with_tool_calls, ...tool_result_msgs]`
fn build_follow_up_messages(
    prompt_prefix: Vec<ChatCompletionRequestMessage>,
    tool_responses: Vec<(ChatCompletionMessageToolCall, String)>,
) -> Vec<ChatCompletionRequestMessage> {
    let assistant_tool_calls: Vec<ChatCompletionMessageToolCalls> = tool_responses
        .iter()
        .map(|(tc, _)| ChatCompletionMessageToolCalls::Function(tc.clone()))
        .collect();

    let assistant_msg: ChatCompletionRequestMessage =
        ChatCompletionRequestAssistantMessageArgs::default()
            .tool_calls(assistant_tool_calls)
            .build()
            .expect("assistant message with tool_calls is always valid")
            .into();

    let tool_msgs: Vec<ChatCompletionRequestMessage> = tool_responses
        .into_iter()
        .map(|(tc, content)| {
            ChatCompletionRequestMessage::Tool(ChatCompletionRequestToolMessage {
                content: content.into(),
                tool_call_id: tc.id,
            })
        })
        .collect();

    let mut messages = prompt_prefix;
    messages.push(assistant_msg);
    messages.extend(tool_msgs);
    messages
}

#[instrument(skip(databases), fields(user_id = user_id.as_i64()), err)]
async fn execute_query_workouts(
    databases: Arc<Databases>,
    user_id: UserId,
    arguments_str: &str,
) -> anyhow::Result<String> {
    let arguments = serde_json::from_str::<super::dto::QueryWorkoutsRequest>(arguments_str)
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

    Ok(super::format::format_workouts(
        &result.workouts,
        &result.excercises,
        Some(arguments.muscle_group),
    ))
}

#[instrument(skip(databases)  fields(user_id = user_id.as_i64()), err)]
async fn execute_list_exercises(
    databases: Arc<Databases>,
    user_id: UserId,
    arguments_str: &str,
) -> anyhow::Result<String> {
    let arguments = serde_json::from_str::<super::dto::ListExercisesRequest>(arguments_str)
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

    Ok(super::format::format_exercises(
        &metadata,
        Some(arguments.muscle_group),
    ))
}

fn exercise_query_tool() -> ChatCompletionTools {
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

fn workout_query_tool() -> ChatCompletionTools {
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
            .unwrap()
    })
}

fn health_query_tool() -> ChatCompletionTools {
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
