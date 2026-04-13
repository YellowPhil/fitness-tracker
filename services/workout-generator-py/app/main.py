from __future__ import annotations

import asyncio
import signal

import grpc
import grpc_health.v1.health
import grpc_health.v1.health_pb2
import grpc_health.v1.health_pb2_grpc
import structlog

from app.dependencies import get_generation_service, get_provider, get_settings
from app.infrastructure.grpc.workout_generator_service import WorkoutGeneratorGrpcService
from app.infrastructure.logging import configure_logging
from fitness_tracker import workout_generator_pb2_grpc

configure_logging()

logger = structlog.get_logger(__name__)

WORKOUT_GENERATOR_SERVICE_NAME = "fitness_tracker.workout_generator.WorkoutGeneratorService"


async def serve() -> None:
    settings = get_settings()

    grpc_server = grpc.aio.server()

    health_servicer = grpc_health.v1.health.HealthServicer()
    grpc_health.v1.health_pb2_grpc.add_HealthServicer_to_server(health_servicer, grpc_server)

    health_servicer.set(
        WORKOUT_GENERATOR_SERVICE_NAME, grpc_health.v1.health_pb2.HealthCheckResponse.SERVING
    )
    health_servicer.set(
        "", grpc_health.v1.health_pb2.HealthCheckResponse.SERVING
    )

    workout_generator_pb2_grpc.add_WorkoutGeneratorServiceServicer_to_server(
        WorkoutGeneratorGrpcService(get_generation_service()),
        grpc_server,
    )

    bound_port = grpc_server.add_insecure_port(settings.grpc_bind_addr)
    if bound_port == 0:
        raise RuntimeError(f"Failed to bind gRPC server on {settings.grpc_bind_addr}")

    await grpc_server.start()
    logger.info("grpc_server_started", address=settings.grpc_bind_addr)

    stop_event = asyncio.Event()

    def _handle_signal(*_):
        stop_event.set()

    loop = asyncio.get_running_loop()
    for sig in (signal.SIGINT, signal.SIGTERM):
        loop.add_signal_handler(sig, _handle_signal)

    await stop_event.wait()

    logger.info("grpc_server_stopping")
    health_servicer.set(
        "", grpc_health.v1.health_pb2.HealthCheckResponse.NOT_SERVING
    )
    health_servicer.set(
        WORKOUT_GENERATOR_SERVICE_NAME, grpc_health.v1.health_pb2.HealthCheckResponse.NOT_SERVING
    )
    await grpc_server.stop(grace=5)
    await get_provider().close()
    logger.info("grpc_server_stopped")


if __name__ == "__main__":
    asyncio.run(serve())
