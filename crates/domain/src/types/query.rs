use time::Date;

use super::MuscleGroup;

#[derive(Debug, Clone)]
pub enum QueryType {
    OnDate(Date),
    LastN(usize),
    Latest,
}

#[derive(Debug, Clone)]
pub struct WorkoutQuery {
    pub date: QueryType,
    pub muscle_group: Option<MuscleGroup>,
}
