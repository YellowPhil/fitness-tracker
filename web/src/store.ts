import { create } from "zustand";
import { persist } from "zustand/middleware";
import type {
  Exercise,
  ExerciseKind,
  MuscleGroup,
  PerformedSet,
  Workout,
} from "./types";
import { toDateString } from "./utils";

function offsetDate(days: number): string {
  const d = new Date();
  d.setDate(d.getDate() + days);
  return toDateString(d);
}

const DEFAULT_EXERCISES: Exercise[] = [
  { id: "ex-bench-press", name: "Bench Press", kind: "Weighted", muscleGroup: "Chest", source: "BuiltIn" },
  { id: "ex-incline-db", name: "Incline Dumbbell Press", kind: "Weighted", muscleGroup: "Chest", source: "BuiltIn" },
  { id: "ex-cable-fly", name: "Cable Fly", kind: "Weighted", muscleGroup: "Chest", source: "BuiltIn" },
  { id: "ex-push-up", name: "Push Up", kind: "BodyWeight", muscleGroup: "Chest", source: "BuiltIn" },
  { id: "ex-deadlift", name: "Deadlift", kind: "Weighted", muscleGroup: "Back", source: "BuiltIn" },
  { id: "ex-barbell-row", name: "Barbell Row", kind: "Weighted", muscleGroup: "Back", source: "BuiltIn" },
  { id: "ex-pull-up", name: "Pull Up", kind: "BodyWeight", muscleGroup: "Back", source: "BuiltIn" },
  { id: "ex-lat-pulldown", name: "Lat Pulldown", kind: "Weighted", muscleGroup: "Back", source: "BuiltIn" },
  { id: "ex-squat", name: "Squat", kind: "Weighted", muscleGroup: "Legs", source: "BuiltIn" },
  { id: "ex-leg-press", name: "Leg Press", kind: "Weighted", muscleGroup: "Legs", source: "BuiltIn" },
  { id: "ex-rdl", name: "Romanian Deadlift", kind: "Weighted", muscleGroup: "Legs", source: "BuiltIn" },
  { id: "ex-leg-curl", name: "Leg Curl", kind: "Weighted", muscleGroup: "Legs", source: "BuiltIn" },
  { id: "ex-ohp", name: "Overhead Press", kind: "Weighted", muscleGroup: "Arms", source: "BuiltIn" },
  { id: "ex-bicep-curl", name: "Bicep Curl", kind: "Weighted", muscleGroup: "Arms", source: "BuiltIn" },
  { id: "ex-tricep-push", name: "Tricep Pushdown", kind: "Weighted", muscleGroup: "Arms", source: "BuiltIn" },
  { id: "ex-lateral-raise", name: "Lateral Raise", kind: "Weighted", muscleGroup: "Arms", source: "BuiltIn" },
  { id: "ex-plank", name: "Plank", kind: "BodyWeight", muscleGroup: "Core", source: "BuiltIn" },
  { id: "ex-cable-crunch", name: "Cable Crunch", kind: "Weighted", muscleGroup: "Core", source: "BuiltIn" },
  { id: "ex-leg-raise", name: "Hanging Leg Raise", kind: "BodyWeight", muscleGroup: "Core", source: "BuiltIn" },
];

const DEFAULT_WORKOUTS: Workout[] = [
  {
    id: "demo-1",
    name: "Push Day",
    startDate: offsetDate(0),
    entries: [
      {
        exerciseId: "ex-bench-press",
        sets: [
          { kind: { type: "Weighted", weight: { value: 80, units: "kg" } }, reps: 10 },
          { kind: { type: "Weighted", weight: { value: 85, units: "kg" } }, reps: 8 },
          { kind: { type: "Weighted", weight: { value: 85, units: "kg" } }, reps: 7 },
        ],
      },
      {
        exerciseId: "ex-ohp",
        sets: [
          { kind: { type: "Weighted", weight: { value: 40, units: "kg" } }, reps: 12 },
          { kind: { type: "Weighted", weight: { value: 45, units: "kg" } }, reps: 10 },
        ],
      },
      {
        exerciseId: "ex-lateral-raise",
        sets: [
          { kind: { type: "Weighted", weight: { value: 12, units: "kg" } }, reps: 15 },
          { kind: { type: "Weighted", weight: { value: 12, units: "kg" } }, reps: 14 },
          { kind: { type: "Weighted", weight: { value: 12, units: "kg" } }, reps: 12 },
        ],
      },
    ],
  },
  {
    id: "demo-2",
    name: "Pull Day",
    startDate: offsetDate(-3),
    entries: [
      {
        exerciseId: "ex-deadlift",
        sets: [
          { kind: { type: "Weighted", weight: { value: 120, units: "kg" } }, reps: 5 },
          { kind: { type: "Weighted", weight: { value: 130, units: "kg" } }, reps: 5 },
          { kind: { type: "Weighted", weight: { value: 140, units: "kg" } }, reps: 3 },
        ],
      },
      {
        exerciseId: "ex-barbell-row",
        sets: [
          { kind: { type: "Weighted", weight: { value: 70, units: "kg" } }, reps: 10 },
          { kind: { type: "Weighted", weight: { value: 70, units: "kg" } }, reps: 10 },
          { kind: { type: "Weighted", weight: { value: 70, units: "kg" } }, reps: 8 },
        ],
      },
      {
        exerciseId: "ex-pull-up",
        sets: [
          { kind: { type: "BodyWeight" }, reps: 12 },
          { kind: { type: "BodyWeight" }, reps: 10 },
          { kind: { type: "BodyWeight" }, reps: 8 },
        ],
      },
    ],
  },
  {
    id: "demo-3",
    name: "Leg Day",
    startDate: offsetDate(-5),
    entries: [
      {
        exerciseId: "ex-squat",
        sets: [
          { kind: { type: "Weighted", weight: { value: 100, units: "kg" } }, reps: 8 },
          { kind: { type: "Weighted", weight: { value: 110, units: "kg" } }, reps: 6 },
          { kind: { type: "Weighted", weight: { value: 110, units: "kg" } }, reps: 5 },
        ],
      },
      {
        exerciseId: "ex-leg-press",
        sets: [
          { kind: { type: "Weighted", weight: { value: 180, units: "kg" } }, reps: 12 },
          { kind: { type: "Weighted", weight: { value: 200, units: "kg" } }, reps: 10 },
        ],
      },
      {
        exerciseId: "ex-rdl",
        sets: [
          { kind: { type: "Weighted", weight: { value: 80, units: "kg" } }, reps: 10 },
          { kind: { type: "Weighted", weight: { value: 80, units: "kg" } }, reps: 10 },
        ],
      },
    ],
  },
];

