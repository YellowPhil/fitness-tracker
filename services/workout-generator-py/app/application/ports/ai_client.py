from __future__ import annotations

from dataclasses import dataclass
from typing import Protocol


@dataclass(frozen=True)
class ToolCall:
    id: str
    name: str
    arguments: str


@dataclass(frozen=True)
class ChatMessage:
    role: str
    content: str | None = None
    tool_calls: list[ToolCall] | None = None
    tool_call_id: str | None = None


@dataclass(frozen=True)
class CompletionRequest:
    model: str
    max_completion_tokens: int
    messages: list[ChatMessage]
    tools: list[dict] | None = None
    require_tool_choice: bool = False
    response_schema: dict | None = None


@dataclass(frozen=True)
class CompletionResponse:
    content: str | None
    tool_calls: list[ToolCall]
    refusal: str | None
    usage: "CompletionUsage | None" = None


@dataclass(frozen=True)
class CompletionUsage:
    prompt_tokens: int
    completion_tokens: int
    total_tokens: int


class AiChatClient(Protocol):
    async def complete(self, request: CompletionRequest) -> CompletionResponse:
        """Submits a chat completion request and returns normalized output."""
        ...
