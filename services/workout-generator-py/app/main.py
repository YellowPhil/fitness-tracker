from __future__ import annotations

from contextlib import asynccontextmanager

import grpc
from fastapi import FastAPI, Request
from structlog.contextvars import bind_contextvars, clear_contextvars

from app.api.dependencies import get_generation_service, get_provider, get_settings
from app.api.error_handlers import register_error_handlers
from app.api.routers.health import router as health_router
from app.api.routers.workouts import router as workouts_router
from app.generated.fitness_tracker import workout_generator_pb2_grpc
from app.infrastructure.grpc.workout_generator_service import WorkoutGeneratorGrpcService
from app.infrastructure.logging import configure_logging, request_id_from_headers

configure_logging()


@asynccontextmanager
async def lifespan(app: FastAPI):
    settings = get_settings()
    grpc_server = grpc.aio.server()
    workout_generator_pb2_grpc.add_WorkoutGeneratorServiceServicer_to_server(
        WorkoutGeneratorGrpcService(get_generation_service()),
        grpc_server,
    )
    grpc_bind_address = f"{settings.grpc_server_host}:{settings.grpc_server_port}"
    bound_port = grpc_server.add_insecure_port(grpc_bind_address)
    if bound_port == 0:
        raise RuntimeError(f"Failed to bind gRPC server on {grpc_bind_address}")
    await grpc_server.start()
    try:
        yield
    finally:
        await grpc_server.stop(grace=5)
        await get_provider().close()


app = FastAPI(title="workout-generator-py", version="0.1.0", lifespan=lifespan)


@app.middleware("http")
async def request_context_middleware(request: Request, call_next):
    request_id = request_id_from_headers(request)
    clear_contextvars()
    bind_contextvars(request_id=request_id)
    request.state.request_id = request_id
    response = await call_next(request)
    response.headers["x-request-id"] = request_id
    return response


app.include_router(health_router)
app.include_router(workouts_router)
register_error_handlers(app)
