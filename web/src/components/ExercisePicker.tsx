import { useState } from "react";
import { useStore } from "../store";
import { cn } from "../utils";
import { MUSCLE_GROUPS, type MuscleGroup } from "../types";
import { Modal } from "./Modal";

interface ExercisePickerProps {
  workoutId: string;
  onClose: () => void;
}

export function ExercisePicker({ workoutId, onClose }: ExercisePickerProps) {
  const [search, setSearch] = useState("");
  const [muscleFilter, setMuscleFilter] = useState<MuscleGroup | null>(null);
  const exercises = useStore((s) => s.exercises);
  const addExerciseToWorkout = useStore((s) => s.addExerciseToWorkout);

  const filtered = exercises.filter((e) => {
    if (search && !e.name.toLowerCase().includes(search.toLowerCase()))
      return false;
    if (muscleFilter && e.muscleGroup !== muscleFilter) return false;
    return true;
  });

  function handleSelect(exerciseId: string) {
    addExerciseToWorkout(workoutId, exerciseId);
    onClose();
  }

  return (
    <Modal open onClose={onClose} title="Add Exercise">
      <div className="space-y-4">
        <input
          type="text"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          placeholder="Search exercises..."
          className="w-full bg-surface-2 text-fg rounded-lg px-3 py-2.5 text-sm outline-none border border-transparent focus:border-accent/50 transition-colors placeholder:text-fg-muted"
          autoFocus
        />

        <div className="flex gap-1.5 flex-wrap">
          <button
            onClick={() => setMuscleFilter(null)}
            className={cn(
              "px-3 py-1 rounded-full text-xs font-medium transition-colors",
              !muscleFilter
                ? "bg-accent text-white"
                : "bg-surface-2 text-fg-secondary hover:bg-surface-3",
            )}
          >
            All
          </button>
          {MUSCLE_GROUPS.map((mg) => (
            <button
              key={mg}
              onClick={() =>
                setMuscleFilter(muscleFilter === mg ? null : mg)
              }
              className={cn(
                "px-3 py-1 rounded-full text-xs font-medium transition-colors",
                muscleFilter === mg
                  ? "bg-accent text-white"
                  : "bg-surface-2 text-fg-secondary hover:bg-surface-3",
              )}
            >
              {mg}
            </button>
          ))}
        </div>

        <div className="space-y-1 max-h-64 overflow-y-auto">
          {filtered.length === 0 && (
            <p className="text-sm text-fg-muted text-center py-6">
              No exercises found
            </p>
          )}
          {filtered.map((exercise) => (
            <button
              key={exercise.id}
              onClick={() => handleSelect(exercise.id)}
              className="w-full flex items-center justify-between px-3 py-2.5 rounded-lg hover:bg-surface-2 transition-colors text-left group"
            >
              <div>
                <span className="text-sm font-medium text-fg group-hover:text-accent-bright transition-colors">
                  {exercise.name}
                </span>
                <span className="ml-2 text-[0.65rem] uppercase tracking-wider text-fg-muted">
                  {exercise.kind === "BodyWeight" ? "BW" : "WT"}
                </span>
              </div>
              <span className="text-[0.65rem] text-fg-muted bg-surface-2 px-2 py-0.5 rounded-full group-hover:bg-surface-3">
                {exercise.muscleGroup}
              </span>
            </button>
          ))}
        </div>
      </div>
    </Modal>
  );
}
