# Architecture

## Overview
- `fitness-tracker` is a Telegram Mini App for logging workouts, tracking body metrics, and generating AI workout plans.
- It is a multi-language workspace: Rust backend + React frontend + Python workout-generation service.

## Tech Stack
- **Rust**: Axum, Tokio, SQLx, Tonic, Teloxide, tracing
- **Frontend**: React 19, TypeScript, Vite 6, Tailwind CSS 4, Zustand
- **Python service**: FastAPI-style package layout, gRPC (`grpcio`), OpenAI client, structlog, pytest
- **Data**: PostgreSQL
- **Transport**: HTTP/JSON, gRPC, SSE

## Directory Structure
```
fitness-tracker/
├── src/bin/backend.rs        # Rust HTTP + gRPC server
├── src/bin/bot.rs            # Telegram bot entry point
├── crates/
│   ├── domain/               # Pure types, traits, business rules
│   ├── application/          # Use-case orchestration
│   ├── infra/                # HTTP handlers, repos, gRPC, bot, AI integration
│   └── fitness-tracker-proto/ # Shared protobuf definitions
├── web/                      # React SPA
├── services/workout-generator-py/ # Python AI workout service
├── deploy/Dockerfile         # Multi-target container build
├── compose.yaml              # Production stack
├── compose.dev.yaml          # Dev stack
└── .github/workflows/deploy.yml
```

## Core Components

### Rust entry points
- `src/bin/backend.rs`
  - Initializes tracing (`fitness_tracker::init_tracing`).
  - Reads env config and connects to PostgreSQL.
  - Builds the Axum router from `infra::http_router`.
  - Starts HTTP and gRPC servers together.
- `src/bin/bot.rs`
  - Validates Telegram bot token presence.
  - Resolves the Mini App URL from `FRONTEND_URL` or `FITNESS_WEB_APP_URL`.
  - Delegates to `infra::bot::run_bot`.

### Rust layer split
- `crates/domain/`
  - Core domain types (`types/`), preferences, health, and repository traits (`traits.rs`).
  - No direct IO/framework dependencies.
- `crates/application/`
  - Use-case services: `GymApp`, `HealthApp`, `PreferencesApp`.
  - Coordinates repository traits and domain logic.
- `crates/infra/`
  - Concrete implementations:
    - `web/` Axum routes, auth, request/response mapping
    - `repos/` SQLx/Postgres repositories
    - `grpc/` Tonic services + gRPC client for generation
    - `generation/` job queueing, dispatch, SSE event bus
    - `bot/` Telegram bot integration
    - `ai/` formatting helpers for generation prompts/context

### Frontend
- `web/src/main.tsx` bootstraps the React app and calls `initTelegramApp()`.
- `web/src/App.tsx` provides the shell UI and bottom tab navigation.
- `web/src/store.ts` is the client state hub (Zustand) for workouts, exercises, profile, preferences, and generation jobs.
- `web/src/api.ts` is the API adapter and wire-type mapper.
- `web/src/telegram.ts` reads Telegram Mini App data and applies theme variables.

### Python generator
- `services/workout-generator-py/app/main.py` starts the gRPC server.
- `app/dependencies.py` wires the AI client, provider, and generation service.
- `app/application/services/workout_generation_service.py` (and helpers under `app/domain/`) implement the AI generation flow.

## Data Flow

### Normal app usage
1. Telegram opens the Mini App.
2. Frontend reads `window.Telegram.WebApp.initData` (`web/src/telegram.ts`).
3. Requests go to the Rust backend with `Authorization: tma ...` or `x-user-id` in dev (`web/src/api.ts`).
4. Axum handlers in `crates/infra/src/web/` validate input and call application services.
5. Application services use repository traits from `crates/domain/src/traits.rs`.
6. SQLx repositories in `crates/infra/src/repos/` persist to PostgreSQL.

### Workout generation flow
1. Frontend submits a generation request.
2. `crates/infra/src/generation/mod.rs` deduplicates the request and stores a generation job.
3. `crates/infra/src/generation/dispatcher.rs` calls the Python service over gRPC.
4. Python service requests extra context from the backend via gRPC `WorkoutDataService` / `HealthDataService` (`crates/infra/src/grpc/`).
5. The generator calls OpenAI, produces a workout, and returns it.
6. Backend persists the workout and emits job updates through `GenerationEventBus` + SSE (`crates/infra/src/web/workout_generation_jobs.rs`).
7. The frontend subscribes to the job stream and refreshes local state.

## External Integrations
- **Telegram**: Mini App auth and bot delivery (`src/bin/bot.rs`, `crates/infra/src/web/telegram_auth.rs`).
- **PostgreSQL**: primary storage for workouts, exercises, health, preferences, and generation jobs.
- **OpenAI**: used by `services/workout-generator-py`.
- **gRPC**: shared proto definitions in `crates/fitness-tracker-proto/`.
- **Docker/Coolify**: deployment via `compose.yaml`, `compose.dev.yaml`, and `.github/workflows/deploy.yml`.

## Configuration
- Root Rust workspace: `Cargo.toml`
- Frontend: `web/package.json`, `web/vite.config.ts`, `web/tsconfig*.json`
- Python service: `services/workout-generator-py/pyproject.toml`
- Environment templates: `.env.example`, `services/workout-generator-py/.env.example`, `.envrc`
- Key runtime env vars:
  - `POSTGRES_URL`
  - `TELOXIDE_TOKEN`
  - `BIND_ADDR`
  - `FRONTEND_URL`
  - `DEV_SKIP_AUTH`
  - `ALLOWED_USER_IDS`
  - `WORKOUT_GENERATOR_GRPC_URL`
  - `GRPC_BIND_ADDR`
  - `GRPC_TIMEOUT_SECONDS`
  - `WG_OPENAI_API_KEY`

## Build & Deploy
- **Backend**: `cargo run --bin backend`
- **Bot**: `cargo run --bin bot`
- **Frontend dev**: `cd web && npm run dev`
- **Frontend build**: `cd web && npm run build`
- **Full dev stack**: `docker compose -f compose.dev.yaml up --build`
- **Production stack**: `compose.yaml` plus separate frontend deployment
- **Deploy trigger**: `.github/workflows/deploy.yml` pings a Coolify webhook on `main`
