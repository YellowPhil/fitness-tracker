use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef};

use crate::excercise::{ExcerciseId, ExcerciseKind, ExcerciseSource, MuscleGroup, WorkoutId};
use crate::types::{HeightUnits, UserId, WeightUnits};

impl ToSql for ExcerciseId {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(self.as_uuid().to_string().into())
    }
}

impl FromSql for ExcerciseId {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let s = value.as_str()?;
        let uuid = uuid::Uuid::parse_str(s).map_err(|e| FromSqlError::Other(Box::new(e)))?;
        Ok(ExcerciseId::from_uuid(uuid))
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

impl ToSql for ExcerciseKind {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        let v: i64 = match self {
            ExcerciseKind::Weighted => 1,
            ExcerciseKind::BodyWeight => 2,
        };
        Ok(v.into())
    }
}

impl FromSql for ExcerciseKind {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value.as_i64()? {
            1 => Ok(ExcerciseKind::Weighted),
            2 => Ok(ExcerciseKind::BodyWeight),
            other => Err(FromSqlError::OutOfRange(other)),
        }
    }
}

impl ToSql for ExcerciseSource {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        let v: i64 = match self {
            ExcerciseSource::BuiltIn => 1,
            ExcerciseSource::UserDefined => 2,
        };
        Ok(v.into())
    }
}

impl FromSql for ExcerciseSource {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value.as_i64()? {
            1 => Ok(ExcerciseSource::BuiltIn),
            2 => Ok(ExcerciseSource::UserDefined),
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
