import { useState, useRef, useEffect, type FormEvent } from "react";
import { useStore } from "../store";
import { cn, formatDateHeading } from "../utils";
import type { Exercise, PerformedSet, WorkoutExercise, Workout } from "../types";
import { ExercisePicker } from "./ExercisePicker";

export function WorkoutView() {
  const selectedDate = useStore((s) => s.selectedDate);
  const workouts = useStore((s) => s.workouts);
  const exercises = useStore((s) => s.exercises);
  const createWorkout = useStore((s) => s.createWorkout);

  const dateWorkouts = workouts.filter((w) => w.startDate === selectedDate);

  return (
    <section className="px-4 pb-12 animate-fade-in" aria-label="Workouts">
      <div className="flex items-center justify-between mt-4 mb-4">
        <h3 className="text-sm font-medium text-fg-secondary">
          {formatDateHeading(selectedDate)}
        </h3>
        {dateWorkouts.length > 0 && (
          <button
            onClick={() => createWorkout(selectedDate)}
            className="text-xs text-accent hover:text-accent-bright transition-colors font-medium"
          >
            + Workout
          </button>
        )}
      </div>

      {dateWorkouts.length === 0 ? (
        <EmptyState onStart={() => createWorkout(selectedDate)} />
      ) : (
        <div className="space-y-4">
          {dateWorkouts.map((workout) => (
            <WorkoutCard
              key={workout.id}
              workout={workout}
              exercises={exercises}
            />
          ))}
        </div>
      )}
    </section>
  );
}

function EmptyState({ onStart }: { onStart: () => void }) {
  return (
    <div className="flex flex-col items-center justify-center py-16 animate-fade-in">
      <div className="w-16 h-16 rounded-2xl bg-surface-1 border border-border flex items-center justify-center text-2xl mb-4">
        💪
      </div>
      <p className="text-fg-secondary text-sm mb-1">Rest day?</p>
      <p className="text-fg-muted text-xs mb-6">No workout recorded</p>
      <button
        onClick={onStart}
        className="bg-accent hover:bg-accent-bright text-white text-sm font-semibold px-6 py-2.5 rounded-xl transition-colors shadow-lg shadow-accent/20"
      >
        Start Workout
      </button>
    </div>
  );
}

function WorkoutCard({
  workout,
  exercises,
}: {
  workout: Workout;
  exercises: Exercise[];
}) {
  const [showPicker, setShowPicker] = useState(false);
  const deleteWorkout = useStore((s) => s.deleteWorkout);

  return (
    <article className="bg-surface-1 rounded-xl border border-border overflow-hidden animate-slide-up">
      <div className="px-4 py-3 flex items-center justify-between border-b border-border/60">
        <WorkoutName workout={workout} />
        <button
          onClick={() => deleteWorkout(workout.id)}
          className="text-fg-muted hover:text-danger transition-colors text-xs px-2 py-1 rounded hover:bg-surface-2"
          aria-label="Delete workout"
        >
          Delete
        </button>
      </div>

      <div className="divide-y divide-border/40">
        {workout.entries.map((entry, idx) => (
          <ExerciseEntry
            key={`${entry.exerciseId}-${idx}`}
            workoutId={workout.id}
            entry={entry}
            entryIndex={idx}
            exercise={exercises.find((e) => e.id === entry.exerciseId)}
          />
        ))}
      </div>

      <button
        onClick={() => setShowPicker(true)}
        className="w-full px-4 py-3 text-sm text-accent hover:text-accent-bright hover:bg-surface-2/50 transition-colors font-medium text-left"
      >
        + Add Exercise
      </button>

      {showPicker && (
        <ExercisePicker
          workoutId={workout.id}
          onClose={() => setShowPicker(false)}
        />
      )}
    </article>
  );
}

function WorkoutName({ workout }: { workout: Workout }) {
  const [editing, setEditing] = useState(false);
  const [name, setName] = useState(workout.name || "");
  const updateWorkoutName = useStore((s) => s.updateWorkoutName);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (editing) {
      inputRef.current?.focus();
      inputRef.current?.select();
    }
  }, [editing]);

  function save() {
    updateWorkoutName(workout.id, name.trim());
    setEditing(false);
  }

  if (editing) {
    return (
      <input
        ref={inputRef}
        value={name}
        onChange={(e) => setName(e.target.value)}
        onBlur={save}
        onKeyDown={(e) => {
          if (e.key === "Enter") save();
          if (e.key === "Escape") {
            setName(workout.name || "");
            setEditing(false);
          }
        }}
        className="bg-transparent text-sm font-semibold text-fg outline-none border-b border-accent pb-0.5 min-w-0"
        placeholder="Workout name..."
      />
    );
  }

  return (
    <button
      onClick={() => {
        setName(workout.name || "");
        setEditing(true);
      }}
      className="text-sm font-semibold text-fg hover:text-accent-bright transition-colors text-left truncate"
    >
      {workout.name || (
        <span className="text-fg-muted italic font-normal">
          Untitled Workout
        </span>
      )}
    </button>
  );
}

