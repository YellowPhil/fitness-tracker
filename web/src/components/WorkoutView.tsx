import { useState, useRef, useEffect, type FormEvent, type ReactNode } from "react";
import { useStore } from "../store";
import { cn, formatDateHeading } from "../utils";
import type { Exercise, PerformedSet, WorkoutExercise, Workout } from "../types";
import { ExercisePicker } from "./ExercisePicker";
import { GenerateWorkoutModal } from "./GenerateWorkoutModal";

export function WorkoutView() {
  const selectedDate = useStore((s) => s.selectedDate);
  const workouts = useStore((s) => s.workouts);
  const exercises = useStore((s) => s.exercises);
  const createWorkout = useStore((s) => s.createWorkout);

  const [showGenerateModal, setShowGenerateModal] = useState(false);

  const dateWorkouts = workouts.filter((w) => w.startDate === selectedDate);

  return (
    <section className="px-4 pb-12 animate-fade-in" aria-label="Workouts">
      <div className="flex items-center justify-between mt-4 mb-4">
        <h3 className="text-sm font-medium text-fg-secondary">
          {formatDateHeading(selectedDate)}
        </h3>
        {dateWorkouts.length > 0 && (
          <div className="flex items-center gap-3">
            <button
              onClick={() => setShowGenerateModal(true)}
              className="text-xs text-fg-muted hover:text-accent transition-colors font-medium flex items-center gap-1"
              title="Generate with AI"
            >
              <SparkleIcon />
              AI
            </button>
            <button
              onClick={() => void createWorkout(selectedDate)}
              className="text-xs text-accent hover:text-accent-bright transition-colors font-medium"
            >
              + Workout
            </button>
          </div>
        )}
      </div>

      {dateWorkouts.length === 0 ? (
        <EmptyState
          onStart={() => void createWorkout(selectedDate)}
          onGenerate={() => setShowGenerateModal(true)}
        />
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

      <GenerateWorkoutModal
        open={showGenerateModal}
        onClose={() => setShowGenerateModal(false)}
        date={selectedDate}
      />
    </section>
  );
}

function SparkleIcon() {
  return (
    <svg
      width="13"
      height="13"
      viewBox="0 0 24 24"
      fill="currentColor"
      aria-hidden="true"
    >
      <path d="M12 2l2.4 7.2H22l-6.2 4.5 2.4 7.2L12 16.4l-6.2 4.5 2.4-7.2L2 9.2h7.6z" />
    </svg>
  );
}

function EmptyState({
  onStart,
  onGenerate,
}: {
  onStart: () => void;
  onGenerate: () => void;
}) {
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
      <button
        onClick={onGenerate}
        className="mt-3 flex items-center gap-1.5 text-sm font-medium text-fg-muted hover:text-accent border border-border hover:border-accent/50 px-5 py-2 rounded-xl transition-colors"
      >
        <SparkleIcon />
        Generate with AI
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
  const [confirmDelete, setConfirmDelete] = useState(false);
  const deleteWorkout = useStore((s) => s.deleteWorkout);

  return (
    <article className="bg-surface-1 rounded-xl border border-border overflow-hidden animate-slide-up">
      <div className="px-4 py-3 flex items-center justify-between border-b border-border/60">
        <WorkoutName workout={workout} />
        {confirmDelete ? (
          <div className="flex items-center gap-2 shrink-0 animate-fade-in">
            <span className="text-xs text-fg-muted">Delete?</span>
            <button
              onClick={() => void deleteWorkout(workout.id)}
              className="text-xs font-semibold text-danger bg-danger/10 hover:bg-danger/20 px-3 py-1 rounded-lg transition-colors"
            >
              Yes
            </button>
            <button
              onClick={() => setConfirmDelete(false)}
              className="text-xs font-semibold text-fg-muted hover:text-fg px-3 py-1 rounded-lg hover:bg-surface-2 transition-colors"
            >
              No
            </button>
          </div>
        ) : (
          <button
            onClick={() => setConfirmDelete(true)}
            className="text-fg-muted hover:text-danger transition-colors text-xs px-2 py-1 rounded hover:bg-surface-2 shrink-0"
            aria-label="Delete workout"
          >
            Delete
          </button>
        )}
      </div>

      <div className="divide-y divide-border/40">
        {workout.entries.map((entry, idx) => (
          <SwipeToRemove
            key={`${entry.exerciseId}-${idx}`}
            onRemove={() =>
              void useStore
                .getState()
                .removeExerciseFromWorkout(workout.id, entry.exerciseId)
            }
          >
            <ExerciseEntry
              workoutId={workout.id}
              entry={entry}
              exercise={exercises.find((e) => e.id === entry.exerciseId)}
            />
          </SwipeToRemove>
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

// ---------------------------------------------------------------------------
// Swipe-to-remove wrapper (touch devices only, X button stays on desktop)
// ---------------------------------------------------------------------------

function SwipeToRemove({
  onRemove,
  children,
}: {
  onRemove: () => void;
  children: ReactNode;
}) {
  const [offsetX, setOffsetX] = useState(0);
  const [removed, setRemoved] = useState(false);
  const swipingRef = useRef(false);
  const startX = useRef(0);
  const startY = useRef(0);
  const locked = useRef<"h" | "v" | null>(null);
  const currentOffset = useRef(0);
  const containerRef = useRef<HTMLDivElement>(null);
  const contentRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const el = contentRef.current;
    if (!el) return;

    function handleStart(e: TouchEvent) {
      startX.current = e.touches[0].clientX;
      startY.current = e.touches[0].clientY;
      locked.current = null;
      swipingRef.current = true;
      currentOffset.current = 0;
    }

    function handleMove(e: TouchEvent) {
      if (!swipingRef.current || !el) return;
      const dx = e.touches[0].clientX - startX.current;
      const dy = e.touches[0].clientY - startY.current;

      if (!locked.current) {
        if (Math.abs(dx) > 8 || Math.abs(dy) > 8) {
          locked.current = Math.abs(dx) > Math.abs(dy) ? "h" : "v";
        }
        return;
      }
      if (locked.current === "v") return;

      e.preventDefault();
      const clamped = Math.min(0, dx);
      currentOffset.current = clamped;
      el.style.transform = `translateX(${clamped}px)`;
      el.style.transition = "none";
      setOffsetX(clamped);
    }

    function handleEnd() {
      if (!swipingRef.current || !el) return;
      swipingRef.current = false;
      el.style.transition = "";

      if (locked.current !== "h") {
        currentOffset.current = 0;
        el.style.transform = "translateX(0)";
        setOffsetX(0);
        return;
      }

      const width = containerRef.current?.offsetWidth ?? 300;
      if (Math.abs(currentOffset.current) > width * 0.4) {
        el.style.transform = "translateX(-100%)";
        setRemoved(true);
        setTimeout(onRemove, 180);
      } else {
        currentOffset.current = 0;
        el.style.transform = "translateX(0)";
        setOffsetX(0);
      }
    }

    el.addEventListener("touchstart", handleStart, { passive: true });
    el.addEventListener("touchmove", handleMove, { passive: false });
    el.addEventListener("touchend", handleEnd, { passive: true });
    return () => {
      el.removeEventListener("touchstart", handleStart);
      el.removeEventListener("touchmove", handleMove);
      el.removeEventListener("touchend", handleEnd);
    };
  }, [onRemove]);

  const showBg = offsetX < -4 || removed;
  const pct = containerRef.current
    ? Math.min(Math.abs(offsetX) / containerRef.current.offsetWidth, 1)
    : 0;

  return (
    <div ref={containerRef} className="relative overflow-hidden">
      {showBg && (
        <div
          className="absolute inset-0 flex items-center justify-end pr-5"
          style={{
            backgroundColor: `oklch(0.55 ${0.12 + pct * 0.1} 27)`,
          }}
        >
          <span className="text-white text-xs font-semibold tracking-wide">
            Remove
          </span>
        </div>
      )}
      <div
        ref={contentRef}
        className="relative bg-surface-1 transition-transform duration-180 ease-out"
      >
        {children}
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Workout name (inline-editable)
// ---------------------------------------------------------------------------

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

  async function save() {
    try {
      await updateWorkoutName(workout.id, name.trim());
      setEditing(false);
    } catch {
      /* syncError set in store */
    }
  }

  if (editing) {
    return (
      <input
        ref={inputRef}
        value={name}
        onChange={(e) => setName(e.target.value)}
        onBlur={() => void save()}
        onKeyDown={(e) => {
          if (e.key === "Enter") void save();
          if (e.key === "Escape") {
            setName(workout.name || "");
            setEditing(false);
          }
        }}
        className="bg-transparent text-base font-semibold text-fg outline-none border-b border-accent pb-0.5 min-w-0"
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

// ---------------------------------------------------------------------------
// Exercise entry (sets table + add-set form)
// ---------------------------------------------------------------------------

function ExerciseEntry({
  workoutId,
  entry,
  exercise,
}: {
  workoutId: string;
  entry: WorkoutExercise;
  exercise: Exercise | undefined;
}) {
  const removeExerciseFromWorkout = useStore(
    (s) => s.removeExerciseFromWorkout,
  );

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
          onClick={() =>
            void removeExerciseFromWorkout(workoutId, entry.exerciseId)
          }
          className="hidden md:flex text-fg-muted hover:text-danger transition-colors ml-2 shrink-0 items-center justify-center"
          aria-label="Remove exercise"
        >
          <svg
            width="14"
            height="14"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
          >
            <path d="M18 6L6 18M6 6l12 12" />
          </svg>
        </button>
      </div>

      {entry.sets.length > 0 && (
        <div className="mb-2">
          <div
            className={cn(
              "grid text-[0.6rem] uppercase tracking-wider text-fg-muted font-semibold mb-1",
              isWeighted
                ? "grid-cols-[2rem_1fr_1fr] gap-2 justify-items-center"
                : "grid-cols-[2rem_1fr] justify-items-center",
            )}
          >
            <span className="justify-self-start">Set</span>
            {isWeighted && <span>Weight</span>}
            <span>Reps</span>
          </div>
          {entry.sets.map((set, si) => (
            <SwipeToRemove
              key={si}
              onRemove={() =>
                void useStore
                  .getState()
                  .removeSet(workoutId, entry.exerciseId, si)
              }
            >
              <SetRow
                workoutId={workoutId}
                exerciseId={entry.exerciseId}
                set={set}
                index={si}
                isWeighted={isWeighted}
              />
            </SwipeToRemove>
          ))}
        </div>
      )}

      <AddSetForm
        workoutId={workoutId}
        exerciseId={entry.exerciseId}
        isWeighted={isWeighted}
        lastSet={entry.sets.at(-1)}
      />
    </div>
  );
}

// ---------------------------------------------------------------------------
// Editable set row
// ---------------------------------------------------------------------------

function SetRow({
  workoutId,
  exerciseId,
  set,
  index,
  isWeighted,
}: {
  workoutId: string;
  exerciseId: string;
  set: PerformedSet;
  index: number;
  isWeighted: boolean;
}) {
  const updateSet = useStore((s) => s.updateSet);

  function commit(field: "weight" | "reps", raw: string) {
    const num = parseFloat(raw);
    if (isNaN(num) || num <= 0) return;

    let newSet: PerformedSet;
    if (field === "weight" && set.kind.type === "Weighted") {
      newSet = {
        ...set,
        kind: {
          type: "Weighted",
          weight: { value: num, units: set.kind.weight.units },
        },
      };
    } else if (field === "reps") {
      newSet = { ...set, reps: Math.round(num) };
    } else {
      return;
    }

    void updateSet(workoutId, exerciseId, index, newSet);
  }

  return (
    <div
      className={cn(
        "grid items-center py-2 transition-colors",
        isWeighted
          ? "grid-cols-[2rem_1fr_1fr] gap-2 justify-items-center"
          : "grid-cols-[2rem_1fr] justify-items-center",
      )}
    >
      <span className="text-fg-muted text-xs font-medium justify-self-start tabular-nums pl-1">
        {index + 1}
      </span>

      {isWeighted && set.kind.type === "Weighted" && (
        <EditableCell
          value={String(set.kind.weight.value)}
          suffix={set.kind.weight.units}
          inputMode="decimal"
          onCommit={(v) => commit("weight", v)}
        />
      )}

      <EditableCell
        value={String(set.reps)}
        suffix="reps"
        inputMode="numeric"
        onCommit={(v) => commit("reps", v)}
      />
    </div>
  );
}

function EditableCell({
  value,
  suffix,
  inputMode,
  onCommit,
}: {
  value: string;
  suffix: string;
  inputMode: "decimal" | "numeric";
  onCommit: (raw: string) => void;
}) {
  const ref = useRef<HTMLInputElement>(null);
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState(value);

  useEffect(() => {
    if (!editing) setDraft(value);
  }, [value, editing]);

  function handleFocus() {
    setDraft("");
    setEditing(true);
  }

  function handleBlur() {
    setEditing(false);
    const committed = draft.trim() === "" ? value : draft;
    if (committed !== value) onCommit(committed);
  }

  return (
    <div className="relative min-w-[4rem] text-center">
      <input
        ref={ref}
        type="number"
        inputMode={inputMode}
        step={inputMode === "decimal" ? "0.5" : "1"}
        value={editing ? draft : value}
        placeholder={editing ? value : undefined}
        onFocus={handleFocus}
        onBlur={handleBlur}
        onChange={(e) => setDraft(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === "Enter") ref.current?.blur();
          if (e.key === "Escape") {
            setDraft(value);
            setEditing(false);
            ref.current?.blur();
          }
        }}
        className={cn(
          "inline-edit w-full rounded-lg px-3 py-1.5 text-sm font-medium text-fg tabular-nums text-center transition-colors",
          editing
            ? "bg-surface-2 border border-accent/50 outline-none placeholder:text-fg-muted/50"
            : "bg-surface-2/60 border border-border/60 cursor-text hover:border-accent/40 hover:bg-surface-2",
        )}
      />
      {!editing && (
        <span className="pointer-events-none absolute right-1.5 top-1/2 -translate-y-1/2 text-fg-muted text-[0.65rem]">
          {suffix}
        </span>
      )}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Add-set form
// ---------------------------------------------------------------------------

function AddSetForm({
  workoutId,
  exerciseId,
  isWeighted,
  lastSet,
}: {
  workoutId: string;
  exerciseId: string;
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

    void addSet(workoutId, exerciseId, newSet);
    setReps("");
  }

  return (
    <form onSubmit={handleSubmit} className="flex items-center gap-2 mt-1">
      <button
        type="submit"
        className="bg-accent/15 text-accent text-xs font-semibold px-3 py-1.5 rounded-lg hover:bg-accent/25 transition-colors"
      >
        + Set
      </button>
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
    </form>
  );
}
