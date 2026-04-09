import pytest

from app.application.ports.ai_client import ToolCall
from app.application.services.tool_dispatcher import ToolDispatcher, ToolRegistry


async def known_handler(user_id: int, arguments_json: str) -> str:
    return f"known:{user_id}:{arguments_json}"


@pytest.mark.asyncio
async def test_dispatcher_returns_unknown_tool_payload() -> None:
    dispatcher = ToolDispatcher(ToolRegistry({"workout_query": known_handler}))
    calls = [
        ToolCall(id="c1", name="workout_query", arguments="{}"),
        ToolCall(id="c2", name="missing_tool", arguments="{}"),
    ]

    result = await dispatcher.dispatch(user_id=99, tool_calls=calls)

    assert result[0].content == "known:99:{}"
    assert result[1].content == '{"error":"unknown_tool","name":"missing_tool"}'
