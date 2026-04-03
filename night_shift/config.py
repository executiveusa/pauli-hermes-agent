from __future__ import annotations

import os
from dataclasses import dataclass


@dataclass(frozen=True)
class NightShiftConfig:
    database_url: str = os.getenv("HERMES_NIGHTSHIFT_DB", "sqlite:///./night_shift.db")
    redis_url: str = os.getenv("HERMES_REDIS_URL", "redis://redis:6379/0")
    temporal_host: str = os.getenv("HERMES_TEMPORAL_HOST", "temporal:7233")
    paperclip_url: str = os.getenv("HERMES_PAPERCLIP_URL", "")
    composio_url: str = os.getenv("HERMES_COMPOSIO_URL", "")
    appwrite_endpoint: str = os.getenv("APPWRITE_ENDPOINT", "")
    appwrite_project_id: str = os.getenv("APPWRITE_PROJECT_ID", "")

    @property
    def missing_secrets(self) -> dict[str, bool]:
        required = {
            "GITHUB_TOKEN": os.getenv("GITHUB_TOKEN"),
            "OPENAI_API_KEY": os.getenv("OPENAI_API_KEY"),
            "GEMINI_API_KEY": os.getenv("GEMINI_API_KEY"),
            "VENICE_API_KEY": os.getenv("VENICE_API_KEY"),
            "TELEGRAM_BOT_TOKEN": os.getenv("TELEGRAM_BOT_TOKEN"),
            "APPWRITE_API_KEY": os.getenv("APPWRITE_API_KEY"),
            "COMPOSIO_API_KEY": os.getenv("COMPOSIO_API_KEY"),
            "PAPERCLIP_API_KEY": os.getenv("PAPERCLIP_API_KEY"),
        }
        return {k: not bool(v) for k, v in required.items()}
