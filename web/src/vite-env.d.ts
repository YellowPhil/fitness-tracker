/// <reference types="vite/client" />

interface ImportMetaEnv {
  /** API origin (e.g. `http://127.0.0.1:3001`). Empty = same origin (Vite proxy or backend-served SPA). */
  readonly VITE_API_BASE?: string;
  /** Dev only: when not opened in Telegram, sent as `x-user-id` (backend needs `DEV_SKIP_AUTH=1`). */
  readonly VITE_DEV_USER_ID?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
