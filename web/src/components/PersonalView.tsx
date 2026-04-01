import { useEffect, useState } from "react";
import { useStore } from "../store";
import { cn } from "../utils";
import type { HeightUnits, WeightUnits } from "../types";

export function PersonalView() {
  const profile = useStore((s) => s.profile);
  const profileLoading = useStore((s) => s.profileLoading);
  const refreshProfile = useStore((s) => s.refreshProfile);
  const updateProfile = useStore((s) => s.updateProfile);
  const updateWeight = useStore((s) => s.updateWeight);

  const [quickWeight, setQuickWeight] = useState("");
  const [weightUnits, setWeightUnits] = useState<WeightUnits>("kg");

  const [editingDetails, setEditingDetails] = useState(false);
  const [formAge, setFormAge] = useState("");
  const [formHeight, setFormHeight] = useState("");
  const [formHeightUnits, setFormHeightUnits] = useState<HeightUnits>("cm");
  const [formWeightUnits, setFormWeightUnits] = useState<WeightUnits>("kg");
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    void refreshProfile();
  }, [refreshProfile]);

  useEffect(() => {
    if (!profile) return;
    setQuickWeight(String(profile.weight.value));
    setWeightUnits(profile.weight.units);
    setFormAge(String(profile.age));
    setFormHeight(String(profile.height.value));
    setFormHeightUnits(profile.height.units);
    setFormWeightUnits(profile.weight.units);
  }, [profile]);

  async function handleQuickWeight() {
    const v = parseFloat(quickWeight);
    if (isNaN(v) || v <= 0) return;
    try {
      await updateWeight(v, weightUnits);
    } catch {
      /* error is surfaced via syncError */
    }
  }

  function stepWeight(delta: number) {
    const current = parseFloat(quickWeight) || 0;
    const step = weightUnits === "kg" ? 0.5 : 1;
    const next = Math.max(0, current + delta * step);
    setQuickWeight(String(parseFloat(next.toFixed(1))));
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

          <div className="relative">
            <input
              type="number"
              inputMode="decimal"
              value={quickWeight}
              onChange={(e) => setQuickWeight(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") void handleQuickWeight();
              }}
              className="w-28 text-center text-3xl font-bold text-fg bg-transparent outline-none tabular-nums"
              step={weightUnits === "kg" ? 0.5 : 1}
              min={0}
            />
            <span className="absolute -right-8 bottom-1 text-sm font-medium text-fg-muted">
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

        <div className="flex items-center justify-center gap-2 mb-4">
          {(["kg", "lbs"] as WeightUnits[]).map((u) => (
            <button
              key={u}
              type="button"
              onClick={() => setWeightUnits(u)}
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

        <button
          type="button"
          onClick={() => void handleQuickWeight()}
          className="w-full py-2.5 rounded-xl text-sm font-semibold bg-accent text-white hover:bg-accent-bright transition-colors"
        >
          Log Weight
        </button>
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
