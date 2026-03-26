#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::Display)]
pub enum WeightUnits {
    #[strum(serialize = "kg")]
    Kilograms,
    #[strum(serialize = "lbs")]
    Pounds,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedLoad {
    pub weight: f64,
    pub units: WeightUnits,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoadType {
    Weighted(WeightedLoad),
    BodyWeight,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PerformedSet {
    pub kind: LoadType,
    pub reps: u32,
}
