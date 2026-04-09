use crate::types::units::Weight;

#[derive(Debug, Clone, PartialEq)]
pub enum LoadType {
    Weighted(Weight),
    BodyWeight,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PerformedSet {
    pub kind: LoadType,
    pub reps: u32,
}
