from __future__ import annotations

from functools import lru_cache

from app.application.services.workout_generation_service import WorkoutGenerationService
from app.application.services.tool_dispatcher import ToolDispatcher
from app.infrastructure.ai.openai_chat_client import OpenAiChatClient
from app.infrastructure.config import Settings, load_settings
from app.infrastructure.providers.grpc_tool_data_provider import GrpcToolDataProvider
from app.infrastructure.tooling.registry import build_tool_registry


@lru_cache
def get_settings() -> Settings:
    return load_settings()


@lru_cache
def get_provider() -> GrpcToolDataProvider:
    return GrpcToolDataProvider(get_settings())


@lru_cache
def get_generation_service() -> WorkoutGenerationService:
    settings = get_settings()
    ai_client = OpenAiChatClient(
        api_key=settings.openai_api_key,
        timeout_seconds=settings.openai_timeout_seconds,
    )
    dispatcher = ToolDispatcher(build_tool_registry(get_provider()))
    return WorkoutGenerationService(
        ai_client=ai_client,
        tool_dispatcher=dispatcher,
        provider=get_provider(),
        model=settings.openai_model,
        max_completion_tokens=settings.openai_max_completion_tokens,
    )
