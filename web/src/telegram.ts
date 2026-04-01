/**
 * Telegram Mini App bridge: `initData` for API auth and theme from `themeParams`.
 * @see https://core.telegram.org/bots/webapps
 */

export interface TelegramThemeParams {
  bg_color?: string;
  text_color?: string;
  hint_color?: string;
  link_color?: string;
  button_color?: string;
  button_text_color?: string;
  secondary_bg_color?: string;
  header_bg_color?: string;
  accent_text_color?: string;
  section_bg_color?: string;
  section_separator_color?: string;
  subtitle_text_color?: string;
}

export interface TelegramWebApp {
  initData: string;
  initDataUnsafe: unknown;
  version: string;
  platform: string;
  colorScheme: "light" | "dark";
  themeParams: TelegramThemeParams;
  isExpanded: boolean;
  viewportHeight: number;
  viewportStableHeight: number;
  ready: () => void;
  expand: () => void;
  setHeaderColor: (color: string) => void;
  setBackgroundColor: (color: string) => void;
}

declare global {
  interface Window {
    Telegram?: {
      WebApp: TelegramWebApp;
    };
  }
}

/** Raw `initData` query string for `Authorization: tma …` (empty outside Telegram). */
export function getInitData(): string {
  return window.Telegram?.WebApp?.initData?.trim() ?? "";
}

/** True when running inside the Telegram client (script injected and WebApp present). */
export function isTelegramMiniApp(): boolean {
  return Boolean(window.Telegram?.WebApp && getInitData().length > 0);
}

/**
 * Map Telegram theme colors onto app CSS variables (`@theme` in `index.css`).
 * No-op when not in Telegram or when params are missing.
 */
function applyTelegramTheme(): void {
  const w = window.Telegram?.WebApp;
  if (!w) return;

  const tp = w.themeParams;
  const root = document.documentElement;

  if (tp.bg_color) {
    root.style.setProperty("--color-surface-0", tp.bg_color);
  }
  if (tp.secondary_bg_color) {
    root.style.setProperty("--color-surface-1", tp.secondary_bg_color);
  }
  if (tp.section_bg_color) {
    root.style.setProperty("--color-surface-2", tp.section_bg_color);
  }
  if (tp.text_color) {
    root.style.setProperty("--color-fg", tp.text_color);
  }
  if (tp.subtitle_text_color) {
    root.style.setProperty("--color-fg-secondary", tp.subtitle_text_color);
  }
  if (tp.hint_color) {
    root.style.setProperty("--color-fg-muted", tp.hint_color);
  }
  if (tp.link_color || tp.accent_text_color) {
    const accent = tp.link_color ?? tp.accent_text_color;
    if (accent) {
      root.style.setProperty("--color-accent", accent);
      root.style.setProperty("--color-accent-bright", accent);
    }
  }
  if (tp.button_color) {
    root.style.setProperty("--color-accent-dim", tp.button_color);
  }
  if (tp.section_separator_color) {
    root.style.setProperty("--color-border", tp.section_separator_color);
  }

  if (tp.bg_color) {
    w.setHeaderColor(tp.bg_color);
    w.setBackgroundColor(tp.bg_color);
  }

  document.body.classList.add("telegram-mini-app");
}

/** Call once at startup: notify Telegram, expand viewport, apply theme. */
export function initTelegramApp(): void {
  const w = window.Telegram?.WebApp;
  if (!w) return;

  w.ready();
  w.expand();
  applyTelegramTheme();
}
