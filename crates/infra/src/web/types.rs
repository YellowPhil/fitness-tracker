use std::str::FromStr;

use serde::Deserialize;

use domain::excercise::{ExerciseKind, MuscleGroup};
use domain::types::{HeightUnits, WeightUnits};

/// Maximum byte length for free-form name fields.
pub const MAX_NAME_LEN: usize = 200;

/// A validated name: non-blank and at most [`MAX_NAME_LEN`] bytes.
///
/// Deserializes directly from a JSON string; serde rejects values that fail
/// the length or blank check at deserialization time so handlers never receive
/// invalid data.
#[derive(Deserialize)]
#[serde(try_from = "String")]
pub struct Name(String);

impl TryFrom<String> for Name {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        if s.trim().is_empty() {
            return Err("must not be blank".into());
        }
        if s.len() > MAX_NAME_LEN {
            return Err(format!("too long (max {MAX_NAME_LEN} characters)"));
        }
        Ok(Name(s))
    }
}

impl From<Name> for String {
    fn from(n: Name) -> Self {
        n.0
    }
}

/// Weight units accepted by the API: `"kg"` or `"lbs"`.
#[derive(Deserialize)]
pub enum WeightUnitsReq {
    #[serde(rename = "kg")]
    Kg,
    #[serde(rename = "lbs")]
    Lbs,
}

impl From<WeightUnitsReq> for WeightUnits {
    fn from(u: WeightUnitsReq) -> Self {
        match u {
            WeightUnitsReq::Kg => WeightUnits::Kilograms,
            WeightUnitsReq::Lbs => WeightUnits::Pounds,
        }
    }
}

/// Height units accepted by the API: `"cm"` or `"in"`.
#[derive(Deserialize)]
pub enum HeightUnitsReq {
    #[serde(rename = "cm")]
    Cm,
    #[serde(rename = "in")]
    In,
}

impl From<HeightUnitsReq> for HeightUnits {
    fn from(u: HeightUnitsReq) -> Self {
        match u {
            HeightUnitsReq::Cm => HeightUnits::Centimeters,
            HeightUnitsReq::In => HeightUnits::Inches,
        }
    }
}

/// Exercise kind accepted by the API: `"weighted"` or `"bodyweight"`.
#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExerciseKindReq {
    Weighted,
    Bodyweight,
}

impl From<ExerciseKindReq> for ExerciseKind {
    fn from(k: ExerciseKindReq) -> Self {
        match k {
            ExerciseKindReq::Weighted => ExerciseKind::Weighted,
            ExerciseKindReq::Bodyweight => ExerciseKind::BodyWeight,
        }
    }
}

/// Muscle group accepted by the API: `"Chest"`, `"Back"`, `"Shoulders"`,
/// `"Arms"`, `"Legs"`, or `"Core"` (PascalCase, matching the API response).
#[derive(Deserialize)]
#[serde(try_from = "String")]
pub struct MuscleGroupReq(pub MuscleGroup);

impl TryFrom<String> for MuscleGroupReq {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        MuscleGroup::from_str(&s).map(MuscleGroupReq).map_err(|_| {
            format!(
                "unknown muscle group '{s}' \
                 (expected one of: Chest, Back, Shoulders, Arms, Legs, Core)"
            )
        })
    }
}

impl From<MuscleGroupReq> for MuscleGroup {
    fn from(m: MuscleGroupReq) -> Self {
        m.0
    }
}
