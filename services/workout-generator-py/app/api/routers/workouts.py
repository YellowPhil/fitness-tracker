from __future__ import annotations

from fastapi import APIRouter, Depends

from app.api.dependencies import get_generation_service
from app.api.schemas.workout_generation import GenerateWorkoutRequest, GenerateWorkoutResponse
from app.application.services.workout_generation_service import WorkoutGenerationService
from app.domain.models import GenerateWorkoutCommand

router = APIRouter(prefix="/v1/workouts", tags=["workouts"])


@router.post("/generate", response_model=GenerateWorkoutResponse)
async def generate_workout(
    request: GenerateWorkoutRequest,
    service: WorkoutGenerationService = Depends(get_generation_service),
) -> GenerateWorkoutResponse:
    result = await service.generate(GenerateWorkoutCommand.model_validate(request.model_dump()))
    return GenerateWorkoutResponse.model_validate(result.model_dump())
