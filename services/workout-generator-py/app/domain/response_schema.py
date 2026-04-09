from __future__ import annotations


def workout_response_schema(exercise_names: list[str], max_exercise_count: int) -> dict:
    return {
        "type": "object",
        "properties": {
            "workout_name": {"anyOf": [{"type": "string"}, {"type": "null"}]},
            "exercises": {
                "type": "array",
                "maxItems": max_exercise_count,
                "items": {
                    "type": "object",
                    "properties": {
                        "exercise_name": {"type": "string", "enum": exercise_names},
                        "notes": {"anyOf": [{"type": "string"}, {"type": "null"}]},
                        "sets": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "reps": {"type": "integer", "minimum": 1},
                                    "weight_kg": {
                                        "anyOf": [{"type": "number"}, {"type": "null"}]
                                    },
                                },
                                "required": ["reps", "weight_kg"],
                                "additionalProperties": False,
                            },
                        },
                    },
                    "required": ["exercise_name", "sets", "notes"],
                    "additionalProperties": False,
                },
            },
        },
        "required": ["workout_name", "exercises"],
        "additionalProperties": False,
    }
