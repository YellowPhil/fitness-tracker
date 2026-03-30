export type WeightUnits = "kg" | "lbs";
export type MuscleGroup = "Chest" | "Back" | "Arms" | "Legs" | "Core";
export type ExerciseSource = "BuiltIn" | "UserDefined";
export type ExerciseKind = "Weighted" | "BodyWeight";

export const MUSCLE_GROUPS: MuscleGroup[] = [
  "Chest",
  "Back",
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

export interface Workout {
  id: string;
  name?: string;
  startDate: string;
  endDate?: string;
  entries: WorkoutExercise[];
}
