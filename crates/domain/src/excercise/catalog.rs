use super::{Exercise, ExerciseKind, MuscleGroup};

use ExerciseKind::{BodyWeight, Weighted};
use MuscleGroup::{Arms, Back, Chest, Core, Legs, Shoulders};

struct Entry {
    name: &'static str,
    primary: MuscleGroup,
    secondary: &'static [MuscleGroup],
    kind: ExerciseKind,
}

const CATALOG: &[Entry] = &[
    // ── Chest ───────────────────────────────────────────
    Entry {
        name: "Barbell Bench Press",
        primary: Chest,
        secondary: &[Arms],
        kind: Weighted,
    },
    Entry {
        name: "Incline Dumbbell Press",
        primary: Chest,
        secondary: &[Arms, Shoulders],
        kind: Weighted,
    },
    Entry {
        name: "Decline Bench Press",
        primary: Chest,
        secondary: &[Arms],
        kind: Weighted,
    },
    Entry {
        name: "Dumbbell Fly",
        primary: Chest,
        secondary: &[],
        kind: Weighted,
    },
    Entry {
        name: "Cable Crossover",
        primary: Chest,
        secondary: &[],
        kind: Weighted,
    },
    Entry {
        name: "Push-Up",
        primary: Chest,
        secondary: &[Arms, Core],
        kind: BodyWeight,
    },
    Entry {
        name: "Chest Dip",
        primary: Chest,
        secondary: &[Arms, Shoulders],
        kind: BodyWeight,
    },
    // ── Back ────────────────────────────────────────────
    Entry {
        name: "Deadlift",
        primary: Back,
        secondary: &[Legs, Core],
        kind: Weighted,
    },
    Entry {
        name: "Barbell Row",
        primary: Back,
        secondary: &[Arms],
        kind: Weighted,
    },
    Entry {
        name: "Pull-Up",
        primary: Back,
        secondary: &[Arms],
        kind: BodyWeight,
    },
    Entry {
        name: "Lat Pulldown",
        primary: Back,
        secondary: &[Arms],
        kind: Weighted,
    },
    Entry {
        name: "Seated Cable Row",
        primary: Back,
        secondary: &[Arms],
        kind: Weighted,
    },
    Entry {
        name: "T-Bar Row",
        primary: Back,
        secondary: &[Arms],
        kind: Weighted,
    },
    Entry {
        name: "Single-Arm Dumbbell Row",
        primary: Back,
        secondary: &[Arms],
        kind: Weighted,
    },
    // ── Shoulders ───────────────────────────────────────
    Entry {
        name: "Overhead Press",
        primary: Shoulders,
        secondary: &[Arms],
        kind: Weighted,
    },
    Entry {
        name: "Lateral Raise",
        primary: Shoulders,
        secondary: &[],
        kind: Weighted,
    },
    Entry {
        name: "Front Raise",
        primary: Shoulders,
        secondary: &[],
        kind: Weighted,
    },
    Entry {
        name: "Arnold Press",
        primary: Shoulders,
        secondary: &[Arms],
        kind: Weighted,
    },
    Entry {
        name: "Reverse Fly",
        primary: Shoulders,
        secondary: &[Back],
        kind: Weighted,
    },
    Entry {
        name: "Upright Row",
        primary: Shoulders,
        secondary: &[Arms],
        kind: Weighted,
    },
    Entry {
        name: "Face Pull",
        primary: Shoulders,
        secondary: &[Back],
        kind: Weighted,
    },
    // ── Arms ────────────────────────────────────────────
    Entry {
        name: "Barbell Curl",
        primary: Arms,
        secondary: &[],
        kind: Weighted,
    },
    Entry {
        name: "Hammer Curl",
        primary: Arms,
        secondary: &[],
        kind: Weighted,
    },
    Entry {
        name: "Tricep Pushdown",
        primary: Arms,
        secondary: &[],
        kind: Weighted,
    },
    Entry {
        name: "Overhead Tricep Extension",
        primary: Arms,
        secondary: &[],
        kind: Weighted,
    },
    Entry {
        name: "Preacher Curl",
        primary: Arms,
        secondary: &[],
        kind: Weighted,
    },
    Entry {
        name: "Skull Crusher",
        primary: Arms,
        secondary: &[],
        kind: Weighted,
    },
    Entry {
        name: "Concentration Curl",
        primary: Arms,
        secondary: &[],
        kind: Weighted,
    },
    Entry {
        name: "Tricep Dip",
        primary: Arms,
        secondary: &[Chest, Shoulders],
        kind: BodyWeight,
    },
    // ── Legs ────────────────────────────────────────────
    Entry {
        name: "Barbell Squat",
        primary: Legs,
        secondary: &[Core],
        kind: Weighted,
    },
    Entry {
        name: "Leg Press",
        primary: Legs,
        secondary: &[],
        kind: Weighted,
    },
    Entry {
        name: "Romanian Deadlift",
        primary: Legs,
        secondary: &[Back],
        kind: Weighted,
    },
    Entry {
        name: "Leg Extension",
        primary: Legs,
        secondary: &[],
        kind: Weighted,
    },
    Entry {
        name: "Leg Curl",
        primary: Legs,
        secondary: &[],
        kind: Weighted,
    },
    Entry {
        name: "Standing Calf Raise",
        primary: Legs,
        secondary: &[],
        kind: Weighted,
    },
    Entry {
        name: "Lunges",
        primary: Legs,
        secondary: &[Core],
        kind: Weighted,
    },
    Entry {
        name: "Bulgarian Split Squat",
        primary: Legs,
        secondary: &[Core],
        kind: Weighted,
    },
    // ── Core ────────────────────────────────────────────
    Entry {
        name: "Crunch",
        primary: Core,
        secondary: &[],
        kind: BodyWeight,
    },
    Entry {
        name: "Plank",
        primary: Core,
        secondary: &[Shoulders],
        kind: BodyWeight,
    },
    Entry {
        name: "Russian Twist",
        primary: Core,
        secondary: &[],
        kind: BodyWeight,
    },
    Entry {
        name: "Hanging Leg Raise",
        primary: Core,
        secondary: &[],
        kind: BodyWeight,
    },
    Entry {
        name: "Ab Rollout",
        primary: Core,
        secondary: &[Arms, Shoulders],
        kind: BodyWeight,
    },
    Entry {
        name: "Cable Woodchop",
        primary: Core,
        secondary: &[],
        kind: Weighted,
    },
    Entry {
        name: "Dead Bug",
        primary: Core,
        secondary: &[],
        kind: BodyWeight,
    },
];

pub fn built_in_exercises() -> Vec<Exercise> {
    CATALOG
        .iter()
        .map(|e| {
            let secondary = if e.secondary.is_empty() {
                None
            } else {
                Some(e.secondary.to_vec())
            };
            Exercise::built_in(e.name.into(), e.primary, secondary, e.kind)
        })
        .collect()
}
