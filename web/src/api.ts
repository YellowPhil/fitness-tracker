import type {
  Exercise,
  ExerciseKind,
  ExerciseSource,
  HeightUnits,
  MuscleGroup,
  PerformedSet,
  UserProfile,
  WeightUnits,
  Workout,
  WorkoutExercise,
  WorkoutSource,
} from "./types";
import { WORKOUT_SOURCE_API } from "./types";
import { getInitData } from "./telegram";
import { toDateString } from "./utils";

const API_BASE = import.meta.env.VITE_API_BASE ?? "";

/** When not in Telegram, use `x-user-id` (backend must run with `DEV_SKIP_AUTH=1`). */
const DEV_USER_ID = import.meta.env.VITE_DEV_USER_ID ?? "1";

function headers(): HeadersInit {
  const h: Record<string, string> = {
    "Content-Type": "application/json",
  };
  const initData = getInitData();
  if (initData) {
    h.Authorization = `tma ${initData}`;
  } else {
    h["x-user-id"] = DEV_USER_ID;
  }
  return h;
}

async function parseError(res: Response): Promise<string> {
  const text = await res.text();
  if (!text) return res.statusText || `HTTP ${res.status}`;
  try {
    const j = JSON.parse(text) as { message?: string };
    return j.message ?? text;
  } catch {
    return text;
  }
}

