"""Health checks and provider status monitoring."""

import asyncio
import json
import logging
import os
import time
from dataclasses import dataclass, asdict
from typing import Optional
import httpx

logger = logging.getLogger(__name__)


@dataclass
class ProviderHealth:
    """Health status of a single provider."""

    provider_id: str
    label: str
    configured: bool
    connected: bool
    latency_ms: Optional[float] = None
    last_error: Optional[str] = None
    model: Optional[str] = None
    status: str = "unknown"  # unknown | healthy | missing_secret | auth_failed | timeout | failed

    def to_dict(self) -> dict:
        return asdict(self)


@dataclass
class FreeModeHealth:
    """Aggregate health of all FREE MODE providers."""

    free_mode_enabled: bool
    proxy_reachable: bool
    proxy_latency_ms: Optional[float]
    providers: list[ProviderHealth]
    timestamp: float

    @property
    def summary(self) -> dict:
        """Summary statistics."""
        return {
            "configured": sum(1 for p in self.providers if p.configured),
            "connected": sum(1 for p in self.providers if p.connected),
            "missing_secret": sum(1 for p in self.providers if p.status == "missing_secret"),
            "auth_failed": sum(1 for p in self.providers if p.status == "auth_failed"),
            "timeout": sum(1 for p in self.providers if p.status == "timeout"),
            "failed": sum(1 for p in self.providers if p.status == "failed"),
        }

    def to_dict(self) -> dict:
        return {
            "free_mode_enabled": self.free_mode_enabled,
            "proxy_reachable": self.proxy_reachable,
            "proxy_latency_ms": self.proxy_latency_ms,
            "providers": [p.to_dict() for p in self.providers],
            "summary": self.summary,
            "timestamp": self.timestamp,
        }


async def check_proxy_health(
    base_url: str = "http://127.0.0.1:4000",
    timeout_ms: int = 15000,
) -> tuple[bool, Optional[float]]:
    """Check if LiteLLM proxy is reachable.

    Returns (reachable, latency_ms).
    """
    timeout = httpx.Timeout(timeout_ms / 1000)

    try:
        start = time.time()
        async with httpx.AsyncClient(timeout=timeout) as client:
            response = await client.get(f"{base_url}/health")
            latency = (time.time() - start) * 1000
            return response.status_code == 200, latency
    except Exception as e:
        logger.debug(f"Proxy health check failed: {e}")
        return False, None


async def check_provider_health(
    provider_id: str,
    base_url: str,
    api_key: Optional[str],
    model: str,
    timeout_ms: int = 15000,
) -> ProviderHealth:
    """Check health of a single provider.

    Returns ProviderHealth with status.
    """
    if not api_key and provider_id not in ("ollama", "lmstudio", "llamacpp", "vllm"):
        return ProviderHealth(
            provider_id=provider_id,
            label=provider_id,
            configured=False,
            connected=False,
            status="missing_secret",
            last_error="API key not configured",
        )

    timeout = httpx.Timeout(timeout_ms / 1000)
    headers = {}
    if api_key:
        headers["Authorization"] = f"Bearer {api_key}"

    try:
        start = time.time()
        async with httpx.AsyncClient(timeout=timeout) as client:
            # Try models endpoint first
            response = await client.get(
                f"{base_url}/v1/models" if "/v1" in base_url else f"{base_url}/models",
                headers=headers,
            )
            latency = (time.time() - start) * 1000

            if response.status_code == 200:
                return ProviderHealth(
                    provider_id=provider_id,
                    label=provider_id,
                    configured=True,
                    connected=True,
                    latency_ms=latency,
                    model=model,
                    status="healthy",
                )
            elif response.status_code == 401:
                return ProviderHealth(
                    provider_id=provider_id,
                    label=provider_id,
                    configured=True,
                    connected=False,
                    status="auth_failed",
                    last_error="Invalid API key",
                )
            else:
                return ProviderHealth(
                    provider_id=provider_id,
                    label=provider_id,
                    configured=True,
                    connected=False,
                    status="failed",
                    last_error=f"HTTP {response.status_code}",
                )
    except asyncio.TimeoutError:
        return ProviderHealth(
            provider_id=provider_id,
            label=provider_id,
            configured=True,
            connected=False,
            status="timeout",
            last_error="Health check timeout",
        )
    except Exception as e:
        return ProviderHealth(
            provider_id=provider_id,
            label=provider_id,
            configured=True,
            connected=False,
            status="failed",
            last_error=str(e),
        )
