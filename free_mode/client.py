"""FREE MODE client — routes model calls through LiteLLM proxy when enabled."""

import logging
import os
from dataclasses import dataclass
from typing import Optional

# Lazy imports — only needed if actually creating clients
try:
    from anthropic import Anthropic  # type: ignore[import-not-found]
except ImportError:
    Anthropic = None

try:
    from openai import OpenAI  # type: ignore[import-not-found]
except ImportError:
    OpenAI = None

logger = logging.getLogger(__name__)


@dataclass
class FreeModeConfig:
    """FREE MODE configuration."""

    enabled: bool
    proxy_base_url: str
    proxy_master_key: str
    provider: str
    model: str
    timeout_ms: int


class FreeMode:
    """FREE MODE manager — handles provider routing and client creation."""

    def __init__(self, config: Optional[FreeModeConfig] = None):
        if config is None:
            config = get_free_mode_config()
        self.config = config

    @property
    def is_enabled(self) -> bool:
        return self.config.enabled

    def create_openai_client(self):
        """Create OpenAI-compatible client routed through proxy if FREE MODE enabled."""
        if OpenAI is None:
            raise RuntimeError("OpenAI SDK not installed: pip install openai")

        if self.config.enabled:
            return OpenAI(
                base_url=self.config.proxy_base_url,
                api_key=self.config.proxy_master_key or "sk-free-mode",
                timeout=self.config.timeout_ms / 1000,
            )
        # Use standard OpenAI client
        return OpenAI(
            api_key=os.environ.get("OPENAI_API_KEY", ""),
            timeout=self.config.timeout_ms / 1000,
        )

    def create_anthropic_client(self):
        """Create Anthropic client — use proxy in FREE MODE if it supports Anthropic API."""
        if Anthropic is None:
            raise RuntimeError("Anthropic SDK not installed: pip install anthropic")

        if self.config.enabled:
            # Route through LiteLLM proxy (OpenAI-compatible)
            # Note: Hermes will need to translate Anthropic calls to OpenAI format
            return Anthropic(
                base_url=self.config.proxy_base_url,
                api_key=self.config.proxy_master_key or "sk-free-mode",
                timeout=self.config.timeout_ms / 1000,
            )
        # Use standard Anthropic client
        return Anthropic(
            api_key=os.environ.get("ANTHROPIC_API_KEY", ""),
            timeout=self.config.timeout_ms / 1000,
        )

    def get_model_name(self) -> str:
        """Get the active model name for this FREE MODE configuration."""
        if self.config.enabled:
            return self.config.model
        # Fallback to environment or default
        return os.environ.get("LLM_MODEL", "default-auto")

    def get_base_url(self) -> str:
        """Get the active base URL."""
        if self.config.enabled:
            return self.config.proxy_base_url
        return os.environ.get("ANTHROPIC_BASE_URL", "https://api.anthropic.com")

    def get_api_key(self) -> str:
        """Get the active API key."""
        if self.config.enabled:
            return self.config.proxy_master_key or "dummy"
        return os.environ.get("ANTHROPIC_API_KEY", "")


def is_free_mode_enabled() -> bool:
    """Check if FREE MODE is enabled."""
    return os.environ.get("FREE_MODE", "").lower() in ("true", "1", "yes")


def get_free_mode_config() -> FreeModeConfig:
    """Load FREE MODE configuration from environment."""
    return FreeModeConfig(
        enabled=is_free_mode_enabled(),
        proxy_base_url=os.environ.get("FREE_MODE_PROXY_BASE_URL", "http://127.0.0.1:4000"),
        proxy_master_key=os.environ.get(
            "FREE_MODE_PROXY_MASTER_KEY",
            os.environ.get("LITELLM_MASTER_KEY", "change-me-local-master-key"),
        ),
        provider=os.environ.get("FREE_MODE_PROVIDER", "auto"),
        model=os.environ.get("FREE_MODE_MODEL", "free-auto"),
        timeout_ms=int(os.environ.get("FREE_MODE_TIMEOUT_MS", "60000")),
    )


def create_free_mode_client() -> FreeMode:
    """Create a FREE MODE client with environment configuration."""
    config = get_free_mode_config()
    return FreeMode(config)
