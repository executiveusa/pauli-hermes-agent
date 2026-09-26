"""Provider registry and metadata loading."""

import json
from pathlib import Path
from typing import Optional


class Provider:
    """Provider metadata."""

    def __init__(self, data: dict):
        self.id = data["id"]
        self.label = data["label"]
        self.mode = data["mode"]  # local | cloud
        self.free_mode = data.get("free_mode", False)
        self.requires_secret = data.get("requires_secret", False)
        self.base_url_env = data.get("base_url_env", "")
        self.api_key_env = data.get("api_key_env", "")
        self.model_env = data.get("model_env", "")
        self.protocol = data.get("protocol", "openai-compatible")
        self.health_endpoint = data.get("health_endpoint", "/health")
        self.test_strategy = data.get("test_strategy", "openai_models_then_chat")

    def __repr__(self) -> str:
        return f"<Provider {self.id}>"


class ProviderRegistry:
    """In-memory registry of all supported providers."""

    def __init__(self, providers: list[Provider]):
        self.providers = {p.id: p for p in providers}

    def get(self, provider_id: str) -> Optional[Provider]:
        """Get a provider by ID."""
        return self.providers.get(provider_id)

    def list_all(self) -> list[Provider]:
        """List all providers."""
        return list(self.providers.values())

    def list_free_mode(self) -> list[Provider]:
        """List providers suitable for free mode."""
        return [p for p in self.providers.values() if p.free_mode]

    def list_by_mode(self, mode: str) -> list[Provider]:
        """List providers by mode (local or cloud)."""
        return [p for p in self.providers.values() if p.mode == mode]

    def __len__(self) -> int:
        return len(self.providers)


def load_providers() -> ProviderRegistry:
    """Load provider registry from providers.json."""
    registry_path = Path(__file__).parent.parent / "free-mode" / "providers.json"

    if not registry_path.exists():
        raise FileNotFoundError(f"Provider registry not found: {registry_path}")

    with open(registry_path) as f:
        data = json.load(f)

    providers = [Provider(p) for p in data["providers"]]
    return ProviderRegistry(providers)
