use crate::excercise::MuscleGroup;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExcerciseId(uuid::Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExcerciseSource {
    BuiltIn,
    UserDefined,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExcerciseKind {
    Weighted,
    BodyWeight,
}

#[derive(Debug, Clone)]
pub struct Excercise {
    pub id: ExcerciseId,
    pub name: String,
    pub kind: ExcerciseKind,
    pub muscle_group: MuscleGroup,
    pub secondary_muscle_groups: Option<Vec<MuscleGroup>>,
    pub source: ExcerciseSource,
}

impl Excercise {
    pub fn new(
        name: String,
        muscle_group: MuscleGroup,
        secondary_muscle_groups: Option<Vec<MuscleGroup>>,
        kind: ExcerciseKind,
    ) -> Self {
        Self {
            id: ExcerciseId::new(),
            name,
            kind,
            muscle_group,
            secondary_muscle_groups,
            source: ExcerciseSource::UserDefined,
        }
    }
}

impl ExcerciseId {
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
