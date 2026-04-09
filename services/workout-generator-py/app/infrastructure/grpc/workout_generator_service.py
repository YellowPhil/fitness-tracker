from __future__ import annotations

from datetime import date

import grpc

from app.application.errors import ApplicationError
from app.application.services.workout_generation_service import WorkoutGenerationService
from app.domain.models import GenerateWorkoutCommand
from app.generated.fitness_tracker import common_pb2, workout_generator_pb2, workout_generator_pb2_grpc


class WorkoutGeneratorGrpcService(workout_generator_pb2_grpc.WorkoutGeneratorServiceServicer):
    def __init__(self, service: WorkoutGenerationService):
        self._service = service

    async def GenerateWorkout(self, request, context):
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
            return workout_generator_pb2.GenerateWorkoutResponse()
        except ValueError as exc:
            context.set_code(grpc.StatusCode.INVALID_ARGUMENT)
            context.set_details(str(exc))
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
