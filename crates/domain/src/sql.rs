use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef};

use crate::excercise::{ExerciseId, ExerciseKind, ExerciseSource, MuscleGroup, WorkoutId};
use crate::types::{HeightUnits, UserId, WeightUnits};

impl ToSql for ExerciseId {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(self.as_uuid().to_string().into())
    }
}

impl FromSql for ExerciseId {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let s = value.as_str()?;
        let uuid = uuid::Uuid::parse_str(s).map_err(|e| FromSqlError::Other(Box::new(e)))?;
        Ok(ExerciseId::from_uuid(uuid))
    }
}

impl ToSql for WorkoutId {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(self.as_uuid().to_string().into())
    }
}

impl FromSql for WorkoutId {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let s = value.as_str()?;
        let uuid = uuid::Uuid::parse_str(s).map_err(|e| FromSqlError::Other(Box::new(e)))?;
        Ok(WorkoutId::from_uuid(uuid))
    }
}

impl ToSql for UserId {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(self.as_i64().into())
    }
}

impl FromSql for UserId {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value.as_i64().map(UserId::new)
    }
}

impl ToSql for ExerciseKind {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        let v: i64 = match self {
            ExerciseKind::Weighted => 1,
            ExerciseKind::BodyWeight => 2,
        };
        Ok(v.into())
    }
}

impl FromSql for ExerciseKind {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value.as_i64()? {
            1 => Ok(ExerciseKind::Weighted),
            2 => Ok(ExerciseKind::BodyWeight),
            other => Err(FromSqlError::OutOfRange(other)),
        }
    }
}

impl ToSql for ExerciseSource {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        let v: i64 = match self {
            ExerciseSource::BuiltIn => 1,
            ExerciseSource::UserDefined => 2,
        };
        Ok(v.into())
    }
}

impl FromSql for ExerciseSource {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value.as_i64()? {
            1 => Ok(ExerciseSource::BuiltIn),
            2 => Ok(ExerciseSource::UserDefined),
            other => Err(FromSqlError::OutOfRange(other)),
        }
    }
}

impl ToSql for MuscleGroup {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        let v: i64 = match self {
            MuscleGroup::Chest => 1,
            MuscleGroup::Back => 2,
            MuscleGroup::Arms => 3,
            MuscleGroup::Legs => 4,
            MuscleGroup::Core => 5,
            MuscleGroup::Shoulders => 6,
        };
        Ok(v.into())
    }
}

impl FromSql for MuscleGroup {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value.as_i64()? {
            1 => Ok(MuscleGroup::Chest),
            2 => Ok(MuscleGroup::Back),
            3 => Ok(MuscleGroup::Arms),
            4 => Ok(MuscleGroup::Legs),
            5 => Ok(MuscleGroup::Core),
            6 => Ok(MuscleGroup::Shoulders),
            other => Err(FromSqlError::OutOfRange(other)),
        }
    }
}

impl ToSql for WeightUnits {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        let v: i64 = match self {
            WeightUnits::Kilograms => 1,
            WeightUnits::Pounds => 2,
        };
        Ok(v.into())
    }
}

impl FromSql for WeightUnits {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value.as_i64()? {
            1 => Ok(WeightUnits::Kilograms),
            2 => Ok(WeightUnits::Pounds),
            other => Err(FromSqlError::OutOfRange(other)),
        }
    }
}

impl ToSql for HeightUnits {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        let v: i64 = match self {
            HeightUnits::Centimeters => 1,
            HeightUnits::Inches => 2,
        };
        Ok(v.into())
    }
}

impl FromSql for HeightUnits {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value.as_i64()? {
            1 => Ok(HeightUnits::Centimeters),
            2 => Ok(HeightUnits::Inches),
            other => Err(FromSqlError::OutOfRange(other)),
        }
    }
}