export async function apiFetch<T>(
  path: string,
  init?: RequestInit,
): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`, {
    ...init,
    headers: { ...headers(), ...init?.headers },
  });

  if (res.status === 204) {
    return undefined as T;
  }

  if (!res.ok) {
    throw new Error(await parseError(res));
  }

  const ct = res.headers.get("content-type");
  if (!ct?.includes("application/json")) {
    return undefined as T;
  }

  const text = await res.text();
  if (!text) return undefined as T;
  return JSON.parse(text) as T;
}

// --- wire types (match crates/infra/src/web) ---

interface ApiLoadWeighted {
  type: "weighted";
  value: number;
  units: string;
}

interface ApiLoadBodyweight {
  type: "bodyweight";
}

type ApiLoad = ApiLoadWeighted | ApiLoadBodyweight;

interface ApiSet {
  reps: number;
  load: ApiLoad;
}

interface ApiWorkoutEntry {
  excercise_id: string;
  notes?: string | null;
  sets: ApiSet[];
}

interface ApiWorkout {
  id: string;
  name: string | null;
  start_date: number;
  end_date: number | null;
  source?: string;
  entries: ApiWorkoutEntry[];
}

interface ApiExercise {
  id: string;
  name: string;
  kind: string;
  muscle_group: string;
  secondary_muscle_groups?: string[] | null;
  source: string;
}

const MUSCLE: MuscleGroup[] = ["Chest", "Back", "Shoulders", "Arms", "Legs", "Core"];

function mapMuscleGroup(s: string): MuscleGroup {
  if (MUSCLE.includes(s as MuscleGroup)) return s as MuscleGroup;
  const t = s.charAt(0).toUpperCase() + s.slice(1).toLowerCase();
  if (MUSCLE.includes(t as MuscleGroup)) return t as MuscleGroup;
  return "Chest";
}

function mapExerciseKind(k: string): ExerciseKind {
  const x = k.toLowerCase();
  if (x === "bodyweight") return "BodyWeight";
  return "Weighted";
}

function mapSource(s: string): ExerciseSource {
  return s.toLowerCase() === "user" ? "UserDefined" : "BuiltIn";
}

export function mapExerciseFromApi(e: ApiExercise): Exercise {
  return {
    id: e.id,
    name: e.name,
    kind: mapExerciseKind(e.kind),
    muscleGroup: mapMuscleGroup(e.muscle_group),
    secondaryMuscleGroups: e.secondary_muscle_groups?.map(mapMuscleGroup),
    source: mapSource(e.source),
  };
}

function mapLoadFromApi(load: ApiLoad): PerformedSet["kind"] {
  if (load.type === "bodyweight") {
    return { type: "BodyWeight" };
  }
  const u = load.units.toLowerCase();
  const units: WeightUnits = u === "lbs" || u === "pounds" ? "lbs" : "kg";
  return {
    type: "Weighted",
    weight: { value: load.value, units },
  };
}

function mapSetFromApi(s: ApiSet): PerformedSet {
  return {
    kind: mapLoadFromApi(s.load),
    reps: s.reps,
  };
}

function mapEntryFromApi(e: ApiWorkoutEntry): WorkoutExercise {
  return {
    exerciseId: e.excercise_id,
    notes: e.notes ?? undefined,
    sets: e.sets.map(mapSetFromApi),
  };
}

function mapWorkoutSourceFromApi(s: string | undefined): WorkoutSource {
  if (s === WORKOUT_SOURCE_API.aiGenerated) return WORKOUT_SOURCE_API.aiGenerated;
  return WORKOUT_SOURCE_API.manual;
}

export function mapWorkoutFromApi(w: ApiWorkout): Workout {
  const start = new Date(w.start_date * 1000);
  const end =
    w.end_date != null ? new Date(w.end_date * 1000) : undefined;
  return {
    id: w.id,
    name: w.name ?? undefined,
    startDate: toDateString(start),
    endDate: end ? toDateString(end) : undefined,
    source: mapWorkoutSourceFromApi(w.source),
    entries: w.entries.map(mapEntryFromApi),
  };
}

function mapLoadToApi(kind: PerformedSet["kind"]): Record<string, unknown> {
  if (kind.type === "BodyWeight") {
    return { type: "bodyweight" };
  }
  return {
    type: "weighted",
    value: kind.weight.value,
    units: kind.weight.units,
  };
}

export async function listExercises(): Promise<Exercise[]> {
  const rows = await apiFetch<ApiExercise[]>("/api/exercises");
  return rows.map(mapExerciseFromApi);
}

export async function createExerciseApi(
  name: string,
  kind: ExerciseKind,
  muscleGroup: MuscleGroup,
): Promise<void> {
  await apiFetch("/api/exercises", {
    method: "POST",
    body: JSON.stringify({
      name,
      kind: kind === "BodyWeight" ? "bodyweight" : "weighted",
      muscle_group: muscleGroup.toLowerCase(),
      secondary_muscle_groups: null,
    }),
  });
}

export async function deleteExerciseApi(id: string): Promise<void> {
  await apiFetch(`/api/exercises/${encodeURIComponent(id)}`, {
    method: "DELETE",
  });
}

export async function listWorkoutsForDate(date: string): Promise<Workout[]> {
  const q = new URLSearchParams({ date });
  const rows = await apiFetch<ApiWorkout[]>(`/api/workouts?${q}`);
  return rows.map(mapWorkoutFromApi);
}

export async function getWorkoutDates(
  from: string,
  to: string,
): Promise<string[]> {
  const q = new URLSearchParams({ from, to });
  const res = await apiFetch<{ dates: string[] }>(
    `/api/workouts/dates?${q}`,
  );
  return res.dates;
}

export async function createWorkoutApi(
  date: string,
  name?: string,
): Promise<Workout> {
  const body: { name?: string; date: string } = {
    date: `${date}T00:00:00Z`,
  };
  if (name !== undefined && name.trim() !== "") body.name = name.trim();
  const w = await apiFetch<ApiWorkout>("/api/workouts", {
    method: "POST",
    body: JSON.stringify(body),
  });
  return mapWorkoutFromApi(w);
}

/** AI-generated workout plan; requires `OPENAI_API_KEY` on the server. */
export async function generateWorkoutApi(
  muscleGroups: MuscleGroup[],
  maxExerciseCount: number,
  date?: string,
): Promise<Workout> {
  const body: Record<string, unknown> = {
    muscle_groups: muscleGroups,
    max_exercise_count: maxExerciseCount,
  };
  if (date !== undefined) {
    body.date = `${date}T00:00:00Z`;
  }
  const w = await apiFetch<ApiWorkout>("/api/workouts/generate", {
    method: "POST",
    body: JSON.stringify(body),
  });
  return mapWorkoutFromApi(w);
}

export async function deleteWorkoutApi(id: string): Promise<void> {
  await apiFetch(`/api/workouts/${encodeURIComponent(id)}`, {
    method: "DELETE",
  });
}

export async function updateWorkoutNameApi(
  id: string,
  name: string | null,
): Promise<Workout> {
  const w = await apiFetch<ApiWorkout>(
    `/api/workouts/${encodeURIComponent(id)}`,
    {
      method: "PATCH",
      body: JSON.stringify({ name }),
    },
  );
  return mapWorkoutFromApi(w);
}

export async function addExerciseToWorkoutApi(
  workoutId: string,
  exerciseId: string,
): Promise<void> {
  await apiFetch(
    `/api/workouts/${encodeURIComponent(workoutId)}/exercises`,
    {
      method: "POST",
      body: JSON.stringify({ excercise_id: exerciseId }),
    },
  );
}

export async function removeExerciseFromWorkoutApi(
  workoutId: string,
  exerciseId: string,
): Promise<void> {
  await apiFetch(
    `/api/workouts/${encodeURIComponent(workoutId)}/exercises/${encodeURIComponent(exerciseId)}`,
    { method: "DELETE" },
  );
}

export async function addSetApi(
  workoutId: string,
  exerciseId: string,
  set: PerformedSet,
): Promise<void> {
  await apiFetch(
    `/api/workouts/${encodeURIComponent(workoutId)}/exercises/${encodeURIComponent(exerciseId)}/sets`,
    {
      method: "POST",
      body: JSON.stringify({
        reps: set.reps,
        load: mapLoadToApi(set.kind),
      }),
    },
  );
}

export async function updateSetApi(
  workoutId: string,
  exerciseId: string,
  setIndex: number,
  set: PerformedSet,
): Promise<void> {
  await apiFetch(
    `/api/workouts/${encodeURIComponent(workoutId)}/exercises/${encodeURIComponent(exerciseId)}/sets/${setIndex}`,
    {
      method: "PUT",
      body: JSON.stringify({
        reps: set.reps,
        load: mapLoadToApi(set.kind),
      }),
    },
  );
}

export async function removeSetApi(
  workoutId: string,
  exerciseId: string,
  setIndex: number,
): Promise<void> {
  await apiFetch(
    `/api/workouts/${encodeURIComponent(workoutId)}/exercises/${encodeURIComponent(exerciseId)}/sets/${setIndex}`,
    { method: "DELETE" },
  );
}

// --- Profile / Health ---

interface ApiProfile {
  weight_value: number;
  weight_units: string;
  height_value: number;
  height_units: string;
  age: number;
}

function mapProfileFromApi(p: ApiProfile): UserProfile {
  const wu = p.weight_units.toLowerCase();
  const hu = p.height_units.toLowerCase();
  return {
    weight: {
      value: p.weight_value,
      units: wu === "lbs" || wu === "pounds" ? "lbs" : "kg",
    },
    height: {
      value: p.height_value,
      units: (hu === "in" || hu === "inches" ? "in" : "cm") as HeightUnits,
    },
    age: p.age,
  };
}

export async function getProfile(): Promise<UserProfile> {
  const p = await apiFetch<ApiProfile>("/api/profile");
  return mapProfileFromApi(p);
}

export async function updateProfile(
  profile: UserProfile,
): Promise<UserProfile> {
  const p = await apiFetch<ApiProfile>("/api/profile", {
    method: "PUT",
    body: JSON.stringify({
      weight_value: profile.weight.value,
      weight_units: profile.weight.units,
      height_value: profile.height.value,
      height_units: profile.height.units,
      age: profile.age,
    }),
  });
  return mapProfileFromApi(p);
}

export async function updateWeight(
  value: number,
  units: WeightUnits,
): Promise<UserProfile> {
  const p = await apiFetch<ApiProfile>("/api/profile/weight", {
    method: "PATCH",
    body: JSON.stringify({ value, units }),
  });
  return mapProfileFromApi(p);
}
