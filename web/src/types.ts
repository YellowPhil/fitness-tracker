export type WeightUnits = "kg" | "lbs";
export type MuscleGroup = "Chest" | "Back" | "Shoulders" | "Arms" | "Legs" | "Core";
export type ExerciseSource = "BuiltIn" | "UserDefined";
export type ExerciseKind = "Weighted" | "BodyWeight";

export const MUSCLE_GROUPS: MuscleGroup[] = [
  "Chest",
  "Back",
  "Shoulders",
  "Arms",
  "Legs",
  "Core",
];

export interface Weight {
  value: number;
  units: WeightUnits;
}

export type LoadType =
  | { type: "Weighted"; weight: Weight }
  | { type: "BodyWeight" };

export interface PerformedSet {
  kind: LoadType;
  reps: number;
}

export interface Exercise {
  id: string;
  name: string;
  kind: ExerciseKind;
  muscleGroup: MuscleGroup;
  secondaryMuscleGroups?: MuscleGroup[];
  source: ExerciseSource;
}

export interface WorkoutExercise {
  exerciseId: string;
  sets: PerformedSet[];
  notes?: string;
}

/**
 * Single source of truth for API `WorkoutResponse.source` (must match Rust
 * `domain::excercise::workout_source`).
 */
export const WORKOUT_SOURCE_API = {
  manual: "manual",
  aiGenerated: "ai_generated",
} as const;

export type WorkoutSource =
  (typeof WORKOUT_SOURCE_API)[keyof typeof WORKOUT_SOURCE_API];

export interface Workout {
  id: string;
  name?: string;
  startDate: string;
  endDate?: string;
  /** How the workout was created (manual log vs AI-generated plan). */
  source: WorkoutSource;
  entries: WorkoutExercise[];
}

export type HeightUnits = "cm" | "in";

export interface UserProfile {
  weight: Weight;
  height: { value: number; units: HeightUnits };
  age: number;
}
