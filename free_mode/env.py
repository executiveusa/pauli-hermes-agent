"""Environment variable loading and validation for FREE MODE."""

import os
from pathlib import Path
from typing import Optional


def load_free_mode_env() -> dict[str, str]:
    """Load and validate FREE MODE environment variables.

    Returns a dict of all FREE_MODE_* env vars currently set.
    Does not validate values, only retrieves them.
    """
    env_vars = {}
    for key, value in os.environ.items():
        if key.startswith("FREE_MODE") or key.startswith("LITELLM"):
            env_vars[key] = value
    return env_vars


def is_free_mode_enabled() -> bool:
    """Check if FREE_MODE is enabled.

    Returns True if FREE_MODE=true (case-insensitive).
    """
    return os.environ.get("FREE_MODE", "").lower() in ("true", "1", "yes")


def get_proxy_config() -> dict:
    """Get LiteLLM proxy configuration from environment.

    Returns dict with proxy settings.
    """
    return {
        "base_url": os.environ.get("FREE_MODE_PROXY_BASE_URL", "http://127.0.0.1:4000"),
        "master_key": os.environ.get("LITELLM_MASTER_KEY", os.environ.get("FREE_MODE_PROXY_MASTER_KEY")),
        "provider": os.environ.get("FREE_MODE_PROVIDER", "auto"),
        "model": os.environ.get("FREE_MODE_MODEL", "free-auto"),
        "timeout_ms": int(os.environ.get("FREE_MODE_TIMEOUT_MS", "60000")),
    }


def get_provider_env(provider_id: str) -> Optional[str]:
    """Get the API key env var name for a provider.

    Example: get_provider_env("groq") -> "GROQ_API_KEY"
    """
    mapping = {
        "ollama": "OLLAMA_API_KEY",
        "lmstudio": "LMSTUDIO_API_KEY",
        "openrouter": "OPENROUTER_API_KEY",
        "groq": "GROQ_API_KEY",
        "gemini": "GEMINI_API_KEY",
        "nvidia_nim": "NVIDIA_NIM_API_KEY",
        "cerebras": "CEREBRAS_API_KEY",
        "deepseek": "DEEPSEEK_API_KEY",
        "mistral": "MISTRAL_API_KEY",
        "huggingface": "HF_TOKEN",
        "together": "TOGETHER_API_KEY",
        "fireworks": "FIREWORKS_API_KEY",
        "perplexity": "PERPLEXITY_API_KEY",
        "openai": "OPENAI_API_KEY",
        "anthropic": "ANTHROPIC_API_KEY",
    }
    return mapping.get(provider_id)


def validate_provider_secret(provider_id: str) -> bool:
    """Check if a provider's required API key is set."""
    key_name = get_provider_env(provider_id)
    if key_name:
        return bool(os.environ.get(key_name))
    return False
