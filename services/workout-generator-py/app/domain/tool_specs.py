from __future__ import annotations


def build_workout_query_tool(allowed_muscle_groups: list[str]) -> dict:
    return {
        "type": "function",
        "function": {
            "name": "workout_query",
            "description": (
                "Query workouts by date, recent count, and optional muscle group. "
                "If both `date` and `last_n` are omitted, returns the latest workout."
            ),
            "parameters": {
                "type": "object",
                "properties": {
                    "date": {
                        "type": "string",
                        "format": "date",
                        "description": "Workout date in ISO 8601 (YYYY-MM-DD). Mutually exclusive with `last_n`.",
                    },
                    "last_n": {
                        "type": "integer",
                        "minimum": 1,
                        "description": "Return the most recent N workouts. Mutually exclusive with `date`.",
                    },
                    "muscle_group": {
                        "type": "string",
                        "enum": allowed_muscle_groups,
                        "description": "Muscle group filter.",
                    },
                },
                "required": ["muscle_group"],
                "additionalProperties": False,
            },
            "strict": False,
        },
    }


def build_exercise_list_tool(allowed_muscle_groups: list[str]) -> dict:
    return {
        "type": "function",
        "function": {
            "name": "exercise_list",
            "description": "Query existing exercises by muscle group.",
            "parameters": {
                "type": "object",
                "properties": {
                    "muscle_group": {
                        "type": "string",
                        "enum": allowed_muscle_groups,
                        "description": "Muscle group filter.",
                    }
                },
                "required": ["muscle_group"],
                "additionalProperties": False,
            },
            "strict": True,
        },
    }
