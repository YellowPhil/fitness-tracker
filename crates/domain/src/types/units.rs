use std::ops::Deref;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UserId(i64);

impl UserId {
    pub fn new(id: i64) -> Self {
        Self(id)
    }

    pub fn as_i64(&self) -> i64 {
        self.0
    }
}

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl Deref for UserId {
    type Target = i64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::Display)]
pub enum WeightUnits {
    #[strum(serialize = "kg")]
    Kilograms,
    #[strum(serialize = "lbs")]
    Pounds,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::Display)]
pub enum HeightUnits {
    #[strum(serialize = "cm")]
    Centimeters,
    #[strum(serialize = "in")]
    Inches,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Weight {
    pub value: f64,
    pub units: WeightUnits,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Height {
    pub value: f64,
    pub units: HeightUnits,
}

impl Weight {
    pub fn new(value: f64, units: WeightUnits) -> Self {
        Self { value, units }
    }
}

impl Height {
    pub fn new(value: f64, units: HeightUnits) -> Self {
        Self { value, units }
    }
}
