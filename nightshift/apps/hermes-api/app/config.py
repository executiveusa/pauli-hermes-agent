"""Agent MAXX — hermes-api: Settings from environment."""

import os
from dataclasses import dataclass, field


@dataclass
class Settings:
    app_name: str = "Agent MAXX"
    app_env: str = "production"
    api_key: str = ""
    secret_key: str = ""
    database_url: str = ""
    redis_url: str = "redis://redis:6379/0"
    provider_router_url: str = "http://provider-router:8080"
    github_token: str = ""
    github_org: str = ""
    anthropic_api_key: str = ""
    openai_api_key: str = ""
    google_api_key: str = ""
    openrouter_api_key: str = ""
    telegram_bot_token: str = ""
    telegram_chat_id: str = ""
    appwrite_endpoint: str = ""
    appwrite_project_id: str = ""
    appwrite_api_key: str = ""

    @classmethod
    def from_env(cls) -> "Settings":
        return cls(
            app_name=os.getenv("APP_NAME", "Agent MAXX"),
            app_env=os.getenv("APP_ENV", "production"),
            api_key=os.getenv("API_KEY", ""),
            secret_key=os.getenv("SECRET_KEY", ""),
            database_url=os.getenv("DATABASE_URL", "postgresql+asyncpg://maxx:maxx_secret@postgres:5432/maxx_db"),
            redis_url=os.getenv("REDIS_URL", "redis://redis:6379/0"),
            provider_router_url=os.getenv("PROVIDER_ROUTER_URL", "http://provider-router:8080"),
            github_token=os.getenv("GITHUB_TOKEN", ""),
            github_org=os.getenv("GITHUB_ORG", ""),
            anthropic_api_key=os.getenv("ANTHROPIC_API_KEY", ""),
            openai_api_key=os.getenv("OPENAI_API_KEY", ""),
            google_api_key=os.getenv("GOOGLE_API_KEY", ""),
            openrouter_api_key=os.getenv("OPENROUTER_API_KEY", ""),
            telegram_bot_token=os.getenv("TELEGRAM_BOT_TOKEN", ""),
            telegram_chat_id=os.getenv("TELEGRAM_CHAT_ID", ""),
            appwrite_endpoint=os.getenv("APPWRITE_ENDPOINT", ""),
            appwrite_project_id=os.getenv("APPWRITE_PROJECT_ID", ""),
            appwrite_api_key=os.getenv("APPWRITE_API_KEY", ""),
        )

    def missing_secrets(self) -> list[dict]:
        checks = [
            ("ANTHROPIC_API_KEY", self.anthropic_api_key, "Anthropic Claude models", "No Claude access", False),
            ("DATABASE_URL", self.database_url, "PostgreSQL connection", "No data persistence", False),
            ("OPENAI_API_KEY", self.openai_api_key, "OpenAI GPT models", "No GPT access", True),
            ("GOOGLE_API_KEY", self.google_api_key, "Google Gemini models", "No Gemini access", True),
            ("OPENROUTER_API_KEY", self.openrouter_api_key, "OpenRouter multi-model", "No OpenRouter access", True),
            ("GITHUB_TOKEN", self.github_token, "GitHub API access", "No repo scanning", True),
            ("TELEGRAM_BOT_TOKEN", self.telegram_bot_token, "Telegram notifications", "No alerts", True),
            ("APPWRITE_ENDPOINT", self.appwrite_endpoint, "Appwrite backend", "No Appwrite provisioning", True),
        ]
        return [
            {"key": k, "description": d, "why": w, "deferrable": df}
            for k, v, d, w, df in checks if not v
        ]


settings = Settings.from_env()
