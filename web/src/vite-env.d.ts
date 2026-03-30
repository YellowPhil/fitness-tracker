/// <reference types="vite/client" />

interface ImportMetaEnv {
  /** API origin (e.g. `http://127.0.0.1:3001`). Empty = same origin (Vite proxy or backend-served SPA). */
  readonly VITE_API_BASE?: string;
  /** Sent as `x-user-id` (must match a user in the DB). */
  readonly VITE_USER_ID?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
