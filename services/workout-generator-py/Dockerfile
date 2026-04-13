FROM python:3.11-slim

ENV PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1 \
    PIP_NO_CACHE_DIR=1 \
    PYTHONPATH=/app/app/generated

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends build-essential curl \
    && curl -L https://github.com/grpc-ecosystem/grpc-health-probe/releases/download/v0.4.38/grpc_health_probe-linux-$(uname -m) -o /usr/local/bin/grpc_health_probe \
    && chmod +x /usr/local/bin/grpc_health_probe \
    && rm -rf /var/lib/apt/lists/*

COPY services/workout-generator-py/pyproject.toml services/workout-generator-py/README.md ./
COPY services/workout-generator-py/app ./app
COPY crates/fitness-tracker-proto/proto ./proto

RUN mkdir -p app/generated/fitness_tracker \
    && python -m pip install "grpcio-tools>=1.80.0,<2.0.0" \
    && python -m grpc_tools.protoc \
      --proto_path=./proto \
      --python_out=./app/generated \
      --grpc_python_out=./app/generated \
      ./proto/fitness_tracker/common.proto \
      ./proto/fitness_tracker/health_data.proto \
      ./proto/fitness_tracker/workout_data.proto \
      ./proto/fitness_tracker/workout_generator.proto

RUN pip install --upgrade pip setuptools wheel \
    && pip install .

EXPOSE 50052

HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD grpc_health_probe -addr=127.0.0.1:50052 || exit 1

CMD ["python", "-m", "app.main"]
