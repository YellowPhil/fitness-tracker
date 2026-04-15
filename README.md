# GymTracker

Personal workout tracker delivered as a Telegram Mini App. Log exercises, track sets, log body weight, and generate AI-powered workout plans — all from inside Telegram.

## What it does

- **Log workouts by date** — add exercises, sets (weighted or bodyweight), and notes. Navigate with a calendar strip.
- **Exercise library** — built-in catalog plus user-defined exercises, organized by muscle group (Chest, Back, Shoulders, Arms, Legs, Core).
- **AI workout generation** — pick target muscle groups and max exercise count; the server calls OpenAI to produce a complete workout plan.
- **Body profile** — track current weight (kg/lbs), height, and age with debounced auto-save.
- **Telegram authentication** — users sign in through the Mini App; `initData` is validated server-side using HMAC-SHA256.
- **Optional user allowlist** — restrict access to specific Telegram user IDs via `ALLOWED_USER_IDS`.

## Architecture

```
┌──────────────────┐       ┌──────────────────┐
│  Telegram Bot    │       │  React SPA       │
│  (bot binary)    │       │  (Pages/embedded)│
│  /start → MiniApp│       │                  │
└────────┬─────────┘       └────────┬─────────┘
         │                          │
         │ Auth: tma <initData>     │
         └───────────┬──────────────┘
                     ▼
            ┌──────────────────┐       ┌──────────────────┐
            │  Axum Backend    │──────▶│  PostgreSQL      │
            │  (backend binary)│       │  (per-user data) │
            └────────┬─────────┘       └──────────────────┘
                     │
                     │ gRPC: GenerateWorkout
                     ▼
            ┌──────────────────┐
            │ workout-generator│
            │ (Python service) │
            └────────┬─────────┘
                     │
                     │ OpenAI API (tool-calling)
                     ▼
            ┌──────────────────┐
            │ OpenAI API       │
            └──────────────────┘
                     ▲
                     │ gRPC: WorkoutDataService (tool data)
                     │
            ┌────────┴─────────┐
            │  Axum Backend    │
            └──────────────────┘
```

The Rust workspace is split into three crates following a layered architecture:

| Crate | Role |
|---|---|
| `domain` | Types, traits, and business rules (no IO or framework dependencies) |
| `application` | Use-case orchestration (`GymApp`, `HealthApp`) — depends only on domain traits |
| `infra` | Concrete implementations: Axum HTTP handlers, teloxide bot, SQLx repos, gRPC client/server |

Two binaries ship from `src/bin/`:

- **`backend`** — Axum HTTP server (REST API + optional static SPA serving)
- **`bot`** — Telegram bot that sends the Mini App URL on `/start`

Workout generation is handled by `services/workout-generator-py` (FastAPI + gRPC + OpenAI tool calling). The Rust backend sends generation requests to it over gRPC, and the Python service calls back into the backend's gRPC `WorkoutDataService` to fetch exercise/workout context used by tools.

## Tech stack

- **Backend:** Rust (edition 2024), Axum 0.8, SQLx (Postgres), teloxide 0.17, async-openai
- **Frontend:** React 19, TypeScript, Vite 6, Tailwind CSS 4, Zustand 5
- **Deployment:** Docker (multi-stage build via cargo-chef), Coolify, Cloudflare Pages (production frontend)

## Getting started

### Prerequisites

