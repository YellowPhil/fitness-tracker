use strum::IntoEnumIterator;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    strum::Display,
    strum::EnumString,
    strum::EnumIter,
    strum::AsRefStr,
)]
pub enum MuscleGroup {
    #[strum(serialize = "Chest")]
    Chest,
    #[strum(serialize = "Back")]
    Back,
    #[strum(serialize = "Shoulders")]
    Shoulders,
    #[strum(serialize = "Arms")]
    Arms,
    #[strum(serialize = "Legs")]
    Legs,
    #[strum(serialize = "Core")]
    Core,
}

impl MuscleGroup {
    pub fn all() -> impl Iterator<Item = Self> {
        Self::iter()
    }
}
