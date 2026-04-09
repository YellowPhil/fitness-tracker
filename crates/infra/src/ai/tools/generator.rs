use std::{collections::HashSet, sync::Arc};

use anyhow::Context;
use async_openai::types::chat::{
    ChatCompletionMessageToolCall, ChatCompletionMessageToolCalls,
    ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage,
    ChatCompletionRequestSystemMessage, ChatCompletionRequestToolMessage,
    ChatCompletionRequestUserMessage, ChatCompletionToolChoiceOption, ResponseFormat,
    ResponseFormatJsonSchema, ToolChoiceOptions,
};

use domain::types::{MuscleGroup, UserId, WorkoutExercise};

use tracing::{debug, info, instrument};

use crate::Databases;
use crate::ai::dto;

use super::constants::{EXERCISE_LIST_TOOL, MAX_TOKENS, MODEL, WORKOUT_QUERY_TOOL};
use super::exercise_data::{
    exercises_by_lowercase_name, load_exercises_for_muscle_groups, sorted_exercise_names,
};
use super::prompt::{build_user_message_content, SYSTEM_PROMPT};
use super::resolve::resolve_workout;
use super::schema::workout_response_schema;
use super::tools::{execute_list_exercises, execute_query_workouts, exercise_query_tool, workout_query_tool};

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

        let initial_request = async_openai::types::chat::CreateChatCompletionRequestArgs::default()
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
        let follow_up_request = async_openai::types::chat::CreateChatCompletionRequestArgs::default()
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

        let parsed: dto::AiWorkoutResponse = serde_json::from_str(content.trim())
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
