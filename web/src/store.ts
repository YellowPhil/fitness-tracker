import { create } from "zustand";
import { persist } from "zustand/middleware";
import * as api from "./api";
import type {
  Exercise,
  ExerciseKind,
  MuscleGroup,
  PerformedSet,
  UserProfile,
  WeightUnits,
  Workout,
} from "./types";
import { toDateString } from "./utils";

type ViewId = "calendar" | "exercises" | "personal";

interface GymStore {
  exercises: Exercise[];
  workouts: Workout[];
  /** Dates (YYYY-MM-DD) that have at least one workout — for calendar dots */
  calendarWorkoutDates: string[];
  selectedDate: string;
  currentView: ViewId;
  calendarViewYear: number;
  calendarViewMonth: number;

  profile: UserProfile | null;
  profileLoading: boolean;

  syncError: string | null;
  isLoading: boolean;

  bootstrap: () => Promise<void>;
  clearSyncError: () => void;

  setCalendarViewport: (year: number, month: number) => void;
  refreshExercises: () => Promise<void>;
  refreshWorkouts: () => Promise<void>;
  fetchCalendarDatesForMonth: (year: number, month: number) => Promise<void>;

  setSelectedDate: (date: string) => void;
  setCurrentView: (view: ViewId) => void;

  addExercise: (
    name: string,
    kind: ExerciseKind,
    muscleGroup: MuscleGroup,
  ) => Promise<void>;
  deleteExercise: (id: string) => Promise<void>;

  createWorkout: (date: string, name?: string) => Promise<void>;
  generateWorkout: (
    muscleGroups: MuscleGroup[],
    maxExerciseCount: number,
    date: string,
  ) => Promise<void>;
  deleteWorkout: (id: string) => Promise<void>;
  updateWorkoutName: (id: string, name: string) => Promise<void>;

  addExerciseToWorkout: (workoutId: string, exerciseId: string) => Promise<void>;
  removeExerciseFromWorkout: (workoutId: string, exerciseId: string) => Promise<void>;

  addSet: (workoutId: string, exerciseId: string, set: PerformedSet) => Promise<void>;
  updateSet: (
    workoutId: string,
    exerciseId: string,
    setIndex: number,
    set: PerformedSet,
  ) => Promise<void>;
  removeSet: (
    workoutId: string,
    exerciseId: string,
    setIndex: number,
  ) => Promise<void>;

  refreshProfile: () => Promise<void>;
  updateProfile: (profile: UserProfile) => Promise<void>;
  updateWeight: (value: number, units: WeightUnits) => Promise<void>;
}

async function afterWorkoutMutation(get: () => GymStore) {
  const s = get();
  await Promise.all([
    s.refreshWorkouts(),
    s.fetchCalendarDatesForMonth(s.calendarViewYear, s.calendarViewMonth),
  ]);
}

