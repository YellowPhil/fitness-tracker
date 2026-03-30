import { useState, type FormEvent } from "react";
import { useStore } from "../store";
import { cn } from "../utils";
import { MUSCLE_GROUPS, type ExerciseKind, type MuscleGroup } from "../types";

interface ExerciseLibraryProps {
  onBack: () => void;
}

export function ExerciseLibrary({ onBack }: ExerciseLibraryProps) {
  const exercises = useStore((s) => s.exercises);
  const addExercise = useStore((s) => s.addExercise);
  const deleteExercise = useStore((s) => s.deleteExercise);
  const [search, setSearch] = useState("");
  const [showForm, setShowForm] = useState(false);

  const [formName, setFormName] = useState("");
  const [formKind, setFormKind] = useState<ExerciseKind>("Weighted");
  const [formMuscle, setFormMuscle] = useState<MuscleGroup>("Chest");

  const filtered = exercises.filter(
    (e) => !search || e.name.toLowerCase().includes(search.toLowerCase()),
  );

  const grouped = MUSCLE_GROUPS.map((mg) => ({
    group: mg,
    items: filtered.filter((e) => e.muscleGroup === mg),
  })).filter((g) => g.items.length > 0);

  function handleSubmit(e: FormEvent) {
    e.preventDefault();
    const trimmed = formName.trim();
    if (!trimmed) return;
    addExercise(trimmed, formKind, formMuscle);
    setFormName("");
    setShowForm(false);
  }

  return (
    <div className="min-h-dvh bg-surface-0">
      <header className="sticky top-0 z-20 bg-surface-0/80 backdrop-blur-xl border-b border-border">
        <div className="max-w-lg mx-auto px-4 h-14 flex items-center gap-3">
          <button
            onClick={onBack}
            className="text-accent hover:text-accent-bright transition-colors text-sm font-medium flex items-center gap-1"
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M15 18l-6-6 6-6" />
            </svg>
            Back
          </button>
          <h1 className="text-lg font-bold tracking-tight text-fg">
            Exercise Library
          </h1>
        </div>
      </header>

      <main className="max-w-lg mx-auto px-4 py-4">
        <input
          type="text"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          placeholder="Search exercises..."
          className="w-full bg-surface-1 text-fg rounded-xl px-4 py-3 text-sm outline-none border border-border focus:border-accent/50 transition-colors placeholder:text-fg-muted mb-4"
        />

        <div className="space-y-6">
          {grouped.map(({ group, items }) => (
            <section key={group}>
              <h2 className="text-xs font-semibold text-fg-muted uppercase tracking-wider mb-2 px-1">
                {group}
              </h2>
              <div className="bg-surface-1 rounded-xl border border-border overflow-hidden divide-y divide-border/40">
                {items.map((exercise) => (
                  <div
                    key={exercise.id}
                    className="flex items-center justify-between px-4 py-3"
                  >
                    <div className="flex items-center gap-2 min-w-0">
                      <span className="text-sm font-medium text-fg truncate">
                        {exercise.name}
                      </span>
                      <span
                        className={cn(
                          "shrink-0 text-[0.6rem] uppercase tracking-wider px-1.5 py-0.5 rounded font-semibold",
                          exercise.kind === "Weighted"
                            ? "bg-accent/15 text-accent"
                            : "bg-success/15 text-success",
                        )}
                      >
                        {exercise.kind === "BodyWeight" ? "BW" : "Weighted"}
                      </span>
                    </div>
                    <div className="flex items-center gap-2 shrink-0">
                      <span
                        className={cn(
                          "text-[0.6rem] uppercase tracking-wider",
                          exercise.source === "BuiltIn"
                            ? "text-fg-muted"
                            : "text-accent",
                        )}
                      >
                        {exercise.source === "BuiltIn" ? "Built-in" : "Custom"}
                      </span>
                      {exercise.source === "UserDefined" && (
                        <button
                          onClick={() => deleteExercise(exercise.id)}
                          className="text-fg-muted hover:text-danger transition-colors"
                          aria-label={`Delete ${exercise.name}`}
                        >
                          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                            <path d="M3 6h18M8 6V4a2 2 0 012-2h4a2 2 0 012 2v2m3 0v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6h14" />
                          </svg>
                        </button>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            </section>
          ))}

          {grouped.length === 0 && (
            <p className="text-sm text-fg-muted text-center py-12">
              No exercises found
            </p>
          )}
        </div>

        <div className="mt-6">
          {showForm ? (
            <form
              onSubmit={handleSubmit}
              className="bg-surface-1 rounded-xl border border-border p-4 space-y-4 animate-slide-up"
            >
              <h3 className="text-sm font-semibold text-fg">
                New Custom Exercise
              </h3>

              <div>
                <label className="block text-xs text-fg-muted mb-1.5">
                  Name
                </label>
                <input
                  type="text"
                  value={formName}
                  onChange={(e) => setFormName(e.target.value)}
                  className="w-full bg-surface-2 text-fg rounded-lg px-3 py-2.5 text-sm outline-none border border-transparent focus:border-accent/50 transition-colors"
                  placeholder="e.g. Hammer Curl"
                  autoFocus
                />
              </div>

              <div>
                <label className="block text-xs text-fg-muted mb-1.5">
                  Type
                </label>
                <div className="flex gap-2">
                  {(["Weighted", "BodyWeight"] as ExerciseKind[]).map((k) => (
                    <button
                      key={k}
                      type="button"
                      onClick={() => setFormKind(k)}
                      className={cn(
                        "flex-1 text-sm py-2 rounded-lg font-medium transition-colors",
                        formKind === k
                          ? "bg-accent text-white"
                          : "bg-surface-2 text-fg-secondary hover:bg-surface-3",
                      )}
                    >
                      {k === "BodyWeight" ? "Body Weight" : "Weighted"}
                    </button>
                  ))}
                </div>
              </div>

              <div>
                <label className="block text-xs text-fg-muted mb-1.5">
                  Muscle Group
                </label>
                <div className="flex gap-1.5 flex-wrap">
                  {MUSCLE_GROUPS.map((mg) => (
                    <button
                      key={mg}
                      type="button"
                      onClick={() => setFormMuscle(mg)}
                      className={cn(
                        "px-3 py-1.5 rounded-lg text-xs font-medium transition-colors",
                        formMuscle === mg
                          ? "bg-accent text-white"
                          : "bg-surface-2 text-fg-secondary hover:bg-surface-3",
                      )}
                    >
                      {mg}
                    </button>
                  ))}
                </div>
              </div>

              <div className="flex gap-2 pt-1">
                <button
                  type="button"
                  onClick={() => setShowForm(false)}
                  className="flex-1 text-sm py-2.5 rounded-lg font-medium bg-surface-2 text-fg-secondary hover:bg-surface-3 transition-colors"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  className="flex-1 text-sm py-2.5 rounded-lg font-semibold bg-accent text-white hover:bg-accent-bright transition-colors"
                >
                  Create
                </button>
              </div>
            </form>
          ) : (
            <button
              onClick={() => setShowForm(true)}
              className="w-full bg-surface-1 border border-dashed border-border rounded-xl px-4 py-4 text-sm text-accent hover:text-accent-bright hover:border-accent/40 transition-colors font-medium"
            >
              + Add Custom Exercise
            </button>
          )}
        </div>
      </main>
    </div>
  );
}
