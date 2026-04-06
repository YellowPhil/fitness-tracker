use std::str::FromStr;

use domain::excercise::MuscleGroup;
use serde::Deserialize;

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct QueryWorkoutsRequest {
    #[serde(default)]
    pub date: Option<time::Date>,
    #[serde(default)]
    pub last_n: Option<usize>,
    #[serde(deserialize_with = "deserialize_muscle_group")]
    pub muscle_group: MuscleGroup,
}

#[derive(serde::Deserialize)]
pub(super) enum WorkoutQuery {
    Last(usize),
    Date(#[serde(deserialize_with = "deserialize_date")] time::Date),
}

#[derive(serde::Deserialize)]
pub(super) struct ListExercisesRequest {
    #[serde(deserialize_with = "deserialize_muscle_group")]
    pub muscle_group: MuscleGroup,
}

fn deserialize_optional_muscle_group<'de, D>(
    deserializer: D,
) -> Result<Option<MuscleGroup>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<&str>::deserialize(deserializer)?;
    value
        .map(|value| MuscleGroup::from_str(value).map_err(serde::de::Error::custom))
        .transpose()
}

fn deserialize_muscle_group<'de, D>(deserializer: D) -> Result<MuscleGroup, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = <&str>::deserialize(deserializer)?;
    MuscleGroup::from_str(value).map_err(serde::de::Error::custom)
}

fn deserialize_date<'de, D>(deserializer: D) -> Result<time::Date, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let input = <&str>::deserialize(deserializer)?;

    let format = time::format_description::parse_borrowed::<2>("[year]-[month]-[day]")
        .map_err(serde::de::Error::custom)?;
    time::Date::parse(input, &format).map_err(serde::de::Error::custom)
}

/// Parsed structured output from the model (matches `workout_response_schema`).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AiWorkoutResponse {
    pub workout_name: Option<String>,
    pub exercises: Vec<AiExerciseEntry>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AiExerciseEntry {
    pub exercise_name: String,
    pub notes: Option<String>,
    pub sets: Vec<AiSetEntry>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AiSetEntry {
    pub reps: u32,
    pub weight_kg: Option<f64>,
}
