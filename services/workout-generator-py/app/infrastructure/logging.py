from __future__ import annotations

import logging
import uuid

import structlog
from fastapi import Request


def configure_logging() -> None:
    timestamper = structlog.processors.TimeStamper(fmt="iso")
    structlog.configure(
        processors=[
            structlog.contextvars.merge_contextvars,
            structlog.stdlib.add_log_level,
            timestamper,
            structlog.processors.JSONRenderer(),
        ],
        logger_factory=structlog.stdlib.LoggerFactory(),
        cache_logger_on_first_use=True,
    )
    logging.basicConfig(level=logging.INFO)


def request_id_from_headers(request: Request) -> str:
    inbound = request.headers.get("x-request-id")
    return inbound or str(uuid.uuid4())
