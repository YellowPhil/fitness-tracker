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
