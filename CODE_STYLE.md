# Code Style

## Naming Conventions
- **Rust types**: `PascalCase` (`Workout`, `WorkoutId`, `HealthApp`).
- **Rust functions/modules/files**: `snake_case` (`update_weight`, `workout_data_service.rs`).
- **Rust binaries**: `src/bin/backend.rs`, `src/bin/bot.rs`.
- **TypeScript components**: `PascalCase` (`WorkoutView`, `CalendarStrip`).
- **TypeScript helpers/hooks/state**: `camelCase` (`mapWorkoutFromApi`, `useStore`).
- **Python modules/functions**: `snake_case`.
- The repo uses some historical spellings like `excercise`/`excercies` in file names and module paths; keep existing names when touching those areas.

## File Organization
- Rust follows a layered workspace:
  - `crates/domain/` = core model + traits
  - `crates/application/` = orchestration/services
  - `crates/infra/` = DB, HTTP, gRPC, bot, AI plumbing
- `mod.rs` files are used heavily for module grouping and re-exports.
- Frontend code lives in `web/src/` with a shared `types.ts`, `api.ts`, `store.ts`, and `components/` directory.
- Python service code is split into `app/domain/`, `app/application/`, and `app/infrastructure/`.

## Import Style
- Rust imports are grouped by crate and then by module path.
- Public re-exports are used to present a small surface area from crate roots (`crates/application/src/lib.rs`, `crates/infra/src/lib.rs`).
- TypeScript uses direct named imports; `api.ts` and `types.ts` act as central modules.

## Code Patterns
- **Repository pattern**: domain traits in `crates/domain/src/traits.rs`, concrete Postgres implementations in `crates/infra/src/repos/`.
- **Newtype IDs**: domain IDs wrap `uuid::Uuid` (for example `WorkoutId`).
- **Wire-model mapping**: handlers convert between Rust domain types and API DTOs (`crates/infra/src/web/*`).
- **Async orchestration**: application and infrastructure code is predominantly async.
- **Generation jobs**: enqueue → dispatch → execute → persist → publish updates.
- **Frontend state**: Zustand store owns fetching, mutation methods, and generation stream lifecycle.

## Error Handling
- Rust uses `anyhow` for top-level/boundary errors and `thiserror` for typed internal errors.
- HTTP handlers return `ApiError` and validate inputs early.
- gRPC handlers return `tonic::Status` with `invalid_argument` / `internal` as needed.
- Python code uses custom exceptions plus `pytest.raises(...)` in tests.

## Logging
- Rust uses `tracing` and `#[instrument]` on most service/handler functions.
- Top-level binaries initialize tracing via `fitness_tracker::init_tracing()`.
- Python generator logs with `structlog` in JSON format (`services/workout-generator-py/app/infrastructure/logging.py`).

## Testing
- Rust tests are usually inline with `#[cfg(test)] mod tests`.
- Python tests use `pytest` with `pytest-asyncio`; files are named `test_*.py` under `tests/unit/` and `tests/integration/`.
- Frontend currently has no dedicated test runner configured in `web/package.json`.

## Do’s and Don’ts
- Do preserve existing public wire strings (for example workout source values and API enums).
- Do keep domain types free of IO/framework dependencies.
- Do keep API DTOs and frontend wire types aligned (`crates/infra/src/web/` ↔ `web/src/api.ts` / `web/src/types.ts`).
- Do use the existing `tracing`/`structlog` logging patterns.
- Don’t rename historical `excercise` paths unless you are planning a repo-wide migration.
- Don’t add new config conventions without matching them in both Docker compose files when relevant.
