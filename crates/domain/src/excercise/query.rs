use time::Date;

use super::MuscleGroup;

#[derive(Debug, Clone)]
pub enum WorkoutDateQuery {
    OnDate(Date),
    LastN(usize),
    Latest,
}

#[derive(Debug, Clone)]
pub struct WorkoutQuery {
    pub date: WorkoutDateQuery,
    pub muscle_group: Option<MuscleGroup>,
}
