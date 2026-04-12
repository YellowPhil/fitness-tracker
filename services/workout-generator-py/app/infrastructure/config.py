from __future__ import annotations

from pydantic import Field
from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    model_config = SettingsConfigDict(
        env_file=".env", env_file_encoding="utf-8", extra="ignore"
    )

    app_env: str = Field(default="dev", alias="WG_APP_ENV")
    app_bind_addr: str = Field(default="0.0.0.0:8091", alias="WG_BIND_ADDR")
    grpc_bind_addr: str = Field(default="0.0.0.0:50052", alias="WG_GRPC_BIND_ADDR")

    openai_api_key: str = Field(alias="WG_OPENAI_API_KEY")
    openai_model: str = Field(default="gpt-5-mini", alias="WG_OPENAI_MODEL")
    openai_max_completion_tokens: int = Field(
        default=8192, alias="WG_OPENAI_MAX_COMPLETION_TOKENS"
    )
    openai_timeout_seconds: float = Field(
        default=60.0, alias="WG_OPENAI_TIMEOUT_SECONDS"
    )

    grpc_rust_addr: str = Field(default="localhost:50051", alias="WG_GRPC_RUST_ADDR")
    grpc_timeout_seconds: float = Field(default=10.0, alias="WG_GRPC_TIMEOUT_SECONDS")


def load_settings() -> Settings:
    return Settings()
