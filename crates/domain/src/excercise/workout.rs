use time::OffsetDateTime;

use crate::excercise::{ExcerciseId, PerformedSet};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorkoutId(uuid::Uuid);

#[derive(Debug, Clone)]
pub struct Workout {
    pub id: WorkoutId,
    pub name: Option<String>,
    pub start_date: OffsetDateTime,
    pub end_date: Option<OffsetDateTime>,
    pub entries: Vec<WorkoutExercise>,
}

#[derive(Debug, Clone)]
pub struct WorkoutExercise {
    pub excercise_id: ExcerciseId,
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
    pub fn new(excercise_id: ExcerciseId) -> Self {
        Self {
            excercise_id,
            sets: vec![],
            notes: None,
        }
    }
    pub fn add_set(&mut self, set: PerformedSet) {
        self.sets.push(set);
    }
}
