use crate::types::MuscleGroup;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExerciseId(uuid::Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExerciseSource {
    BuiltIn,
    UserDefined,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExerciseKind {
    Weighted,
    BodyWeight,
}

#[derive(Debug, Clone)]
pub struct Exercise {
    pub id: ExerciseId,
    pub name: String,
    pub kind: ExerciseKind,
    pub muscle_group: MuscleGroup,
    pub secondary_muscle_groups: Option<Vec<MuscleGroup>>,
    pub source: ExerciseSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExerciseMetadata {
    pub id: ExerciseId,
    pub name: String,
    pub muscle_group: MuscleGroup,
    pub secondary_muscle_groups: Option<Vec<MuscleGroup>>,
}

impl Exercise {
    pub fn new(
        name: String,
        muscle_group: MuscleGroup,
        secondary_muscle_groups: Option<Vec<MuscleGroup>>,
        kind: ExerciseKind,
    ) -> Self {
        Self {
            id: ExerciseId::new(),
            name,
            kind,
            muscle_group,
            secondary_muscle_groups,
            source: ExerciseSource::UserDefined,
        }
    }

    pub fn built_in(
        name: String,
        muscle_group: MuscleGroup,
        secondary_muscle_groups: Option<Vec<MuscleGroup>>,
        kind: ExerciseKind,
    ) -> Self {
        Self {
            id: ExerciseId::new(),
            name,
            kind,
            muscle_group,
            secondary_muscle_groups,
            source: ExerciseSource::BuiltIn,
        }
    }

    pub fn metadata(&self) -> ExerciseMetadata {
        ExerciseMetadata {
            id: self.id,
            name: self.name.clone(),
            muscle_group: self.muscle_group,
            secondary_muscle_groups: self.secondary_muscle_groups.clone(),
        }
    }
}

impl ExerciseMetadata {
    pub fn matches_muscle_group(&self, muscle_group: MuscleGroup) -> bool {
        self.muscle_group == muscle_group
            || self
                .secondary_muscle_groups
                .as_ref()
                .is_some_and(|groups| groups.contains(&muscle_group))
    }
}

impl Default for ExerciseId {
    fn default() -> Self {
        Self::new()
    }
}

impl ExerciseId {
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