interface GymStore {
  exercises: Exercise[];
  workouts: Workout[];
  selectedDate: string;
  currentView: "calendar" | "exercises";

  setSelectedDate: (date: string) => void;
  setCurrentView: (view: "calendar" | "exercises") => void;

  addExercise: (
    name: string,
    kind: ExerciseKind,
    muscleGroup: MuscleGroup,
  ) => void;
  deleteExercise: (id: string) => void;

  createWorkout: (date: string, name?: string) => void;
  deleteWorkout: (id: string) => void;
  updateWorkoutName: (id: string, name: string) => void;

  addExerciseToWorkout: (workoutId: string, exerciseId: string) => void;
  removeExerciseFromWorkout: (workoutId: string, entryIndex: number) => void;

  addSet: (workoutId: string, entryIndex: number, set: PerformedSet) => void;
  removeSet: (
    workoutId: string,
    entryIndex: number,
    setIndex: number,
  ) => void;
}

export const useStore = create<GymStore>()(
  persist(
    (set) => ({
      exercises: DEFAULT_EXERCISES,
      workouts: DEFAULT_WORKOUTS,
      selectedDate: toDateString(new Date()),
      currentView: "calendar" as const,

      setSelectedDate: (date) => set({ selectedDate: date }),
      setCurrentView: (view) => set({ currentView: view }),

      addExercise: (name, kind, muscleGroup) =>
        set((s) => ({
          exercises: [
            ...s.exercises,
            {
              id: crypto.randomUUID(),
              name,
              kind,
              muscleGroup,
              source: "UserDefined" as const,
            },
          ],
        })),

      deleteExercise: (id) =>
        set((s) => ({
          exercises: s.exercises.filter((e) => e.id !== id),
          workouts: s.workouts.map((w) => ({
            ...w,
            entries: w.entries.filter((e) => e.exerciseId !== id),
          })),
        })),

      createWorkout: (date, name) =>
        set((s) => ({
          workouts: [
            ...s.workouts,
            {
              id: crypto.randomUUID(),
              name,
              startDate: date,
              entries: [],
            },
          ],
        })),

      deleteWorkout: (id) =>
        set((s) => ({
          workouts: s.workouts.filter((w) => w.id !== id),
        })),

      updateWorkoutName: (id, name) =>
        set((s) => ({
          workouts: s.workouts.map((w) =>
            w.id === id ? { ...w, name } : w,
          ),
        })),

      addExerciseToWorkout: (workoutId, exerciseId) =>
        set((s) => ({
          workouts: s.workouts.map((w) =>
            w.id === workoutId
              ? { ...w, entries: [...w.entries, { exerciseId, sets: [] }] }
              : w,
          ),
        })),

      removeExerciseFromWorkout: (workoutId, entryIndex) =>
        set((s) => ({
          workouts: s.workouts.map((w) =>
            w.id === workoutId
              ? { ...w, entries: w.entries.filter((_, i) => i !== entryIndex) }
              : w,
          ),
        })),

      addSet: (workoutId, entryIndex, newSet) =>
        set((s) => ({
          workouts: s.workouts.map((w) =>
            w.id === workoutId
              ? {
                  ...w,
                  entries: w.entries.map((e, i) =>
                    i === entryIndex
                      ? { ...e, sets: [...e.sets, newSet] }
                      : e,
                  ),
                }
              : w,
          ),
        })),

      removeSet: (workoutId, entryIndex, setIndex) =>
        set((s) => ({
          workouts: s.workouts.map((w) =>
            w.id === workoutId
              ? {
                  ...w,
                  entries: w.entries.map((e, i) =>
                    i === entryIndex
                      ? { ...e, sets: e.sets.filter((_, si) => si !== setIndex) }
                      : e,
                  ),
                }
              : w,
          ),
        })),
    }),
    { name: "gym-tracker-storage" },
  ),
);
