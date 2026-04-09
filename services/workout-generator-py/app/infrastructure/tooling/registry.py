from __future__ import annotations

from typing import cast

from app.application.ports.tool_data_provider import ToolDataProvider
from app.application.services.tool_dispatcher import ToolHandler, ToolRegistry
from app.domain.constants import EXERCISE_LIST_TOOL, WORKOUT_QUERY_TOOL


def build_tool_registry(provider: ToolDataProvider) -> ToolRegistry:
    handlers = {
        WORKOUT_QUERY_TOOL: provider.query_workouts,
        EXERCISE_LIST_TOOL: provider.list_exercises,
    }
    return ToolRegistry(handlers=cast(dict[str, ToolHandler], handlers))
