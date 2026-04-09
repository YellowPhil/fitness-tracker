# workout-generator-py

Python microservice for workout generation using a two-step OpenAI tool-calling flow.

## Features

- FastAPI endpoints:
  - `POST /v1/workouts/generate`
  - `GET /health/live`
  - `GET /health/ready`
- gRPC server:
  - `fitness_tracker.workout_generator.WorkoutGeneratorService/GenerateWorkout`
- Clean architecture layers: `api`, `domain`, `application`, `infrastructure`
- OpenAI adapter isolated behind a port
- Internal data provider adapter via gRPC
- Structured JSON logs with request IDs
- Unit and integration-style tests with fakes

## Protobuf / gRPC contract

This service consumes protobuf definitions from:

- `../../crates/fitness-tracker-proto/proto/fitness_tracker/*.proto`

Regenerate Python stubs after protobuf changes:

```bash
source .venv/bin/activate
python -m grpc_tools.protoc \
  --proto_path="../../crates/fitness-tracker-proto/proto" \
  --python_out="app/generated" \
  --grpc_python_out="app/generated" \
  "../../crates/fitness-tracker-proto/proto/fitness_tracker/common.proto" \
  "../../crates/fitness-tracker-proto/proto/fitness_tracker/workout_data.proto" \
  "../../crates/fitness-tracker-proto/proto/fitness_tracker/workout_generator.proto"
```

## Run locally

1. Create a virtualenv and install dependencies:

```bash
python3 -m venv .venv
source .venv/bin/activate
pip install -e ".[dev]"
```

2. Copy environment variables:

```bash
cp .env.example .env
```

3. Start the service (HTTP + gRPC in one process):

```bash
uvicorn app.main:app --reload --host 0.0.0.0 --port 8091
```

gRPC bind address is configured via `WG_GRPC_SERVER_HOST` and `WG_GRPC_SERVER_PORT`.

## Tests

```bash
pytest
```