- Rust toolchain (matches edition 2024 — Rust 1.85+)
- Node.js 22+
- PostgreSQL instance
- A Telegram bot token from [@BotFather](https://t.me/BotFather)

### Local development

1. Copy the env file and fill in values:

   ```sh
   cp .env.example .env
   ```

   At minimum, set `POSTGRES_URL` and `TELOXIDE_TOKEN`. For local dev without Telegram auth, set `DEV_SKIP_AUTH=1` and `VITE_DEV_USER_ID=1`.

2. Start the backend:

   ```sh
   cargo run --bin backend
   ```

3. In another terminal, start the frontend dev server:

   ```sh
   cd web && npm ci && npm run dev
   ```

   Vite proxies `/api` requests to `http://127.0.0.1:3001` (see `web/vite.config.ts`).

4. (Optional) Start the bot:

   ```sh
   cargo run --bin bot
   ```

### Docker (full-stack dev mode)

```sh
docker compose -f compose.dev.yaml up --build
```

The `app` target builds the React SPA and embeds it in the backend image. The backend serves both the API and the SPA on the same origin (no CORS needed). Access at `http://localhost:3001`.

## Configuration

All config is via environment variables. See `.env.example` for the full list.

| Variable | Required | Description |
|---|---|---|
| `POSTGRES_URL` | Yes | `postgres://user:pass@host:5432/db` |
| `TELOXIDE_TOKEN` | Yes (prod) | Telegram bot token; used for both bot process and Mini App auth |
| `BIND_ADDR` | No | Default `0.0.0.0:3001` |
| `FRONTEND_URL` | Prod only | Public frontend URL; enables CORS for cross-origin setups |
| `OPENAI_API_KEY` | No | Enables `POST /api/v1/workouts/generate` (AI workout plans) |
| `ALLOWED_USER_IDS` | No | Comma-separated Telegram user IDs; unset = all users allowed |
| `RUST_LOG` | No | Default `info`; use `debug` during development |
| `DEV_SKIP_AUTH` | No | Accept `x-user-id` header without Telegram validation (local dev only) |

Frontend build-time variables (set before `npm run build`):

| Variable | Description |
|---|---|
| `VITE_API_BASE` | Backend URL for API calls; leave empty for same-origin |
| `VITE_DEV_USER_ID` | Sent as `x-user-id` when not in Telegram (requires `DEV_SKIP_AUTH=1`) |

## API overview

All endpoints require authentication (`Authorization: tma <initData>` or `x-user-id` in dev mode).

| Method | Path | Description |
|---|---|---|
| GET | `/api/exercises` | List exercises for the current user |
| POST | `/api/exercises` | Create a custom exercise |
| DELETE | `/api/exercises/:id` | Delete an exercise |
| GET | `/api/workouts?date=YYYY-MM-DD` | List workouts for a date |
| GET | `/api/workouts/dates?from=…&to=…` | Dates that have workouts |
| POST | `/api/workouts` | Create a workout |
| POST | `/api/v1/workouts/generate` | Queue AI workout generation (requires `OPENAI_API_KEY`) |
| PATCH | `/api/workouts/:id` | Update workout name |
| DELETE | `/api/workouts/:id` | Delete a workout |
| POST | `/api/workouts/:id/exercises` | Add exercise to workout |
| DELETE | `/api/workouts/:id/exercises/:eid` | Remove exercise from workout |
| POST | `/api/workouts/:id/exercises/:eid/sets` | Add a set |
| PUT | `/api/workouts/:id/exercises/:eid/sets/:idx` | Update a set |
| DELETE | `/api/workouts/:id/exercises/:eid/sets/:idx` | Remove a set |
| GET | `/api/profile` | Get user profile (weight, height, age) |
| PUT | `/api/profile` | Update full profile |
| PATCH | `/api/profile/weight` | Update weight only |
| GET | `/health` | Health check (returns 200) |

## Development

```sh
# Run backend
cargo run --bin backend

# Run bot
cargo run --bin bot

# Frontend dev server (with API proxy)
cd web && npm run dev

# Build frontend for production
cd web && npm run build
```

## Deployment

- **Production** (`main` branch): Push to `main` triggers a Coolify webhook (`.github/workflows/deploy.yml`). Backend and bot run as separate containers via `compose.yaml`. Frontend is built and deployed to Cloudflare Pages with `VITE_API_BASE` pointing to the backend.
- **Dev/staging**: `compose.dev.yaml` builds a single `app` container (backend + embedded SPA) and a `bot` container.

The Dockerfile uses three production targets:
- `api` — backend only (frontend served by Cloudflare Pages)
- `app` — backend + embedded SPA (dev/staging)
- `bot` — Telegram bot

## Known issues

- When adding a workout for a date other than today, it may overwrite today's workout instead.
