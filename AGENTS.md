# AGENTS.md

## Project Overview

Fitness Tracker is a multi-service workout logging product with three major parts:

- **Rust backend** for HTTP, SSE, gRPC, Telegram bot, and Postgres access
- **React frontend** in `web/` for the Telegram Mini App UI
- **Python service** in `services/workout-generator-py/` for AI workout generation

The repo follows a layered architecture. Keep business rules isolated from transport, storage, and framework code.

## Repository Layout

- `crates/domain/` — pure domain types, traits, and business rules
- `crates/application/` — use-case orchestration over domain abstractions
- `crates/infra/` — HTTP handlers, repositories, gRPC, bot, and external integrations
- `crates/fitness-tracker-proto/` — shared protobuf definitions
- `src/bin/backend.rs` — backend binary entrypoint
- `src/bin/bot.rs` — Telegram bot entrypoint
- `web/` — React + TypeScript frontend
- `services/workout-generator-py/` — Python AI generation service
- `deploy/`, `compose.yaml`, `compose.dev.yaml` — container and deployment assets

## Architectural Rules

- Dependency direction is **domain → application → infra**
- **Domain must stay pure**: no DB, HTTP, gRPC, Telegram, or framework concerns
- **Application uses domain abstractions**, not concrete infrastructure implementations
- **Infra owns adapters**: persistence, transport, serialization, and external service integration
- Keep backend DTOs aligned with frontend wire types

## Development Commands

### Rust

- `cargo run --bin backend` — run backend server
- `cargo run --bin bot` — run Telegram bot
- `cargo build --release` — production build
- `cargo test` — run Rust tests

### Frontend

- `cd web && npm ci` — install frontend dependencies
- `cd web && npm run dev` — run Vite dev server
- `cd web && npm run build` — build frontend
- `cd web && npm run cf-typegen` — generate Cloudflare types

### Python Service

- `cd services/workout-generator-py && pytest` — run Python tests

### Docker

- `docker compose -f compose.dev.yaml up --build` — local full-stack development
- `docker compose -f compose.yaml up` — production-style stack

## Testing Guidance

- **Rust**: use `cargo test`; tests usually live inline in `#[cfg(test)]` modules
- **Python**: use `pytest`; tests are split between `tests/unit/` and `tests/integration/`
- **Frontend**: no test runner is currently configured, so validate via build and targeted manual checks
- Add or update tests when changing Rust or Python behavior

## Code Style

- **Rust**: types in `PascalCase`; functions, modules, and files in `snake_case`
- **TypeScript**: React components in `PascalCase`; helpers, hooks, and store methods in `camelCase`
- **Python**: `snake_case` throughout
- Do **not** write comments inside function bodies
- Preserve the repo’s small public surfaces through explicit re-exports at crate roots

## Existing Patterns To Follow

- **Repository pattern**: traits in domain, implementations in infra repos
- **Wire mapping at boundaries**: HTTP/gRPC layers translate between domain models and DTOs
- **Central frontend state**: `web/src/store.ts` owns fetching, mutations, and generation stream lifecycle
- **Auth mode split**: production uses Telegram init data; local dev may use `DEV_SKIP_AUTH=1`
- **Tracing/logging**: Rust uses `tracing`; Python uses `structlog`

## Important Constraints

- Preserve existing public API strings and enum values
- Keep frontend types and backend DTOs in sync
- If adding config, update relevant `.env.example` files and compose files
- Do not rename historical `excercise` / `excercies` paths unless doing a deliberate repo-wide migration

## Deployment Notes

- Pushes to `main` trigger deployment via GitHub Actions and Coolify webhook
- Frontend is built separately for Cloudflare Pages
- Dockerfile supports multiple targets for backend, combined app, and bot workloads

## Agent Workflow Expectations

- Read the nearest architecture and style guidance before making changes
- Prefer minimal, layer-respecting changes over broad rewrites
- Verify the specific surface you changed instead of running unrelated commands
- Do not introduce new architectural patterns when an existing one already fits
