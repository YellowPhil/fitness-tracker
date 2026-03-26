use crate::excercise::MuscleGroup;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExcerciseId(uuid::Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExcerciseSource {
    BuiltIn,
    UserDefined,
}

#[derive(Debug, Clone, PartialEq)]
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
            id: ExcerciseId(uuid::Uuid::new_v4()),
            name,
            kind,
            muscle_group,
            secondary_muscle_groups,
            source: ExcerciseSource::BuiltIn,
        }
    }
}