function ExerciseEntry({
  workoutId,
  entry,
  entryIndex,
  exercise,
}: {
  workoutId: string;
  entry: WorkoutExercise;
  entryIndex: number;
  exercise: Exercise | undefined;
}) {
  const removeExerciseFromWorkout = useStore(
    (s) => s.removeExerciseFromWorkout,
  );
  const removeSet = useStore((s) => s.removeSet);

  const isWeighted = exercise?.kind === "Weighted";

  return (
    <div className="px-4 py-3">
      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center gap-2 min-w-0">
          <span className="text-sm font-medium text-fg truncate">
            {exercise?.name ?? "Unknown Exercise"}
          </span>
          <span className="shrink-0 text-[0.6rem] uppercase tracking-wider text-fg-muted bg-surface-2 px-1.5 py-0.5 rounded">
            {exercise?.muscleGroup}
          </span>
        </div>
        <button
          onClick={() => removeExerciseFromWorkout(workoutId, entryIndex)}
          className="text-fg-muted hover:text-danger transition-colors ml-2 shrink-0"
          aria-label="Remove exercise"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <path d="M18 6L6 18M6 6l12 12" />
          </svg>
        </button>
      </div>

      {entry.sets.length > 0 && (
        <div className="mb-2">
          <div
            className={cn(
              "grid text-[0.6rem] uppercase tracking-wider text-fg-muted font-semibold mb-1 px-1",
              isWeighted
                ? "grid-cols-[1.5rem_1fr_1fr_1.5rem]"
                : "grid-cols-[1.5rem_1fr_1.5rem]",
            )}
          >
            <span>Set</span>
            {isWeighted && <span>Weight</span>}
            <span>Reps</span>
            <span />
          </div>
          {entry.sets.map((set, si) => (
            <div
              key={si}
              className={cn(
                "grid items-center text-sm py-1 px-1 rounded hover:bg-surface-2/50 group transition-colors",
                isWeighted
                  ? "grid-cols-[1.5rem_1fr_1fr_1.5rem]"
                  : "grid-cols-[1.5rem_1fr_1.5rem]",
              )}
            >
              <span className="text-fg-muted text-xs">{si + 1}</span>
              {set.kind.type === "Weighted" && (
                <span className="text-fg-secondary">
                  {set.kind.weight.value}
                  <span className="text-fg-muted text-xs ml-0.5">
                    {set.kind.weight.units}
                  </span>
                </span>
              )}
              <span className="text-fg-secondary">{set.reps}</span>
              <button
                onClick={() => removeSet(workoutId, entryIndex, si)}
                className="opacity-0 group-hover:opacity-100 text-fg-muted hover:text-danger transition-all"
                aria-label={`Remove set ${si + 1}`}
              >
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <path d="M18 6L6 18M6 6l12 12" />
                </svg>
              </button>
            </div>
          ))}
        </div>
      )}

      <AddSetForm
        workoutId={workoutId}
        entryIndex={entryIndex}
        isWeighted={isWeighted}
        lastSet={entry.sets.at(-1)}
      />
    </div>
  );
}

function AddSetForm({
  workoutId,
  entryIndex,
  isWeighted,
  lastSet,
}: {
  workoutId: string;
  entryIndex: number;
  isWeighted: boolean;
  lastSet: PerformedSet | undefined;
}) {
  const addSet = useStore((s) => s.addSet);

  const defaultWeight =
    lastSet?.kind.type === "Weighted"
      ? String(lastSet.kind.weight.value)
      : "";
  const [weight, setWeight] = useState(defaultWeight);
  const [reps, setReps] = useState("");

  useEffect(() => {
    if (lastSet?.kind.type === "Weighted") {
      setWeight(String(lastSet.kind.weight.value));
    }
  }, [lastSet]);

  function handleSubmit(e: FormEvent) {
    e.preventDefault();
    const repsNum = parseInt(reps);
    if (isNaN(repsNum) || repsNum <= 0) return;

    const newSet: PerformedSet = isWeighted
      ? {
          kind: {
            type: "Weighted",
            weight: { value: parseFloat(weight) || 0, units: "kg" },
          },
          reps: repsNum,
        }
      : { kind: { type: "BodyWeight" }, reps: repsNum };

    addSet(workoutId, entryIndex, newSet);
    setReps("");
  }

  return (
    <form onSubmit={handleSubmit} className="flex items-center gap-2">
      {isWeighted && (
        <div className="flex items-center gap-1">
          <input
            type="number"
            inputMode="decimal"
            step="0.5"
            value={weight}
            onChange={(e) => setWeight(e.target.value)}
            className="w-16 bg-surface-2 rounded-lg px-2 py-1.5 text-sm text-center text-fg outline-none border border-transparent focus:border-accent/50 transition-colors"
            placeholder="kg"
          />
          <span className="text-[0.65rem] text-fg-muted">kg</span>
        </div>
      )}
      <div className="flex items-center gap-1">
        <input
          type="number"
          inputMode="numeric"
          value={reps}
          onChange={(e) => setReps(e.target.value)}
          className="w-16 bg-surface-2 rounded-lg px-2 py-1.5 text-sm text-center text-fg outline-none border border-transparent focus:border-accent/50 transition-colors"
          placeholder="reps"
        />
        <span className="text-[0.65rem] text-fg-muted">reps</span>
      </div>
      <button
        type="submit"
        className="bg-accent/15 text-accent text-xs font-semibold px-3 py-1.5 rounded-lg hover:bg-accent/25 transition-colors"
      >
        + Set
      </button>
    </form>
  );
}
