#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::Display)]
pub enum MuscleGroup {
    #[strum(serialize = "Chest")]
    Chest,
    #[strum(serialize = "Back")]
    Back,
    #[strum(serialize = "Arms")]
    Arms,
    #[strum(serialize = "Legs")]
    Legs,
    #[strum(serialize = "Core")]
    Core,
}
