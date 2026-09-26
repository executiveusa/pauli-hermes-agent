"""FREE MODE — Universal free AI toggle for Hermes Agent.

When FREE_MODE=true, all model calls route through LiteLLM proxy → free/local providers.
When FREE_MODE=false, original provider config is used unchanged.
"""

__version__ = "0.1.0"

# Core imports (minimal dependencies)
from free_mode.env import load_free_mode_env
from free_mode.client import (
    FreeMode,
    FreeModeConfig,
    create_free_mode_client,
    is_free_mode_enabled,
    get_free_mode_config,
)
from free_mode.provider_registry import ProviderRegistry, load_providers

# Lazy imports (heavy dependencies)
def __getattr__(name):
    if name == "FreeModeHealth" or name == "check_provider_health":
        from free_mode.health import FreeModeHealth, check_provider_health
        globals()[name] = FreeModeHealth if name == "FreeModeHealth" else check_provider_health
        return globals()[name]
    raise AttributeError(f"module {__name__!r} has no attribute {name!r}")

__all__ = [
    "FreeMode",
    "FreeModeConfig",
    "FreeModeHealth",
    "ProviderRegistry",
    "create_free_mode_client",
    "is_free_mode_enabled",
    "get_free_mode_config",
    "check_provider_health",
    "load_free_mode_env",
    "load_providers",
]
