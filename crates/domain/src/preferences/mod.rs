#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WorkoutPreferences {
    pub max_sets_per_exercise: Option<u8>,
    pub preferred_split: Option<WorkoutSplit>,
    pub training_goal: Option<TrainingGoal>,
    pub session_duration_minutes: Option<u16>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WorkoutSplit {
    FullBody,
    PushPullLegs,
    UpperLower,
}

impl WorkoutSplit {
    #[must_use]
    pub const fn as_api_str(self) -> &'static str {
        match self {
            Self::FullBody => "FullBody",
            Self::PushPullLegs => "PushPullLegs",
            Self::UpperLower => "UpperLower",
        }
    }

    pub fn parse_api_str(value: &str) -> Option<Self> {
        let normalized = value
            .chars()
            .filter(|ch| !ch.is_ascii_whitespace() && *ch != '_' && *ch != '-')
            .flat_map(char::to_lowercase)
            .collect::<String>();

        match normalized.as_str() {
            "fullbody" => Some(Self::FullBody),
            "pushpulllegs" | "ppl" => Some(Self::PushPullLegs),
            "upperlower" => Some(Self::UpperLower),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TrainingGoal {
    Strength,
    Hypertrophy,
    Endurance,
}

impl TrainingGoal {
    #[must_use]
    pub const fn as_api_str(self) -> &'static str {
        match self {
            Self::Strength => "Strength",
            Self::Hypertrophy => "Hypertrophy",
            Self::Endurance => "Endurance",
        }
    }

    pub fn parse_api_str(value: &str) -> Option<Self> {
        let normalized = value
            .chars()
            .filter(|ch| !ch.is_ascii_whitespace() && *ch != '_' && *ch != '-')
            .flat_map(char::to_lowercase)
            .collect::<String>();

        match normalized.as_str() {
            "strength" => Some(Self::Strength),
            "hypertrophy" | "musclegain" => Some(Self::Hypertrophy),
            "endurance" => Some(Self::Endurance),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{TrainingGoal, WorkoutSplit};

    #[test]
    fn workout_split_parse_accepts_common_formats() {
        assert_eq!(
            WorkoutSplit::parse_api_str("FullBody"),
            Some(WorkoutSplit::FullBody)
        );
        assert_eq!(
            WorkoutSplit::parse_api_str("push_pull_legs"),
            Some(WorkoutSplit::PushPullLegs)
        );
        assert_eq!(
            WorkoutSplit::parse_api_str("upper-lower"),
            Some(WorkoutSplit::UpperLower)
        );
        assert_eq!(WorkoutSplit::parse_api_str("unknown"), None);
    }

    #[test]
    fn training_goal_parse_accepts_common_formats() {
        assert_eq!(
            TrainingGoal::parse_api_str("Strength"),
            Some(TrainingGoal::Strength)
        );
        assert_eq!(
            TrainingGoal::parse_api_str("muscle_gain"),
            Some(TrainingGoal::Hypertrophy)
        );
        assert_eq!(
            TrainingGoal::parse_api_str("endurance"),
            Some(TrainingGoal::Endurance)
        );
        assert_eq!(TrainingGoal::parse_api_str("unknown"), None);
    }
}
