import { create } from "zustand";
import { persist } from "zustand/middleware";
import * as api from "./api";
import type {
  Exercise,
  ExerciseKind,
  GenerationJob,
  GenerationJobStatus,
  MuscleGroup,
  PerformedSet,
  UserProfile,
  WeightUnits,
  Workout,
  WorkoutPreferences,
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
  preferences: WorkoutPreferences | null;
  preferencesLoading: boolean;

  syncError: string | null;
  isLoading: boolean;
  generationJobsById: Record<string, GenerationJob>;
  activeGenerationJobId: string | null;
  generationStreamConnected: boolean;
  generationStreamStop: (() => void) | null;

  bootstrap: () => Promise<void>;
  clearSyncError: () => void;
  connectGenerationStream: () => void;
  disconnectGenerationStream: () => void;

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
  refreshPreferences: () => Promise<void>;
  updatePreferences: (preferences: WorkoutPreferences) => Promise<void>;
}

async function afterWorkoutMutation(get: () => GymStore) {
  const s = get();
  await Promise.all([
    s.refreshWorkouts(),
    s.fetchCalendarDatesForMonth(s.calendarViewYear, s.calendarViewMonth),
  ]);
}

function isActiveGenerationStatus(status: GenerationJobStatus): boolean {
  return status === "queued" || status === "running";
}

function selectActiveGenerationJobId(
  jobsById: Record<string, GenerationJob>,
): string | null {
  const activeJobs = Object.values(jobsById).filter((job) =>
    isActiveGenerationStatus(job.status),
  );
  if (activeJobs.length === 0) return null;
  activeJobs.sort((a, b) => b.updatedAt.localeCompare(a.updatedAt));
  return activeJobs[0].id;
}

function mergeGenerationJobs(
  current: Record<string, GenerationJob>,
  incoming: GenerationJob[],
): Record<string, GenerationJob> {
  const next = { ...current };
  for (const job of incoming) {
    const prev = next[job.id];
    if (!prev || job.version >= prev.version) {
      next[job.id] = job;
    }
  }
  return next;
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
      preferences: null,
      preferencesLoading: false,

      syncError: null,
      isLoading: true,
      generationJobsById: {},
      activeGenerationJobId: null,
      generationStreamConnected: false,
      generationStreamStop: null,

      clearSyncError: () => set({ syncError: null }),

      connectGenerationStream: () => {
        const current = get();
        if (current.generationStreamConnected) return;

        const stop = api.connectGenerationJobsStream({
          onSnapshot: (jobs) => {
            set((state) => {
              const generationJobsById = mergeGenerationJobs(state.generationJobsById, jobs);
              return {
                generationJobsById,
                activeGenerationJobId: selectActiveGenerationJobId(generationJobsById),
              };
            });
          },
          onJobUpdated: (job) => {
            const previous = get().generationJobsById[job.id];
            set((state) => {
              const generationJobsById = mergeGenerationJobs(state.generationJobsById, [job]);
              return {
                generationJobsById,
                activeGenerationJobId: selectActiveGenerationJobId(generationJobsById),
              };
            });

            if (
              !previous ||
              previous.status !== job.status ||
              previous.workoutId !== job.workoutId
            ) {
              const selectedDate = get().selectedDate;
              if (job.date === selectedDate || job.status === "completed") {
                void afterWorkoutMutation(get);
              }
            }
          },
          onError: (error) => {
            set({
              generationStreamConnected: false,
              generationStreamStop: null,
              syncError: error.message,
            });
            window.setTimeout(() => {
              get().connectGenerationStream();
            }, 1500);
          },
        });

        set({ generationStreamConnected: true, generationStreamStop: stop });
      },

      disconnectGenerationStream: () => {
        const stop = get().generationStreamStop;
        if (stop) stop();
        set({ generationStreamConnected: false, generationStreamStop: null });
      },

      bootstrap: async () => {
        set({ isLoading: true, syncError: null });
        try {
          const now = new Date();
          set({
            calendarViewYear: now.getFullYear(),
            calendarViewMonth: now.getMonth(),
          });
          const [jobs] = await Promise.all([
            api.listGenerationJobs("all", 20),
            get().refreshExercises(),
            get().refreshWorkouts(),
          ]);
          const generationJobsById = jobs.reduce<Record<string, GenerationJob>>((acc, job) => {
            acc[job.id] = job;
            return acc;
          }, {});
          set({
            generationJobsById,
            activeGenerationJobId: selectActiveGenerationJobId(generationJobsById),
          });
          get().connectGenerationStream();
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
          const result = await api.generateWorkoutApi(
            muscleGroups,
            maxExerciseCount,
            date,
          );
          set((state) => {
            const generationJobsById = {
              ...state.generationJobsById,
              [result.job.id]: result.job,
            };
            return {
              generationJobsById,
              activeGenerationJobId: selectActiveGenerationJobId(generationJobsById),
            };
          });
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

      refreshPreferences: async () => {
        set({ preferencesLoading: true });
        try {
          const preferences = await api.getPreferences();
          set({ preferences });
        } catch (e) {
          const msg = e instanceof Error ? e.message : String(e);
          set({ syncError: msg });
        } finally {
          set({ preferencesLoading: false });
        }
      },

      updatePreferences: async (preferences) => {
        try {
          const updated = await api.updatePreferences(preferences);
          set({ preferences: updated });
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
        generationJobsById: s.generationJobsById,
        activeGenerationJobId: s.activeGenerationJobId,
      }),
    },
  ),
);
