from __future__ import annotations

from pydantic import Field
from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    model_config = SettingsConfigDict(env_file=".env", env_file_encoding="utf-8", extra="ignore")

    app_env: str = Field(default="dev", alias="WG_APP_ENV")
    app_host: str = Field(default="0.0.0.0", alias="WG_APP_HOST")
    app_port: int = Field(default=8091, alias="WG_APP_PORT")
    grpc_server_host: str = Field(default="0.0.0.0", alias="WG_GRPC_SERVER_HOST")
    grpc_server_port: int = Field(default=50052, alias="WG_GRPC_SERVER_PORT")

    openai_api_key: str = Field(alias="WG_OPENAI_API_KEY")
    openai_model: str = Field(default="gpt-5-mini", alias="WG_OPENAI_MODEL")
    openai_max_completion_tokens: int = Field(default=8192, alias="WG_OPENAI_MAX_COMPLETION_TOKENS")
    openai_timeout_seconds: float = Field(default=60.0, alias="WG_OPENAI_TIMEOUT_SECONDS")

    grpc_rust_host: str = Field(default="localhost", alias="WG_GRPC_RUST_HOST")
    grpc_rust_port: int = Field(default=50051, alias="WG_GRPC_RUST_PORT")
    grpc_timeout_seconds: float = Field(default=10.0, alias="WG_GRPC_TIMEOUT_SECONDS")


def load_settings() -> Settings:
    return Settings()