export const useStore = create<GymStore>()(
  persist(
    (set, get) => ({
      exercises: [],
      workouts: [],
      calendarWorkoutDates: [],
      selectedDate: toDateString(new Date()),
      currentView: "calendar",
      calendarViewYear: new Date().getFullYear(),
      calendarViewMonth: new Date().getMonth(),

      profile: null,
      profileLoading: false,

      syncError: null,
      isLoading: true,

      clearSyncError: () => set({ syncError: null }),

      bootstrap: async () => {
        set({ isLoading: true, syncError: null });
        try {
          const now = new Date();
          set({
            calendarViewYear: now.getFullYear(),
            calendarViewMonth: now.getMonth(),
          });
          await Promise.all([get().refreshExercises(), get().refreshWorkouts()]);
        } catch (e) {
          set({
            syncError: e instanceof Error ? e.message : String(e),
          });
        } finally {
          set({ isLoading: false });
        }
      },

      setCalendarViewport: (year, month) => {
        set({ calendarViewYear: year, calendarViewMonth: month });
        void get().fetchCalendarDatesForMonth(year, month);
      },

      refreshExercises: async () => {
        const list = await api.listExercises();
        set({ exercises: list });
      },

      refreshWorkouts: async () => {
        const date = get().selectedDate;
        const list = await api.listWorkoutsForDate(date);
        set({ workouts: list });
      },

      fetchCalendarDatesForMonth: async (year, month) => {
        try {
          const first = new Date(year, month, 1);
          const last = new Date(year, month + 1, 0);
          const from = toDateString(first);
          const to = toDateString(last);
          const dates = await api.getWorkoutDates(from, to);
          set({ calendarWorkoutDates: dates });
        } catch (e) {
          set({
            syncError: e instanceof Error ? e.message : String(e),
          });
        }
      },

      setSelectedDate: (date) => {
        set({ selectedDate: date });
        void (async () => {
          try {
            await get().refreshWorkouts();
          } catch (e) {
            set({
              syncError: e instanceof Error ? e.message : String(e),
            });
          }
        })();
      },

      setCurrentView: (view) => set({ currentView: view }),

      addExercise: async (name, kind, muscleGroup) => {
        try {
          await api.createExerciseApi(name, kind, muscleGroup);
          await get().refreshExercises();
        } catch (e) {
          const msg = e instanceof Error ? e.message : String(e);
          set({ syncError: msg });
          throw e;
        }
      },

      deleteExercise: async (id) => {
        try {
          await api.deleteExerciseApi(id);
          await get().refreshExercises();
          await afterWorkoutMutation(get);
        } catch (e) {
          const msg = e instanceof Error ? e.message : String(e);
          set({ syncError: msg });
          throw e;
        }
      },

      createWorkout: async (date, name) => {
        try {
          await api.createWorkoutApi(date, name);
          await afterWorkoutMutation(get);
        } catch (e) {
          const msg = e instanceof Error ? e.message : String(e);
          set({ syncError: msg });
          throw e;
        }
      },

      generateWorkout: async (muscleGroups, maxExerciseCount, date) => {
        try {
          await api.generateWorkoutApi(muscleGroups, maxExerciseCount, date);
          await afterWorkoutMutation(get);
        } catch (e) {
          const msg = e instanceof Error ? e.message : String(e);
          set({ syncError: msg });
          throw e;
        }
      },

      deleteWorkout: async (id) => {
        try {
          await api.deleteWorkoutApi(id);
          await afterWorkoutMutation(get);
        } catch (e) {
          const msg = e instanceof Error ? e.message : String(e);
          set({ syncError: msg });
          throw e;
        }
      },

      updateWorkoutName: async (id, name) => {
        try {
          const trimmed = name.trim();
          const w = await api.updateWorkoutNameApi(
            id,
            trimmed === "" ? null : trimmed,
          );
          set((s) => ({
            workouts: s.workouts.map((x) => (x.id === id ? w : x)),
          }));
        } catch (e) {
          const msg = e instanceof Error ? e.message : String(e);
          set({ syncError: msg });
          throw e;
        }
      },

      addExerciseToWorkout: async (workoutId, exerciseId) => {
        try {
          await api.addExerciseToWorkoutApi(workoutId, exerciseId);
          await get().refreshWorkouts();
        } catch (e) {
          const msg = e instanceof Error ? e.message : String(e);
          set({ syncError: msg });
          throw e;
        }
      },

      removeExerciseFromWorkout: async (workoutId, exerciseId) => {
        try {
          await api.removeExerciseFromWorkoutApi(workoutId, exerciseId);
          await get().refreshWorkouts();
        } catch (e) {
          const msg = e instanceof Error ? e.message : String(e);
          set({ syncError: msg });
          throw e;
        }
      },

      addSet: async (workoutId, exerciseId, newSet) => {
        try {
          await api.addSetApi(workoutId, exerciseId, newSet);
          await get().refreshWorkouts();
        } catch (e) {
          const msg = e instanceof Error ? e.message : String(e);
          set({ syncError: msg });
          throw e;
        }
      },

      updateSet: async (workoutId, exerciseId, setIndex, newSet) => {
        const previous = get().workouts;
        set((s) => ({
          workouts: s.workouts.map((w) => {
            if (w.id !== workoutId) return w;
            return {
              ...w,
              entries: w.entries.map((e) => {
                if (e.exerciseId !== exerciseId) return e;
                const sets = [...e.sets];
                sets[setIndex] = newSet;
                return { ...e, sets };
              }),
            };
          }),
        }));
        try {
          await api.updateSetApi(workoutId, exerciseId, setIndex, newSet);
        } catch (e) {
          set({ workouts: previous });
          const msg = e instanceof Error ? e.message : String(e);
          set({ syncError: msg });
          throw e;
        }
      },

      removeSet: async (workoutId, exerciseId, setIndex) => {
        try {
          await api.removeSetApi(workoutId, exerciseId, setIndex);
          await get().refreshWorkouts();
        } catch (e) {
          const msg = e instanceof Error ? e.message : String(e);
          set({ syncError: msg });
          throw e;
        }
      },

      refreshProfile: async () => {
        set({ profileLoading: true });
        try {
          const profile = await api.getProfile();
          set({ profile });
        } catch (e) {
          const msg = e instanceof Error ? e.message : String(e);
          set({ syncError: msg });
        } finally {
          set({ profileLoading: false });
        }
      },

      updateProfile: async (profile) => {
        try {
          const updated = await api.updateProfile(profile);
          set({ profile: updated });
        } catch (e) {
          const msg = e instanceof Error ? e.message : String(e);
          set({ syncError: msg });
          throw e;
        }
      },

      updateWeight: async (value, units) => {
        try {
          const updated = await api.updateWeight(value, units);
          set({ profile: updated });
        } catch (e) {
          const msg = e instanceof Error ? e.message : String(e);
          set({ syncError: msg });
          throw e;
        }
      },
    }),
    {
      name: "gym-tracker-storage",
      partialize: (s) => ({
        selectedDate: s.selectedDate,
        currentView: s.currentView,
      }),
    },
  ),
);
