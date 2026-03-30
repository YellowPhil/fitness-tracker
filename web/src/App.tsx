import { useStore } from "./store";
import { CalendarStrip } from "./components/CalendarStrip";
import { WorkoutView } from "./components/WorkoutView";
import { ExerciseLibrary } from "./components/ExerciseLibrary";

export function App() {
  const currentView = useStore((s) => s.currentView);
  const setCurrentView = useStore((s) => s.setCurrentView);

  if (currentView === "exercises") {
    return <ExerciseLibrary onBack={() => setCurrentView("calendar")} />;
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

      <main className="max-w-lg mx-auto">
        <CalendarStrip />
        <WorkoutView />
      </main>
    </div>
  );
}
