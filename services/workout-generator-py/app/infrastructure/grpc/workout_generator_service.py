from __future__ import annotations

from datetime import date
import time

import grpc
import structlog

from app.application.errors import ApplicationError
from app.application.services.workout_generation_service import WorkoutGenerationService
from app.domain.models import GenerateWorkoutCommand
from app.generated.fitness_tracker import common_pb2, workout_generator_pb2, workout_generator_pb2_grpc


logger = structlog.get_logger(__name__)


class WorkoutGeneratorGrpcService(workout_generator_pb2_grpc.WorkoutGeneratorServiceServicer):
    def __init__(self, service: WorkoutGenerationService):
        self._service = service

    async def GenerateWorkout(self, request, context):
        started_at = time.perf_counter()
        request_id = grpc_metadata_value(context, "x-request-id")
        deadline_seconds = context.time_remaining()
        logger.info(
            "grpc_generate_workout_started",
            request_id=request_id,
            user_id=request.user_id,
            peer=context.peer(),
            deadline_seconds=round(deadline_seconds, 3) if deadline_seconds is not None else None,
            max_exercise_count=request.max_exercise_count,
            muscle_group_count=len(request.muscle_groups),
        )

        try:
            command = GenerateWorkoutCommand(
                user_id=request.user_id,
                date=parse_iso_date(request.date),
                muscle_groups=[muscle_group_from_proto(value) for value in request.muscle_groups],
                max_exercise_count=request.max_exercise_count,
            )
            result = await self._service.generate(command)
        except ApplicationError as exc:
            context.set_code(map_status_code(exc.status_code))
            context.set_details(exc.message)
            logger.warning(
                "grpc_generate_workout_failed",
                request_id=request_id,
                grpc_code=map_status_code(exc.status_code).name,
                app_status_code=exc.status_code,
                error=exc.message,
                elapsed_ms=elapsed_ms(started_at),
            )
            return workout_generator_pb2.GenerateWorkoutResponse()
        except ValueError as exc:
            context.set_code(grpc.StatusCode.INVALID_ARGUMENT)
            context.set_details(str(exc))
            logger.warning(
                "grpc_generate_workout_failed",
                request_id=request_id,
                grpc_code=grpc.StatusCode.INVALID_ARGUMENT.name,
                error=str(exc),
                elapsed_ms=elapsed_ms(started_at),
            )
            return workout_generator_pb2.GenerateWorkoutResponse()
        except Exception as exc:
            context.set_code(grpc.StatusCode.INTERNAL)
            context.set_details("Unexpected workout generator failure")
            logger.exception(
                "grpc_generate_workout_unexpected_error",
                request_id=request_id,
                error_type=type(exc).__name__,
                elapsed_ms=elapsed_ms(started_at),
            )
            return workout_generator_pb2.GenerateWorkoutResponse()

        response = workout_generator_pb2.GenerateWorkoutResponse()
        if result.workout_name is not None:
            response.workout_name = result.workout_name

        for exercise in result.exercises:
            exercise_message = response.exercises.add()
            exercise_message.exercise_id = exercise.exercise_id
            exercise_message.exercise_name = exercise.exercise_name
            if exercise.notes is not None:
                exercise_message.notes = exercise.notes
            for workout_set in exercise.sets:
                set_message = exercise_message.sets.add()
                set_message.reps = workout_set.reps
                if workout_set.weight_kg is not None:
                    set_message.weight_kg = workout_set.weight_kg

        logger.info(
            "grpc_generate_workout_succeeded",
            request_id=request_id,
            user_id=request.user_id,
            exercise_count=len(response.exercises),
            elapsed_ms=elapsed_ms(started_at),
        )

        return response


def parse_iso_date(value: str) -> date:
    try:
        return date.fromisoformat(value)
    except ValueError as exc:
        raise ValueError(f"Invalid date format: {value}") from exc


def muscle_group_from_proto(value: int) -> str:
    mapping = {
        common_pb2.CHEST: "Chest",
        common_pb2.BACK: "Back",
        common_pb2.SHOULDERS: "Shoulders",
        common_pb2.ARMS: "Arms",
        common_pb2.LEGS: "Legs",
        common_pb2.CORE: "Core",
    }
    if value not in mapping:
        raise ValueError(f"Invalid muscle_group value: {value}")
    return mapping[value]


def map_status_code(status_code: int) -> grpc.StatusCode:
    mapping = {
        400: grpc.StatusCode.INVALID_ARGUMENT,
        422: grpc.StatusCode.FAILED_PRECONDITION,
        502: grpc.StatusCode.UNAVAILABLE,
        503: grpc.StatusCode.UNAVAILABLE,
    }
    return mapping.get(status_code, grpc.StatusCode.INTERNAL)


def grpc_metadata_value(context, key: str) -> str | None:
    normalized_key = key.lower()
    for metadata_key, metadata_value in context.invocation_metadata():
        if metadata_key.lower() == normalized_key:
            return metadata_value
    return None


def elapsed_ms(started_at: float) -> int:
    return int((time.perf_counter() - started_at) * 1000)
