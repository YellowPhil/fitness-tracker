use crate::types::{Height, Weight};

pub struct HealthParams {
    pub height: Height,
    pub weight: Weight,
    pub age: u32,
}

impl HealthParams {
    pub fn new(height: Height, weight: Weight, age: u32) -> Self {
        Self {
            height,
            weight,
            age,
        }
    }
}
