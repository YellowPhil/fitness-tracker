import { useEffect } from "react";
import { useStore } from "./store";
import { CalendarStrip } from "./components/CalendarStrip";
import { WorkoutView } from "./components/WorkoutView";
import { ExerciseLibrary } from "./components/ExerciseLibrary";

export function App() {
  const currentView = useStore((s) => s.currentView);
  const setCurrentView = useStore((s) => s.setCurrentView);
  const bootstrap = useStore((s) => s.bootstrap);
  const isLoading = useStore((s) => s.isLoading);
  const syncError = useStore((s) => s.syncError);
  const clearSyncError = useStore((s) => s.clearSyncError);

  useEffect(() => {
    void bootstrap();
  }, [bootstrap]);

  const errorBanner =
    syncError && (
      <div className="max-w-lg mx-auto px-4 pt-2" role="alert">
        <div className="flex items-start gap-2 rounded-xl border border-danger/40 bg-danger/10 px-3 py-2 text-sm text-fg">
          <span className="flex-1">{syncError}</span>
          <button
            type="button"
            onClick={() => clearSyncError()}
            className="shrink-0 text-fg-muted hover:text-fg font-medium"
          >
            Dismiss
          </button>
        </div>
      </div>
    );

  if (currentView === "exercises") {
    return (
      <>
        {errorBanner}
        <ExerciseLibrary onBack={() => setCurrentView("calendar")} />
      </>
    );
  }

  return (
    <div className="min-h-dvh bg-surface-0">
      <header className="sticky top-0 z-20 bg-surface-0/80 backdrop-blur-xl border-b border-border">
        <div className="max-w-lg mx-auto px-4 h-14 flex items-center justify-between">
          <h1 className="text-lg font-bold tracking-tight text-fg">
            <span className="mr-1.5" aria-hidden="true">
              🏋️
            </span>
            GymTracker
          </h1>
          <button
            onClick={() => setCurrentView("exercises")}
            className="text-sm font-medium text-accent hover:text-accent-bright transition-colors px-3 py-1.5 rounded-lg hover:bg-surface-2"
          >
            Exercises
          </button>
        </div>
      </header>

      {errorBanner}

      <main className="max-w-lg mx-auto relative">
        {isLoading && (
          <div
            className="absolute inset-0 z-10 flex items-start justify-center pt-24 bg-surface-0/60 backdrop-blur-[2px]"
            aria-busy="true"
            aria-label="Loading"
          >
            <div className="rounded-xl border border-border bg-surface-1 px-4 py-3 text-sm text-fg-secondary shadow-lg">
              Loading…
            </div>
          </div>
        )}
        <CalendarStrip />
        <WorkoutView />
      </main>
    </div>
  );
}
