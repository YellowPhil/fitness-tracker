from __future__ import annotations

from app.application.ports.ai_client import ChatMessage, ToolCall


def serialize_messages(messages: list[ChatMessage]) -> list[dict]:
    serialized: list[dict] = []
    for message in messages:
        payload: dict = {"role": message.role}
        if message.content is not None:
            payload["content"] = message.content
        if message.tool_calls:
            payload["tool_calls"] = [
                {
                    "id": call.id,
                    "type": "function",
                    "function": {"name": call.name, "arguments": call.arguments},
                }
                for call in message.tool_calls
            ]
        if message.tool_call_id is not None:
            payload["tool_call_id"] = message.tool_call_id
        serialized.append(payload)
    return serialized


def deserialize_tool_calls(raw_tool_calls: list) -> list[ToolCall]:
    tool_calls: list[ToolCall] = []
    for tool_call in raw_tool_calls:
        function = getattr(tool_call, "function", None)
        if function is None:
            continue
        tool_calls.append(
            ToolCall(
                id=getattr(tool_call, "id"),
                name=getattr(function, "name"),
                arguments=getattr(function, "arguments"),
            )
        )
    return tool_calls
