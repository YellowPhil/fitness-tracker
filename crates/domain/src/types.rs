#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::Display)]
pub enum WeightUnits {
    #[strum(serialize = "kg")]
    Kilograms,
    #[strum(serialize = "lbs")]
    Pounds,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Weight {
    pub value: f64,
    pub units: WeightUnits,
}

impl Weight {
    pub fn new(value: f64, units: WeightUnits) -> Self {
        Self { value, units }
    }
}
