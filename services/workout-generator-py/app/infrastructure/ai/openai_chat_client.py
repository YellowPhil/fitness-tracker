from __future__ import annotations

import structlog
from openai import APIConnectionError, APITimeoutError, AsyncOpenAI
from openai import OpenAIError

from app.application.errors import AiUnavailableError
from app.application.ports.ai_client import (
    AiChatClient,
    CompletionRequest,
    CompletionResponse,
    CompletionUsage,
)
from app.infrastructure.ai.serializers import deserialize_tool_calls, serialize_messages

logger = structlog.get_logger(__name__)


class OpenAiChatClient(AiChatClient):
    def __init__(self, api_key: str, timeout_seconds: float):
        self._client = AsyncOpenAI(api_key=api_key, timeout=timeout_seconds)

    async def complete(self, request: CompletionRequest) -> CompletionResponse:
        payload: dict = {
            "model": request.model,
            "max_completion_tokens": request.max_completion_tokens,
            "messages": serialize_messages(request.messages),
        }

        if request.tools is not None:
            payload["tools"] = request.tools
        if request.require_tool_choice:
            payload["tool_choice"] = "required"
        if request.response_schema is not None:
            payload["response_format"] = {
                "type": "json_schema",
                "json_schema": {
                    "name": "workout_plan",
                    "description": "Structured workout plan",
                    "strict": True,
                    "schema": request.response_schema,
                },
            }

        logger.debug(
            "openai_request",
            model=request.model,
            max_tokens=request.max_completion_tokens,
            tool_choice="required" if request.require_tool_choice else "auto",
            has_response_schema=request.response_schema is not None,
            message_count=len(request.messages),
        )

        try:
            response = await self._client.chat.completions.create(**payload)
        except (APITimeoutError, APIConnectionError) as exc:
            logger.error(
                "openai_error",
                error_type=type(exc).__name__,
                message=str(exc),
            )
            raise AiUnavailableError(f"OpenAI unavailable: {exc}") from exc
        except OpenAIError as exc:
            logger.error(
                "openai_error",
                error_type=type(exc).__name__,
                message=str(exc),
            )
            raise AiUnavailableError(f"OpenAI request failed: {exc}") from exc

        if not response.choices:
            return CompletionResponse(content=None, tool_calls=[], refusal=None)

        message = response.choices[0].message
        usage = getattr(response, "usage", None)
        completion_usage = None
        if usage is not None:
            completion_usage = CompletionUsage(
                prompt_tokens=getattr(usage, "prompt_tokens", 0) or 0,
                completion_tokens=getattr(usage, "completion_tokens", 0) or 0,
                total_tokens=getattr(usage, "total_tokens", 0) or 0,
            )

        tool_calls = deserialize_tool_calls(getattr(message, "tool_calls", []) or [])
        content = getattr(message, "content", None)
        refusal = getattr(message, "refusal", None)

        logger.debug(
            "openai_response",
            has_content=content is not None,
            tool_call_count=len(tool_calls),
            has_refusal=refusal is not None,
            prompt_tokens=completion_usage.prompt_tokens if completion_usage else 0,
            completion_tokens=completion_usage.completion_tokens if completion_usage else 0,
            total_tokens=completion_usage.total_tokens if completion_usage else 0,
            finish_reason=getattr(response.choices[0], "finish_reason", None),
        )

        return CompletionResponse(
            content=content,
            tool_calls=tool_calls,
            refusal=refusal,
            usage=completion_usage,
        )
