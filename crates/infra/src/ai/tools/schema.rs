pub(super) fn workout_response_schema(
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
