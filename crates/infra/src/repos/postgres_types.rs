use domain::{
    excercise::{ExerciseKind, ExerciseSource, MuscleGroup},
    types::{HeightUnits, WeightUnits},
};
use postgres_types::{FromSql, ToSql};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ToSql, FromSql)]
#[postgres(name = "exercise_kind")]
pub(crate) enum PgExerciseKind {
    #[postgres(name = "weighted")]
    Weighted,
    #[postgres(name = "bodyweight")]
    BodyWeight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ToSql, FromSql)]
#[postgres(name = "exercise_source")]
pub(crate) enum PgExerciseSource {
    #[postgres(name = "built_in")]
    BuiltIn,
    #[postgres(name = "user_defined")]
    UserDefined,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ToSql, FromSql)]
#[postgres(name = "muscle_group")]
pub(crate) enum PgMuscleGroup {
    #[postgres(name = "chest")]
    Chest,
    #[postgres(name = "back")]
    Back,
    #[postgres(name = "shoulders")]
    Shoulders,
    #[postgres(name = "arms")]
    Arms,
    #[postgres(name = "legs")]
    Legs,
    #[postgres(name = "core")]
    Core,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ToSql, FromSql)]
#[postgres(name = "weight_unit")]
pub(crate) enum PgWeightUnits {
    #[postgres(name = "kg")]
    Kilograms,
    #[postgres(name = "lbs")]
    Pounds,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ToSql, FromSql)]
#[postgres(name = "height_unit")]
pub(crate) enum PgHeightUnits {
    #[postgres(name = "cm")]
    Centimeters,
    #[postgres(name = "in")]
    Inches,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ToSql, FromSql)]
#[postgres(name = "load_type")]
pub(crate) enum PgLoadType {
    #[postgres(name = "weighted")]
    Weighted,
    #[postgres(name = "bodyweight")]
    BodyWeight,
}

impl From<ExerciseKind> for PgExerciseKind {
    fn from(value: ExerciseKind) -> Self {
        match value {
            ExerciseKind::Weighted => Self::Weighted,
            ExerciseKind::BodyWeight => Self::BodyWeight,
        }
    }
}

impl From<PgExerciseKind> for ExerciseKind {
    fn from(value: PgExerciseKind) -> Self {
        match value {
            PgExerciseKind::Weighted => Self::Weighted,
            PgExerciseKind::BodyWeight => Self::BodyWeight,
        }
    }
}

impl From<ExerciseSource> for PgExerciseSource {
    fn from(value: ExerciseSource) -> Self {
        match value {
            ExerciseSource::BuiltIn => Self::BuiltIn,
            ExerciseSource::UserDefined => Self::UserDefined,
        }
    }
}

impl From<PgExerciseSource> for ExerciseSource {
    fn from(value: PgExerciseSource) -> Self {
        match value {
            PgExerciseSource::BuiltIn => Self::BuiltIn,
            PgExerciseSource::UserDefined => Self::UserDefined,
        }
    }
}

impl From<MuscleGroup> for PgMuscleGroup {
    fn from(value: MuscleGroup) -> Self {
        match value {
            MuscleGroup::Chest => Self::Chest,
            MuscleGroup::Back => Self::Back,
            MuscleGroup::Shoulders => Self::Shoulders,
            MuscleGroup::Arms => Self::Arms,
            MuscleGroup::Legs => Self::Legs,
            MuscleGroup::Core => Self::Core,
        }
    }
}

impl From<PgMuscleGroup> for MuscleGroup {
    fn from(value: PgMuscleGroup) -> Self {
        match value {
            PgMuscleGroup::Chest => Self::Chest,
            PgMuscleGroup::Back => Self::Back,
            PgMuscleGroup::Shoulders => Self::Shoulders,
            PgMuscleGroup::Arms => Self::Arms,
            PgMuscleGroup::Legs => Self::Legs,
            PgMuscleGroup::Core => Self::Core,
        }
    }
}

impl From<WeightUnits> for PgWeightUnits {
    fn from(value: WeightUnits) -> Self {
        match value {
            WeightUnits::Kilograms => Self::Kilograms,
            WeightUnits::Pounds => Self::Pounds,
        }
    }
}

impl From<PgWeightUnits> for WeightUnits {
    fn from(value: PgWeightUnits) -> Self {
        match value {
            PgWeightUnits::Kilograms => Self::Kilograms,
            PgWeightUnits::Pounds => Self::Pounds,
        }
    }
}

impl From<HeightUnits> for PgHeightUnits {
    fn from(value: HeightUnits) -> Self {
        match value {
            HeightUnits::Centimeters => Self::Centimeters,
            HeightUnits::Inches => Self::Inches,
        }
    }
}

impl From<PgHeightUnits> for HeightUnits {
    fn from(value: PgHeightUnits) -> Self {
        match value {
            PgHeightUnits::Centimeters => Self::Centimeters,
            PgHeightUnits::Inches => Self::Inches,
        }
    }
}

pub(crate) fn to_pg_muscle_groups(
    values: &Option<Vec<MuscleGroup>>,
) -> Option<Vec<PgMuscleGroup>> {
    values
        .as_ref()
        .map(|groups| groups.iter().copied().map(Into::into).collect())
}

pub(crate) fn from_pg_muscle_groups(
    values: Option<Vec<PgMuscleGroup>>,
) -> Option<Vec<MuscleGroup>> {
    values.map(|groups| groups.into_iter().map(Into::into).collect())
}
