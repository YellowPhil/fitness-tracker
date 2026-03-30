import { useState, useRef, useCallback, useEffect } from "react";
import { useStore } from "../store";
import { cn, toDateString } from "../utils";

const DAY_NAMES = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
const SWIPE_THRESHOLD = 50;

type SlideDir = "left" | "right" | null;

export function CalendarStrip() {
  const [viewYear, setViewYear] = useState(() => new Date().getFullYear());
  const [viewMonth, setViewMonth] = useState(() => new Date().getMonth());
  const [slideDir, setSlideDir] = useState<SlideDir>(null);
  const selectedDate = useStore((s) => s.selectedDate);
  const setSelectedDate = useStore((s) => s.setSelectedDate);
  const calendarWorkoutDates = useStore((s) => s.calendarWorkoutDates);
  const setCalendarViewport = useStore((s) => s.setCalendarViewport);

  useEffect(() => {
    setCalendarViewport(viewYear, viewMonth);
  }, [viewYear, viewMonth, setCalendarViewport]);

  const workoutDates = new Set(calendarWorkoutDates);
  const now = new Date();
  const todayStr = toDateString(now);
  const isViewingToday =
    selectedDate === todayStr &&
    viewYear === now.getFullYear() &&
    viewMonth === now.getMonth();

  function animateAndNavigate(dir: SlideDir, navigate: () => void) {
    setSlideDir(dir);
    navigate();
    requestAnimationFrame(() => {
      setTimeout(() => setSlideDir(null), 200);
    });
  }

  const doPrevMonth = useCallback(() => {
    if (viewMonth === 0) {
      setViewYear((y) => y - 1);
      setViewMonth(11);
    } else {
      setViewMonth((m) => m - 1);
    }
  }, [viewMonth]);

  const doNextMonth = useCallback(() => {
    if (viewMonth === 11) {
      setViewYear((y) => y + 1);
      setViewMonth(0);
    } else {
      setViewMonth((m) => m + 1);
    }
  }, [viewMonth]);

  function prevMonth() {
    animateAndNavigate("right", doPrevMonth);
  }
  function nextMonth() {
    animateAndNavigate("left", doNextMonth);
  }

  function goToToday() {
    const dir =
      viewYear < now.getFullYear() ||
      (viewYear === now.getFullYear() && viewMonth < now.getMonth())
        ? "left"
        : "right";
    setSlideDir(dir as SlideDir);
    setSelectedDate(todayStr);
    setViewYear(now.getFullYear());
    setViewMonth(now.getMonth());
    requestAnimationFrame(() => {
      setTimeout(() => setSlideDir(null), 200);
    });
  }

  // --- swipe handling ---
  const touchStartX = useRef(0);
  const touchStartY = useRef(0);

  function onTouchStart(e: React.TouchEvent) {
    touchStartX.current = e.touches[0].clientX;
    touchStartY.current = e.touches[0].clientY;
  }

  function onTouchEnd(e: React.TouchEvent) {
    const dx = e.changedTouches[0].clientX - touchStartX.current;
    const dy = e.changedTouches[0].clientY - touchStartY.current;
    if (Math.abs(dx) < SWIPE_THRESHOLD || Math.abs(dy) > Math.abs(dx)) return;
    if (dx > 0) prevMonth();
    else nextMonth();
  }

  // --- calendar grid ---
  const firstOfMonth = new Date(viewYear, viewMonth, 1);
  const startOffset = (firstOfMonth.getDay() + 6) % 7;
  const daysInMonth = new Date(viewYear, viewMonth + 1, 0).getDate();

  const TOTAL_CELLS = 42; // always 6 rows — keeps grid height constant
  const cells: { dateStr: string; day: number; inMonth: boolean }[] = [];

  for (let i = startOffset - 1; i >= 0; i--) {
    const d = new Date(viewYear, viewMonth, -i);
    cells.push({ dateStr: toDateString(d), day: d.getDate(), inMonth: false });
  }
  for (let i = 1; i <= daysInMonth; i++) {
    const d = new Date(viewYear, viewMonth, i);
    cells.push({ dateStr: toDateString(d), day: i, inMonth: true });
  }
  while (cells.length < TOTAL_CELLS) {
    const overflow = cells.length - startOffset - daysInMonth + 1;
    const d = new Date(viewYear, viewMonth + 1, overflow);
    cells.push({ dateStr: toDateString(d), day: d.getDate(), inMonth: false });
  }

  const monthLabel = firstOfMonth.toLocaleDateString("en-US", {
    month: "long",
    year: "numeric",
  });

  const gridAnimation =
    slideDir === "left"
      ? "animate-swipe-in-left"
      : slideDir === "right"
        ? "animate-swipe-in-right"
        : "";

  return (
    <section className="px-4 pt-4 pb-2 overflow-hidden" aria-label="Calendar">
      <div className="flex items-center justify-center mb-3">
        <div className="flex items-center gap-2.5">
          <h2 className="text-sm font-semibold text-fg-secondary tracking-wide">
            {monthLabel}
          </h2>
          {!isViewingToday && (
            <button
              onClick={goToToday}
              className="text-[0.65rem] font-semibold text-accent hover:text-accent-bright border border-accent/30 hover:border-accent/60 px-2.5 py-0.5 rounded-full transition-colors"
            >
              Today
            </button>
          )}
        </div>
      </div>

      <div className="grid grid-cols-7 mb-1">
        {DAY_NAMES.map((d) => (
          <div
            key={d}
            className="text-center text-[0.65rem] font-semibold text-fg-muted uppercase tracking-wider py-1"
          >
            {d}
          </div>
        ))}
      </div>

      <div
        key={`${viewYear}-${viewMonth}`}
        className={cn("grid grid-cols-7 gap-px touch-pan-y", gridAnimation)}
        onTouchStart={onTouchStart}
        onTouchEnd={onTouchEnd}
      >
        {cells.map((cell) => {
          const isSelected = cell.dateStr === selectedDate;
          const isToday = cell.dateStr === todayStr;
          const hasWorkout = workoutDates.has(cell.dateStr);

          return (
            <button
              key={cell.dateStr}
              onClick={() => setSelectedDate(cell.dateStr)}
              className={cn(
                "relative flex flex-col items-center justify-center py-2 rounded-lg text-sm transition-all duration-150",
                !cell.inMonth && "text-fg-muted/30",
                cell.inMonth &&
                  !isSelected &&
                  "text-fg-secondary hover:bg-surface-2",
                isSelected &&
                  "bg-accent text-white font-semibold shadow-lg shadow-accent/25",
                isToday &&
                  !isSelected &&
                  "ring-1 ring-accent/40 font-medium text-fg",
              )}
            >
              {cell.day}
              {hasWorkout && (
                <span
                  className={cn(
                    "absolute bottom-1 w-1 h-1 rounded-full transition-colors",
                    isSelected ? "bg-white/80" : "bg-accent",
                  )}
                />
              )}
            </button>
          );
        })}
      </div>

      <div className="flex items-center justify-between mt-2 px-1">
        <button
          onClick={prevMonth}
          className="h-10 px-5 flex items-center justify-center gap-1 rounded-xl bg-surface-1 border border-border text-fg-secondary hover:text-fg hover:bg-surface-2 active:bg-surface-3 transition-colors text-sm"
          aria-label="Previous month"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
            <path d="M15 18l-6-6 6-6" />
          </svg>
          <span className="text-xs font-medium">Prev</span>
        </button>
        <button
          onClick={nextMonth}
          className="h-10 px-5 flex items-center justify-center gap-1 rounded-xl bg-surface-1 border border-border text-fg-secondary hover:text-fg hover:bg-surface-2 active:bg-surface-3 transition-colors text-sm"
          aria-label="Next month"
        >
          <span className="text-xs font-medium">Next</span>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
            <path d="M9 18l6-6-6-6" />
          </svg>
        </button>
      </div>
    </section>
  );
}
