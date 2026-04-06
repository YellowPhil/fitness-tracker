use time::OffsetDateTime;

use crate::excercise::{ExerciseId, PerformedSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorkoutId(uuid::Uuid);

/// How the workout was created (manual log vs AI-generated plan).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WorkoutSource {
    Manual,
    AiGenerated,
}

/// Canonical wire strings for [`WorkoutSource`] (JSON API and Postgres `workout_source` enum).
/// Change values only here and in the DB migration that defines the enum.
pub mod workout_source {
    pub const MANUAL: &str = "manual";
    pub const AI_GENERATED: &str = "ai_generated";
}

impl WorkoutSource {
    /// JSON `WorkoutResponse.source` and Postgres `workout_source` enum label.
    #[must_use]
    pub const fn as_api_str(self) -> &'static str {
        match self {
            Self::Manual => workout_source::MANUAL,
            Self::AiGenerated => workout_source::AI_GENERATED,
        }
    }

    pub fn parse_api_str(s: &str) -> Option<Self> {
        match s {
            workout_source::MANUAL => Some(Self::Manual),
            workout_source::AI_GENERATED => Some(Self::AiGenerated),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Workout {
    pub id: WorkoutId,
    pub name: Option<String>,
    pub start_date: OffsetDateTime,
    pub end_date: Option<OffsetDateTime>,
    pub entries: Vec<WorkoutExercise>,
    pub source: WorkoutSource,
}

#[derive(Debug, Clone)]
pub struct WorkoutExercise {
    pub exercise_id: ExerciseId,
    pub sets: Vec<PerformedSet>,
    pub notes: Option<String>,
}

impl Workout {
    pub fn new(name: Option<String>) -> Self {
        let date = OffsetDateTime::now_utc();
        Self {
            id: WorkoutId::new(),
            name,
            start_date: date,
            end_date: None,
            entries: vec![],
            source: WorkoutSource::Manual,
        }
    }

    /// New workout from an AI-generated plan, not yet started (`end_date` unset).
    pub fn ai_generated(
        name: Option<String>,
        start_date: OffsetDateTime,
        entries: Vec<WorkoutExercise>,
    ) -> Self {
        Self {
            id: WorkoutId::new(),
            name,
            start_date,
            end_date: None,
            entries,
            source: WorkoutSource::AiGenerated,
        }
    }
}

impl WorkoutId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }

    pub fn from_uuid(value: uuid::Uuid) -> Self {
        Self(value)
    }

    pub fn as_uuid(&self) -> &uuid::Uuid {
        &self.0
    }
}

impl WorkoutExercise {
    pub fn new(exercise_id: ExerciseId) -> Self {
        Self {
            exercise_id,
            sets: vec![],
            notes: None,
        }
    }
    pub fn add_set(&mut self, set: PerformedSet) {
        self.sets.push(set);
    }
}

#[cfg(test)]
mod tests {
    use super::{WorkoutSource, workout_source};

    #[test]
    fn workout_source_api_str_roundtrip() {
        for s in [WorkoutSource::Manual, WorkoutSource::AiGenerated] {
            assert_eq!(WorkoutSource::parse_api_str(s.as_api_str()), Some(s));
        }
        assert_eq!(
            WorkoutSource::Manual.as_api_str(),
            workout_source::MANUAL
        );
        assert_eq!(
            WorkoutSource::AiGenerated.as_api_str(),
            workout_source::AI_GENERATED
        );
    }
}
