import { useState } from "react";
import { useStore } from "../store";
import { MUSCLE_GROUPS, type MuscleGroup } from "../types";
import { Modal } from "./Modal";

interface Props {
  open: boolean;
  onClose: () => void;
  date: string;
}

const DEFAULT_MAX = 5;
const MIN_MAX = 1;
const MAX_MAX = 12;

export function GenerateWorkoutModal({ open, onClose, date }: Props) {
  const generateWorkout = useStore((s) => s.generateWorkout);
  const generationJobsById = useStore((s) => s.generationJobsById);
  const activeGenerationJobId = useStore((s) => s.activeGenerationJobId);
  const activeJob = activeGenerationJobId
    ? generationJobsById[activeGenerationJobId]
    : undefined;
  const loading =
    !!activeJob &&
    activeJob.date === date &&
    (activeJob.status === "queued" || activeJob.status === "running");

  const [selected, setSelected] = useState<MuscleGroup[]>([]);
  const [maxCount, setMaxCount] = useState(DEFAULT_MAX);
  const [error, setError] = useState<string | null>(null);

  function toggleGroup(g: MuscleGroup) {
    setSelected((prev) =>
      prev.includes(g) ? prev.filter((x) => x !== g) : [...prev, g],
    );
  }

  function decrement() {
    setMaxCount((n) => Math.max(MIN_MAX, n - 1));
  }

  function increment() {
    setMaxCount((n) => Math.min(MAX_MAX, n + 1));
  }

  async function handleGenerate() {
    if (selected.length === 0 || loading) return;
    setError(null);
    try {
      await generateWorkout(selected, maxCount, date);
      handleClose();
    } catch (e) {
      const raw = e instanceof Error ? e.message : String(e);
      setError(
        raw.includes("OPENAI_API_KEY")
          ? "AI generation is not available (server not configured)."
          : raw,
      );
    }
  }

  function handleClose() {
    if (loading) return;
    setSelected([]);
    setMaxCount(DEFAULT_MAX);
    setError(null);
    onClose();
  }

  return (
    <Modal open={open} onClose={handleClose} title="Generate with AI">
      <div className="space-y-6">
        {/* Muscle groups */}
        <div>
          <p className="text-xs font-semibold uppercase tracking-wider text-fg-muted mb-3">
            Target muscle groups
          </p>
          <div className="flex flex-wrap gap-2">
            {MUSCLE_GROUPS.map((g) => {
              const active = selected.includes(g);
              return (
                <button
                  key={g}
                  type="button"
                  disabled={loading}
                  onClick={() => toggleGroup(g)}
                  className={`px-3 py-1.5 rounded-lg text-sm font-medium transition-colors ${
                    active
                      ? "bg-accent text-white shadow-sm shadow-accent/30"
                      : "bg-surface-2 text-fg-secondary hover:bg-surface-3 hover:text-fg"
                  } disabled:opacity-50 disabled:cursor-not-allowed`}
                >
                  {g}
                </button>
              );
            })}
          </div>
          {selected.length === 0 && (
            <p className="text-xs text-fg-muted mt-2">
              Select at least one group.
            </p>
          )}
        </div>

        {/* Max exercise count */}
        <div>
          <p className="text-xs font-semibold uppercase tracking-wider text-fg-muted mb-3">
            Max exercises
          </p>
          <div className="flex items-center gap-3">
            <button
              type="button"
              disabled={loading || maxCount <= MIN_MAX}
              onClick={decrement}
              className="w-9 h-9 rounded-lg bg-surface-2 text-fg-secondary hover:bg-surface-3 hover:text-fg flex items-center justify-center text-lg font-semibold transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
              aria-label="Decrease"
            >
              −
            </button>
            <span className="w-8 text-center text-base font-semibold text-fg tabular-nums">
              {maxCount}
            </span>
            <button
              type="button"
              disabled={loading || maxCount >= MAX_MAX}
              onClick={increment}
              className="w-9 h-9 rounded-lg bg-surface-2 text-fg-secondary hover:bg-surface-3 hover:text-fg flex items-center justify-center text-lg font-semibold transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
              aria-label="Increase"
            >
              +
            </button>
          </div>
        </div>

        {/* Error */}
        {error && (
          <p className="text-sm text-danger bg-danger/10 rounded-lg px-3 py-2 animate-fade-in">
            {error}
          </p>
        )}

        {/* Generate button */}
        <button
          type="button"
          disabled={selected.length === 0 || loading}
          onClick={() => void handleGenerate()}
          className="w-full py-3 rounded-xl font-semibold text-sm transition-colors bg-accent hover:bg-accent-bright text-white shadow-lg shadow-accent/20 disabled:opacity-50 disabled:cursor-not-allowed disabled:shadow-none relative overflow-hidden"
        >
          {loading ? (
            <span className="flex items-center justify-center gap-2">
              <Spinner />
              Generating workout…
            </span>
          ) : (
            "Generate Workout"
          )}
        </button>
      </div>
    </Modal>
  );
}

function Spinner() {
  return (
    <svg
      className="animate-spin"
      width="16"
      height="16"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2.5"
      aria-hidden="true"
    >
      <path
        d="M12 2v4M12 18v4M4.93 4.93l2.83 2.83M16.24 16.24l2.83 2.83M2 12h4M18 12h4M4.93 19.07l2.83-2.83M16.24 7.76l2.83-2.83"
        strokeLinecap="round"
      />
    </svg>
  );
}
