import { useEffect } from "react";
import { useStore } from "./store";
import { CalendarStrip } from "./components/CalendarStrip";
import { WorkoutView } from "./components/WorkoutView";
import { ExerciseLibrary } from "./components/ExerciseLibrary";
import { PersonalView } from "./components/PersonalView";
import { cn, scrollInputIntoView } from "./utils";

function CalendarIcon({ active }: { active: boolean }) {
  return (
    <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={active ? 2.2 : 1.6} className="transition-all">
      <rect x="3" y="4" width="18" height="18" rx="2" />
      <path d="M16 2v4M8 2v4M3 10h18" />
    </svg>
  );
}

function DumbbellIcon({ active }: { active: boolean }) {
  return (
    <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={active ? 2.2 : 1.6} className="transition-all">
      <path d="M6.5 6.5h11M6.5 17.5h11" />
      <rect x="2" y="5" width="4" height="14" rx="1.5" />
      <rect x="18" y="5" width="4" height="14" rx="1.5" />
      <path d="M12 5v14" />
    </svg>
  );
}

function PersonIcon({ active }: { active: boolean }) {
  return (
    <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={active ? 2.2 : 1.6} className="transition-all">
      <circle cx="12" cy="8" r="4" />
      <path d="M20 21a8 8 0 10-16 0" />
    </svg>
  );
}

const TABS = [
  { id: "calendar" as const, label: "Workouts", Icon: CalendarIcon },
  { id: "exercises" as const, label: "Exercises", Icon: DumbbellIcon },
  { id: "personal" as const, label: "Personal", Icon: PersonIcon },
];

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

  /** Mobile: scroll focused inputs into view when the keyboard opens / viewport shrinks. */
  useEffect(() => {
    const mq = window.matchMedia("(max-width: 768px)");
    if (!mq.matches) return;

    function isTextField(el: EventTarget | null): el is HTMLInputElement | HTMLTextAreaElement {
      if (!(el instanceof HTMLElement)) return false;
      if (el instanceof HTMLTextAreaElement) return true;
      if (el instanceof HTMLInputElement) {
        const t = el.type;
        return (
          t === "text" ||
          t === "search" ||
          t === "email" ||
          t === "number" ||
          t === "tel" ||
          t === "url" ||
          t === "password"
        );
      }
      return false;
    }

    const onFocusIn = (e: FocusEvent) => {
      if (isTextField(e.target)) scrollInputIntoView(e.target);
    };

    const onViewportChange = () => {
      const a = document.activeElement;
      if (isTextField(a)) scrollInputIntoView(a);
    };

    document.addEventListener("focusin", onFocusIn);
    window.visualViewport?.addEventListener("resize", onViewportChange);

    return () => {
      document.removeEventListener("focusin", onFocusIn);
      window.visualViewport?.removeEventListener("resize", onViewportChange);
    };
  }, []);

  const errorBanner = syncError && (
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

  return (
    <div className="min-h-dvh bg-surface-0 pb-[4.5rem]">
      <header className="sticky top-0 z-20 bg-surface-0/80 backdrop-blur-xl border-b border-border">
        <div className="max-w-lg mx-auto px-4 h-14 flex items-center">
          <h1 className="text-lg font-bold tracking-tight text-fg">
            <span className="mr-1.5" aria-hidden="true">🏋️</span>
            GymTracker
          </h1>
        </div>
      </header>

      {errorBanner}

      <main className="max-w-lg mx-auto relative">
        {isLoading && currentView === "calendar" && (
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

        {currentView === "calendar" && (
          <>
            <CalendarStrip />
            <WorkoutView />
          </>
        )}
        {currentView === "exercises" && <ExerciseLibrary />}
        {currentView === "personal" && <PersonalView />}
      </main>

      {/* Bottom tab bar */}
      <nav className="fixed bottom-0 inset-x-0 z-30 bg-surface-0/90 backdrop-blur-xl border-t border-border">
        <div className="max-w-lg mx-auto flex">
          {TABS.map(({ id, label, Icon }) => {
            const active = currentView === id;
            return (
              <button
                key={id}
                type="button"
                onClick={() => setCurrentView(id)}
                className={cn(
                  "flex-1 flex flex-col items-center gap-0.5 py-2.5 transition-colors",
                  active ? "text-accent" : "text-fg-muted hover:text-fg-secondary",
                )}
                aria-current={active ? "page" : undefined}
              >
                <Icon active={active} />
                <span className="text-[0.65rem] font-medium tracking-wide">
                  {label}
                </span>
              </button>
            );
          })}
        </div>
      </nav>
    </div>
  );
}
