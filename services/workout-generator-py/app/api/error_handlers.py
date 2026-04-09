from __future__ import annotations

import structlog
from fastapi import FastAPI, Request
from fastapi.responses import JSONResponse
from pydantic import ValidationError

from app.api.schemas.errors import ErrorEnvelope
from app.application.errors import ApplicationError

logger = structlog.get_logger(__name__)


def register_error_handlers(app: FastAPI) -> None:
    @app.exception_handler(ApplicationError)
    async def application_error_handler(request: Request, exc: ApplicationError) -> JSONResponse:
        request_id = getattr(request.state, "request_id", "")
        logger.warning("application_error", code=exc.code, request_id=request_id, message=exc.message)
        envelope = ErrorEnvelope(
            error={
                "code": exc.code,
                "message": exc.message,
                "retriable": exc.retriable,
                "request_id": request_id,
            }
        )
        return JSONResponse(status_code=exc.status_code, content=envelope.model_dump())

    @app.exception_handler(ValidationError)
    async def validation_error_handler(request: Request, exc: ValidationError) -> JSONResponse:
        request_id = getattr(request.state, "request_id", "")
        logger.warning("validation_error", request_id=request_id, errors=exc.errors())
        envelope = ErrorEnvelope(
            error={
                "code": "invalid_request",
                "message": "Request body validation failed",
                "retriable": False,
                "request_id": request_id,
            }
        )
        return JSONResponse(status_code=400, content=envelope.model_dump())

    @app.exception_handler(Exception)
    async def unhandled_error_handler(request: Request, exc: Exception) -> JSONResponse:
        request_id = getattr(request.state, "request_id", "")
        logger.exception("unhandled_error", request_id=request_id, message=str(exc))
        envelope = ErrorEnvelope(
            error={
                "code": "internal_error",
                "message": "Internal server error",
                "retriable": True,
                "request_id": request_id,
            }
        )
        return JSONResponse(status_code=500, content=envelope.model_dump())
