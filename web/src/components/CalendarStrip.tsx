import { useState } from "react";
import { useStore } from "../store";
import { cn, toDateString } from "../utils";

const DAY_NAMES = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

export function CalendarStrip() {
  const [viewYear, setViewYear] = useState(() => new Date().getFullYear());
  const [viewMonth, setViewMonth] = useState(() => new Date().getMonth());
  const selectedDate = useStore((s) => s.selectedDate);
  const setSelectedDate = useStore((s) => s.setSelectedDate);
  const workouts = useStore((s) => s.workouts);

  const workoutDates = new Set(workouts.map((w) => w.startDate));
  const now = new Date();
  const todayStr = toDateString(now);
  const isViewingToday =
    selectedDate === todayStr &&
    viewYear === now.getFullYear() &&
    viewMonth === now.getMonth();

  function goToToday() {
    setSelectedDate(todayStr);
    setViewYear(now.getFullYear());
    setViewMonth(now.getMonth());
  }

  const firstOfMonth = new Date(viewYear, viewMonth, 1);
  const startOffset = (firstOfMonth.getDay() + 6) % 7;
  const daysInMonth = new Date(viewYear, viewMonth + 1, 0).getDate();

  const cells: { dateStr: string; day: number; inMonth: boolean }[] = [];

  for (let i = startOffset - 1; i >= 0; i--) {
    const d = new Date(viewYear, viewMonth, -i);
    cells.push({ dateStr: toDateString(d), day: d.getDate(), inMonth: false });
  }
  for (let i = 1; i <= daysInMonth; i++) {
    const d = new Date(viewYear, viewMonth, i);
    cells.push({ dateStr: toDateString(d), day: i, inMonth: true });
  }
  while (cells.length % 7 !== 0) {
    const overflow = cells.length - startOffset - daysInMonth + 1;
    const d = new Date(viewYear, viewMonth + 1, overflow);
    cells.push({ dateStr: toDateString(d), day: d.getDate(), inMonth: false });
  }

  function prevMonth() {
    if (viewMonth === 0) {
      setViewYear((y) => y - 1);
      setViewMonth(11);
    } else {
      setViewMonth((m) => m - 1);
    }
  }

  function nextMonth() {
    if (viewMonth === 11) {
      setViewYear((y) => y + 1);
      setViewMonth(0);
    } else {
      setViewMonth((m) => m + 1);
    }
  }

  const monthLabel = firstOfMonth.toLocaleDateString("en-US", {
    month: "long",
    year: "numeric",
  });

  return (
    <section className="px-4 pt-4 pb-2" aria-label="Calendar">
      <div className="flex items-center justify-between mb-3">
        <button
          onClick={prevMonth}
          className="w-8 h-8 flex items-center justify-center rounded-lg text-fg-secondary hover:text-fg hover:bg-surface-2 transition-colors"
          aria-label="Previous month"
        >
          ‹
        </button>
        <div className="flex items-center gap-2">
          <h2 className="text-sm font-semibold text-fg-secondary tracking-wide">
            {monthLabel}
          </h2>
          {!isViewingToday && (
            <button
              onClick={goToToday}
              className="text-[0.65rem] font-semibold text-accent bg-accent/10 hover:bg-accent/20 px-2 py-0.5 rounded-full transition-colors"
            >
              Today
            </button>
          )}
        </div>
        <button
          onClick={nextMonth}
          className="w-8 h-8 flex items-center justify-center rounded-lg text-fg-secondary hover:text-fg hover:bg-surface-2 transition-colors"
          aria-label="Next month"
        >
          ›
        </button>
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

      <div className="grid grid-cols-7 gap-px">
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
                cell.inMonth && !isSelected && "text-fg-secondary hover:bg-surface-2",
                isSelected && "bg-accent text-white font-semibold shadow-lg shadow-accent/25",
                isToday && !isSelected && "ring-1 ring-accent/40 font-medium text-fg",
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
    </section>
  );
}
