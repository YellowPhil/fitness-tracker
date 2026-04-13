import { useCallback, useEffect, useRef, useState } from "react";
import { useStore } from "../store";
import { cn } from "../utils";
import type {
  HeightUnits,
  TrainingGoal,
  WeightUnits,
  WorkoutPreferences,
  WorkoutSplit,
} from "../types";

const DEBOUNCE_MS = 600;

const WORKOUT_SPLIT_OPTIONS: { value: WorkoutSplit; label: string }[] = [
  { value: "FullBody", label: "Full Body" },
  { value: "PushPullLegs", label: "Push / Pull / Legs" },
  { value: "UpperLower", label: "Upper / Lower" },
];

const TRAINING_GOAL_OPTIONS: { value: TrainingGoal; label: string }[] = [
  { value: "Strength", label: "Strength" },
  { value: "Hypertrophy", label: "Hypertrophy" },
  { value: "Endurance", label: "Endurance" },
];

export function PersonalView() {
  const profile = useStore((s) => s.profile);
  const profileLoading = useStore((s) => s.profileLoading);
  const preferences = useStore((s) => s.preferences);
  const preferencesLoading = useStore((s) => s.preferencesLoading);
  const refreshProfile = useStore((s) => s.refreshProfile);
  const refreshPreferences = useStore((s) => s.refreshPreferences);
  const updateProfile = useStore((s) => s.updateProfile);
  const updateWeight = useStore((s) => s.updateWeight);
  const updatePreferences = useStore((s) => s.updatePreferences);

  const [quickWeight, setQuickWeight] = useState("");
  const [weightUnits, setWeightUnits] = useState<WeightUnits>("kg");

  const [editingDetails, setEditingDetails] = useState(false);
  const [formAge, setFormAge] = useState("");
  const [formHeight, setFormHeight] = useState("");
  const [formHeightUnits, setFormHeightUnits] = useState<HeightUnits>("cm");
  const [formWeightUnits, setFormWeightUnits] = useState<WeightUnits>("kg");
  const [saving, setSaving] = useState(false);

  const [editingPreferences, setEditingPreferences] = useState(false);
  const [prefMaxSets, setPrefMaxSets] = useState("");
  const [prefSplit, setPrefSplit] = useState<WorkoutSplit | "">("");
  const [prefGoal, setPrefGoal] = useState<TrainingGoal | "">("");
  const [prefSessionMinutes, setPrefSessionMinutes] = useState("");
  const [prefNotes, setPrefNotes] = useState("");
  const [preferencesSaving, setPreferencesSaving] = useState(false);
  const [preferencesValidationError, setPreferencesValidationError] =
    useState<string | null>(null);

  const debounceRef = useRef<ReturnType<typeof setTimeout>>(undefined);
  const isSynced = useRef(true);

  const flushWeight = useCallback(
    (value: string, units: WeightUnits) => {
      clearTimeout(debounceRef.current);
      const v = parseFloat(value);
      if (isNaN(v) || v <= 0) return;
      isSynced.current = false;
      debounceRef.current = setTimeout(() => {
        isSynced.current = true;
        void updateWeight(v, units).catch(() => {});
      }, DEBOUNCE_MS);
    },
    [updateWeight],
  );

  useEffect(() => {
    void Promise.all([refreshProfile(), refreshPreferences()]);
  }, [refreshProfile, refreshPreferences]);

  useEffect(() => {
    if (!profile || !isSynced.current) return;
    setQuickWeight(String(profile.weight.value));
    setWeightUnits(profile.weight.units);
    setFormAge(String(profile.age));
    setFormHeight(String(profile.height.value));
    setFormHeightUnits(profile.height.units);
    setFormWeightUnits(profile.weight.units);
  }, [profile]);

  useEffect(() => {
    if (editingPreferences) return;
    applyPreferenceForm(preferences, {
      setPrefMaxSets,
      setPrefSplit,
      setPrefGoal,
      setPrefSessionMinutes,
      setPrefNotes,
    });
  }, [preferences, editingPreferences]);

  useEffect(() => {
    return () => clearTimeout(debounceRef.current);
  }, []);

  function handleWeightChange(value: string) {
    setQuickWeight(value);
    flushWeight(value, weightUnits);
  }

  function handleUnitsChange(units: WeightUnits) {
    setWeightUnits(units);
    flushWeight(quickWeight, units);
  }

  function stepWeight(delta: number) {
    const current = parseFloat(quickWeight) || 0;
    const step = weightUnits === "kg" ? 0.5 : 1;
    const next = Math.max(0, current + delta * step);
    const nextStr = String(parseFloat(next.toFixed(1)));
    setQuickWeight(nextStr);
    flushWeight(nextStr, weightUnits);
  }

  async function handleSaveDetails() {
    if (!profile) return;
    const age = parseInt(formAge, 10);
    const height = parseFloat(formHeight);
    if (isNaN(age) || age <= 0 || isNaN(height) || height <= 0) return;

    setSaving(true);
    try {
      await updateProfile({
        weight: { value: parseFloat(quickWeight) || profile.weight.value, units: formWeightUnits },
        height: { value: height, units: formHeightUnits },
        age,
      });
      setEditingDetails(false);
    } catch {
      /* error is surfaced via syncError */
    } finally {
      setSaving(false);
    }
  }

  function handleEditPreferences() {
    applyPreferenceForm(preferences, {
      setPrefMaxSets,
      setPrefSplit,
      setPrefGoal,
      setPrefSessionMinutes,
      setPrefNotes,
    });
    setPreferencesValidationError(null);
    setEditingPreferences(true);
  }

  function handleCancelPreferences() {
    applyPreferenceForm(preferences, {
      setPrefMaxSets,
      setPrefSplit,
      setPrefGoal,
      setPrefSessionMinutes,
      setPrefNotes,
    });
    setPreferencesValidationError(null);
    setEditingPreferences(false);
  }

  async function handleSavePreferences() {
    const maxSets = parseOptionalPositiveInt(prefMaxSets);
    const sessionDuration = parseOptionalPositiveInt(prefSessionMinutes);
    if (maxSets === "invalid") {
      setPreferencesValidationError("Max sets must be a positive number.");
      return;
    }
    if (sessionDuration === "invalid") {
      setPreferencesValidationError(
        "Preferred session minutes must be a positive number.",
      );
      return;
    }

    setPreferencesValidationError(null);
    setPreferencesSaving(true);
    const payload: WorkoutPreferences = {
      maxSetsPerExercise: maxSets,
      preferredSplit: prefSplit === "" ? undefined : prefSplit,
      trainingGoal: prefGoal === "" ? undefined : prefGoal,
      sessionDurationMinutes: sessionDuration,
      notes: prefNotes.trim() === "" ? undefined : prefNotes.trim(),
    };

    try {
      await updatePreferences(payload);
      setEditingPreferences(false);
    } catch {
      setPreferencesValidationError(null);
    } finally {
      setPreferencesSaving(false);
    }
  }

  if (profileLoading && !profile) {
    return (
      <div className="flex items-center justify-center py-24">
        <div className="rounded-xl border border-border bg-surface-1 px-4 py-3 text-sm text-fg-secondary shadow-lg">
          Loading profile…
        </div>
      </div>
    );
  }

  return (
    <div className="px-4 py-6 space-y-6 animate-fade-in">
      {/* Quick weight update */}
      <section className="bg-surface-1 rounded-2xl border border-border p-5">
        <h2 className="text-xs font-semibold text-fg-muted uppercase tracking-wider mb-4">
          Current Weight
        </h2>

        <div className="flex items-center justify-center gap-3 mb-4">
          <button
            type="button"
            onClick={() => stepWeight(-1)}
            className="w-11 h-11 rounded-xl bg-surface-2 hover:bg-surface-3 text-fg-secondary hover:text-fg transition-colors flex items-center justify-center text-xl font-medium"
            aria-label="Decrease weight"
          >
            −
          </button>

          <div className="flex items-baseline gap-1.5">
            <input
              type="number"
              inputMode="decimal"
              value={quickWeight}
              onChange={(e) => handleWeightChange(e.target.value)}
              className="w-24 text-center text-3xl font-bold text-fg bg-transparent outline-none tabular-nums"
              step={weightUnits === "kg" ? 0.5 : 1}
              min={0}
            />
            <span className="text-sm font-medium text-fg-muted">
              {weightUnits}
            </span>
          </div>

          <button
            type="button"
            onClick={() => stepWeight(1)}
            className="w-11 h-11 rounded-xl bg-surface-2 hover:bg-surface-3 text-fg-secondary hover:text-fg transition-colors flex items-center justify-center text-xl font-medium"
            aria-label="Increase weight"
          >
            +
          </button>
        </div>

        <div className="flex items-center justify-center gap-2">
          {(["kg", "lbs"] as WeightUnits[]).map((u) => (
            <button
              key={u}
              type="button"
              onClick={() => handleUnitsChange(u)}
              className={cn(
                "px-3 py-1 rounded-lg text-xs font-medium transition-colors",
                weightUnits === u
                  ? "bg-accent text-white"
                  : "bg-surface-2 text-fg-secondary hover:bg-surface-3",
              )}
            >
              {u}
            </button>
          ))}
        </div>
      </section>

      {/* Body details */}
      <section className="bg-surface-1 rounded-2xl border border-border p-5">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-xs font-semibold text-fg-muted uppercase tracking-wider">
            Body Details
          </h2>
          {!editingDetails && (
            <button
              type="button"
              onClick={() => setEditingDetails(true)}
              className="text-xs font-medium text-accent hover:text-accent-bright transition-colors"
            >
              Edit
            </button>
          )}
        </div>

        {editingDetails ? (
          <div className="space-y-4 animate-slide-up">
            <div>
              <label className="block text-xs text-fg-muted mb-1.5">Age</label>
              <input
                type="number"
                inputMode="numeric"
                value={formAge}
                onChange={(e) => setFormAge(e.target.value)}
                className="w-full bg-surface-2 text-fg rounded-lg px-3 py-2.5 text-base outline-none border border-transparent focus:border-accent/50 transition-colors"
                placeholder="e.g. 25"
                min={1}
                max={150}
              />
            </div>

            <div>
              <label className="block text-xs text-fg-muted mb-1.5">
                Height
              </label>
              <div className="flex gap-2">
                <input
                  type="number"
                  inputMode="decimal"
                  value={formHeight}
                  onChange={(e) => setFormHeight(e.target.value)}
                  className="flex-1 bg-surface-2 text-fg rounded-lg px-3 py-2.5 text-base outline-none border border-transparent focus:border-accent/50 transition-colors"
                  placeholder={formHeightUnits === "cm" ? "e.g. 175" : "e.g. 69"}
                  min={0}
                />
                <div className="flex gap-1">
                  {(["cm", "in"] as HeightUnits[]).map((u) => (
                    <button
                      key={u}
                      type="button"
                      onClick={() => setFormHeightUnits(u)}
                      className={cn(
                        "px-3 py-2 rounded-lg text-xs font-medium transition-colors",
                        formHeightUnits === u
                          ? "bg-accent text-white"
                          : "bg-surface-2 text-fg-secondary hover:bg-surface-3",
                      )}
                    >
                      {u}
                    </button>
                  ))}
                </div>
              </div>
            </div>

            <div>
              <label className="block text-xs text-fg-muted mb-1.5">
                Preferred Weight Unit
              </label>
              <div className="flex gap-2">
                {(["kg", "lbs"] as WeightUnits[]).map((u) => (
                  <button
                    key={u}
                    type="button"
                    onClick={() => setFormWeightUnits(u)}
                    className={cn(
                      "flex-1 text-sm py-2 rounded-lg font-medium transition-colors",
                      formWeightUnits === u
                        ? "bg-accent text-white"
                        : "bg-surface-2 text-fg-secondary hover:bg-surface-3",
                    )}
                  >
                    {u}
                  </button>
                ))}
              </div>
            </div>

            <div className="flex gap-2 pt-1">
              <button
                type="button"
                onClick={() => setEditingDetails(false)}
                className="flex-1 text-sm py-2.5 rounded-lg font-medium bg-surface-2 text-fg-secondary hover:bg-surface-3 transition-colors"
              >
                Cancel
              </button>
              <button
                type="button"
                onClick={() => void handleSaveDetails()}
                disabled={saving}
                className="flex-1 text-sm py-2.5 rounded-lg font-semibold bg-accent text-white hover:bg-accent-bright transition-colors disabled:opacity-50"
              >
                {saving ? "Saving…" : "Save"}
              </button>
            </div>
          </div>
        ) : (
          <div className="space-y-3">
            <DetailRow label="Age" value={profile ? `${profile.age} years` : "—"} />
            <DetailRow
              label="Height"
              value={
                profile
                  ? `${profile.height.value} ${profile.height.units}`
                  : "—"
              }
            />
            <DetailRow
              label="Weight"
              value={
                profile
                  ? `${profile.weight.value} ${profile.weight.units}`
                  : "—"
              }
            />
          </div>
        )}
      </section>

      <section className="bg-surface-1 rounded-2xl border border-border p-5">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-xs font-semibold text-fg-muted uppercase tracking-wider">
            AI Workout Preferences
          </h2>
          {!editingPreferences && (
            <button
              type="button"
              onClick={handleEditPreferences}
              className="text-xs font-medium text-accent hover:text-accent-bright transition-colors"
            >
              Edit
            </button>
          )}
        </div>

        {editingPreferences ? (
          <div className="space-y-4 animate-slide-up">
            <div>
              <label className="block text-xs text-fg-muted mb-1.5">
                Max sets per exercise
              </label>
              <input
                type="number"
                inputMode="numeric"
                value={prefMaxSets}
                onChange={(e) => setPrefMaxSets(e.target.value)}
                className="w-full bg-surface-2 text-fg rounded-lg px-3 py-2.5 text-base outline-none border border-transparent focus:border-accent/50 transition-colors"
                placeholder="Optional"
                min={1}
              />
            </div>

            <div>
              <label className="block text-xs text-fg-muted mb-1.5">
                Preferred split
              </label>
              <select
                value={prefSplit}
                onChange={(e) => setPrefSplit(e.target.value as WorkoutSplit | "")}
                className="w-full bg-surface-2 text-fg rounded-lg px-3 py-2.5 text-base outline-none border border-transparent focus:border-accent/50 transition-colors"
              >
                <option value="">No preference</option>
                {WORKOUT_SPLIT_OPTIONS.map((option) => (
                  <option key={option.value} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
            </div>

            <div>
              <label className="block text-xs text-fg-muted mb-1.5">
                Training goal
              </label>
              <select
                value={prefGoal}
                onChange={(e) => setPrefGoal(e.target.value as TrainingGoal | "")}
                className="w-full bg-surface-2 text-fg rounded-lg px-3 py-2.5 text-base outline-none border border-transparent focus:border-accent/50 transition-colors"
              >
                <option value="">No preference</option>
                {TRAINING_GOAL_OPTIONS.map((option) => (
                  <option key={option.value} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
            </div>

            <div>
              <label className="block text-xs text-fg-muted mb-1.5">
                Preferred session minutes
              </label>
              <input
                type="number"
                inputMode="numeric"
                value={prefSessionMinutes}
                onChange={(e) => setPrefSessionMinutes(e.target.value)}
                className="w-full bg-surface-2 text-fg rounded-lg px-3 py-2.5 text-base outline-none border border-transparent focus:border-accent/50 transition-colors"
                placeholder="Optional"
                min={1}
              />
            </div>

            <div>
              <label className="block text-xs text-fg-muted mb-1.5">Notes</label>
              <textarea
                value={prefNotes}
                onChange={(e) => setPrefNotes(e.target.value)}
                className="w-full min-h-24 bg-surface-2 text-fg rounded-lg px-3 py-2.5 text-base outline-none border border-transparent focus:border-accent/50 transition-colors resize-y"
                placeholder="Optional coaching notes for AI-generated workouts"
              />
            </div>

            {preferencesValidationError && (
              <p className="text-sm text-danger bg-danger/10 rounded-lg px-3 py-2 animate-fade-in">
                {preferencesValidationError}
              </p>
            )}

            <div className="flex gap-2 pt-1">
              <button
                type="button"
                onClick={handleCancelPreferences}
                className="flex-1 text-sm py-2.5 rounded-lg font-medium bg-surface-2 text-fg-secondary hover:bg-surface-3 transition-colors"
              >
                Cancel
              </button>
              <button
                type="button"
                onClick={() => void handleSavePreferences()}
                disabled={preferencesSaving}
                className="flex-1 text-sm py-2.5 rounded-lg font-semibold bg-accent text-white hover:bg-accent-bright transition-colors disabled:opacity-50"
              >
                {preferencesSaving ? "Saving…" : "Save"}
              </button>
            </div>
          </div>
        ) : preferencesLoading && !preferences ? (
          <div className="rounded-lg bg-surface-2 px-3 py-3 text-sm text-fg-secondary">
            Loading preferences…
          </div>
        ) : (
          <div className="space-y-3">
            <DetailRow
              label="Max sets per exercise"
              value={formatNumber(preferences?.maxSetsPerExercise)}
            />
            <DetailRow
              label="Preferred split"
              value={formatSplit(preferences?.preferredSplit)}
            />
            <DetailRow
              label="Training goal"
              value={formatGoal(preferences?.trainingGoal)}
            />
            <DetailRow
              label="Preferred session"
              value={formatMinutes(preferences?.sessionDurationMinutes)}
            />
            <DetailRow label="Notes" value={formatNotes(preferences?.notes)} />
          </div>
        )}
      </section>
    </div>
  );
}

function DetailRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-center justify-between py-2 border-b border-border/40 last:border-0">
      <span className="text-sm text-fg-secondary">{label}</span>
      <span className="text-sm font-medium text-fg tabular-nums">{value}</span>
    </div>
  );
}

function parseOptionalPositiveInt(value: string): number | undefined | "invalid" {
  const trimmed = value.trim();
  if (trimmed === "") return undefined;
  const parsed = Number(trimmed);
  if (!Number.isFinite(parsed) || parsed <= 0 || !Number.isInteger(parsed)) return "invalid";
  return parsed;
}

function applyPreferenceForm(
  preferences: WorkoutPreferences | null,
  setters: {
    setPrefMaxSets: (value: string) => void;
    setPrefSplit: (value: WorkoutSplit | "") => void;
    setPrefGoal: (value: TrainingGoal | "") => void;
    setPrefSessionMinutes: (value: string) => void;
    setPrefNotes: (value: string) => void;
  },
) {
  setters.setPrefMaxSets(
    preferences?.maxSetsPerExercise != null
      ? String(preferences.maxSetsPerExercise)
      : "",
  );
  setters.setPrefSplit(preferences?.preferredSplit ?? "");
  setters.setPrefGoal(preferences?.trainingGoal ?? "");
  setters.setPrefSessionMinutes(
    preferences?.sessionDurationMinutes != null
      ? String(preferences.sessionDurationMinutes)
      : "",
  );
  setters.setPrefNotes(preferences?.notes ?? "");
}

function formatNumber(value: number | undefined): string {
  if (value == null) return "—";
  return String(value);
}

function formatSplit(value: WorkoutSplit | undefined): string {
  if (!value) return "—";
  return WORKOUT_SPLIT_OPTIONS.find((option) => option.value === value)?.label ?? "—";
}

function formatGoal(value: TrainingGoal | undefined): string {
  if (!value) return "—";
  return TRAINING_GOAL_OPTIONS.find((option) => option.value === value)?.label ?? "—";
}

function formatMinutes(value: number | undefined): string {
  if (value == null) return "—";
  return `${value} min`;
}

function formatNotes(value: string | undefined): string {
  if (!value || value.trim() === "") return "—";
  return value;
}
