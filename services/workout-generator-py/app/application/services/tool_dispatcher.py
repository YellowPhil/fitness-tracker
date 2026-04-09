from __future__ import annotations

import json
import time
from dataclasses import dataclass
from typing import Awaitable, Callable

import structlog

from app.application.ports.ai_client import ToolCall

logger = structlog.get_logger(__name__)


ToolHandler = Callable[[int, str], Awaitable[str]]


@dataclass(frozen=True)
class ToolResponse:
    tool_call: ToolCall
    content: str


class ToolRegistry:
    def __init__(self, handlers: dict[str, ToolHandler]):
        self._handlers = handlers

    def has_tool(self, name: str) -> bool:
        return name in self._handlers

    def get_handler(self, name: str) -> ToolHandler:
        return self._handlers[name]


class ToolDispatcher:
    def __init__(self, registry: ToolRegistry):
        self._registry = registry

    async def dispatch(self, user_id: int, tool_calls: list[ToolCall]) -> list[ToolResponse]:
        responses: list[ToolResponse] = []
        for tool_call in tool_calls:
            if self._registry.has_tool(tool_call.name):
                handler = self._registry.get_handler(tool_call.name)
                logger.debug(
                    "tool_executing",
                    tool_name=tool_call.name,
                    tool_call_id=tool_call.id,
                    arguments_preview=self._truncate_arguments(tool_call.arguments),
                )
                started = time.monotonic()
                try:
                    content = await handler(user_id, tool_call.arguments)
                    logger.debug(
                        "tool_completed",
                        tool_name=tool_call.name,
                        tool_call_id=tool_call.id,
                        response_length=len(content),
                        duration_ms=int((time.monotonic() - started) * 1000),
                    )
                except Exception as exc:
                    logger.error(
                        "tool_failed",
                        tool_name=tool_call.name,
                        tool_call_id=tool_call.id,
                        error=str(exc),
                    )
                    content = str(exc)
            else:
                logger.debug("tool_unknown", tool_name=tool_call.name, tool_call_id=tool_call.id)
                content = json.dumps(
                    {"error": "unknown_tool", "name": tool_call.name}, separators=(",", ":")
                )
            responses.append(ToolResponse(tool_call=tool_call, content=content))
        return responses

    def _truncate_arguments(self, arguments: str, max_chars: int = 500) -> str:
        if len(arguments) <= max_chars:
            return arguments
        return arguments[:max_chars]
