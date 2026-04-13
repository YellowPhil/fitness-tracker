from __future__ import annotations

import json

import grpc

from app.application.errors import ProviderResponseError, ProviderUnavailableError
from app.application.ports.tool_data_provider import ToolDataProvider
from app.domain.models import ExerciseCatalogItem, HealthProfileAttribute
from app.generated.fitness_tracker import (
    common_pb2,
    health_data_pb2,
    health_data_pb2_grpc,
    workout_data_pb2,
    workout_data_pb2_grpc,
)
from app.infrastructure.config import Settings


class GrpcToolDataProvider(ToolDataProvider):
    def __init__(self, settings: Settings):
        self._timeout_seconds = settings.grpc_timeout_seconds
        target = settings.grpc_rust_addr
        self._channel = grpc.aio.insecure_channel(target)
        self._workout_data_client = workout_data_pb2_grpc.WorkoutDataServiceStub(self._channel)
        self._health_data_client = health_data_pb2_grpc.HealthDataServiceStub(self._channel)

    async def close(self) -> None:
        await self._channel.close()

    async def load_health_profile(self, user_id: int) -> list[HealthProfileAttribute]:
        request = health_data_pb2.GetCurrentHealthProfileRequest(user_id=user_id)
        try:
            response = await self._health_data_client.GetCurrentHealthProfile(
                request, timeout=self._timeout_seconds
            )
        except grpc.RpcError as exc:
            raise map_grpc_error(exc) from exc

        return [
            HealthProfileAttribute(
                key=item.key,
                value=item.value,
                unit=item.unit if item.HasField("unit") else None,
            )
            for item in response.attributes
        ]

    async def load_exercises_for_muscle_groups(
        self,
        user_id: int,
        muscle_groups: list[str],
    ) -> list[ExerciseCatalogItem]:
        request = workout_data_pb2.GetExerciseCatalogRequest(
            user_id=user_id,
            muscle_groups=[muscle_group_to_proto(value) for value in muscle_groups],
        )
        try:
            response = await self._workout_data_client.GetExerciseCatalog(
                request, timeout=self._timeout_seconds
            )
        except grpc.RpcError as exc:
            raise map_grpc_error(exc) from exc

        return [
            ExerciseCatalogItem(
                exercise_id=item.exercise_id,
                name=item.name,
                kind=exercise_kind_from_proto(item.kind),
                muscle_group=muscle_group_from_proto(item.muscle_group),
            )
            for item in response.exercises
        ]

    async def query_workouts(self, user_id: int, arguments_json: str) -> str:
        arguments = parse_json_arguments(arguments_json)
        request = workout_data_pb2.QueryWorkoutsRequest(
            user_id=user_id,
            muscle_group=muscle_group_to_proto(read_required_muscle_group(arguments)),
        )
        if "date" in arguments and arguments["date"] is not None:
            request.date = str(arguments["date"])
        if "last_n" in arguments and arguments["last_n"] is not None:
            request.last_n = int(arguments["last_n"])

        try:
            response = await self._workout_data_client.QueryWorkouts(
                request, timeout=self._timeout_seconds
            )
        except grpc.RpcError as exc:
            raise map_grpc_error(exc) from exc
        return response.content

    async def list_exercises(self, user_id: int, arguments_json: str) -> str:
        arguments = parse_json_arguments(arguments_json)
        request = workout_data_pb2.ListExercisesRequest(
            user_id=user_id,
            muscle_group=muscle_group_to_proto(read_required_muscle_group(arguments)),
        )
        try:
            response = await self._workout_data_client.ListExercises(
                request, timeout=self._timeout_seconds
            )
        except grpc.RpcError as exc:
            raise map_grpc_error(exc) from exc
        return response.content


def parse_json_arguments(arguments_json: str) -> dict:
    try:
        parsed = json.loads(arguments_json)
    except json.JSONDecodeError as exc:
        raise ProviderResponseError("Tool arguments are not valid JSON") from exc
    if not isinstance(parsed, dict):
        raise ProviderResponseError("Tool arguments must be a JSON object")
    return parsed


def read_required_muscle_group(arguments: dict) -> str:
    value = arguments.get("muscle_group")
    if value is None:
        raise ProviderResponseError("muscle_group is required")
    return str(value)


def map_grpc_error(error: grpc.RpcError) -> Exception:
    code = error.code()
    message = error.details() or "gRPC provider request failed"
    if code in {
        grpc.StatusCode.UNAVAILABLE,
        grpc.StatusCode.DEADLINE_EXCEEDED,
        grpc.StatusCode.CANCELLED,
    }:
        return ProviderUnavailableError(f"Internal provider unavailable: {message}")
    if code == grpc.StatusCode.INVALID_ARGUMENT:
        return ProviderResponseError(f"Internal provider rejected request: {message}")
    return ProviderResponseError(f"Internal provider failed: {message}")


def muscle_group_to_proto(value: str) -> int:
    mapping = {
        "Chest": common_pb2.CHEST,
        "Back": common_pb2.BACK,
        "Shoulders": common_pb2.SHOULDERS,
        "Arms": common_pb2.ARMS,
        "Legs": common_pb2.LEGS,
        "Core": common_pb2.CORE,
    }
    try:
        return mapping[value]
    except KeyError as exc:
        raise ProviderResponseError(f"Unknown muscle group: {value}") from exc


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
        raise ProviderResponseError(f"Unknown muscle group value: {value}")
    return mapping[value]


def exercise_kind_from_proto(value: int) -> str:
    if value == common_pb2.WEIGHTED:
        return "Weighted"
    if value == common_pb2.BODY_WEIGHT:
        return "BodyWeight"
    raise ProviderResponseError(f"Unknown exercise kind value: {value}")
