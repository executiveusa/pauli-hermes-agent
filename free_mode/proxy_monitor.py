"""LiteLLM proxy monitoring and request interception.

Monitors:
- Requests to proxy
- Response times
- Token usage
- Errors
"""

import asyncio
import json
import logging
import time
from dataclasses import dataclass
from typing import Any, Callable, Optional

logger = logging.getLogger(__name__)


@dataclass
class ProxyRequest:
    """Proxy request metadata."""

    timestamp: str
    provider: str
    model: str
    input_tokens: int
    output_tokens: int
    latency_ms: float
    status: int
    error: Optional[str] = None


@dataclass
class ProxyMetrics:
    """Proxy metrics snapshot."""

    total_requests: int
    total_tokens: int
    average_latency_ms: float
    error_rate: float
    providers: dict[str, int]  # Provider -> request count


class ProxyMonitor:
    """Monitor and log LiteLLM proxy requests."""

    def __init__(self):
        self.requests: list[ProxyRequest] = []
        self.metrics_callbacks: list[Callable[[ProxyMetrics], None]] = []
        self.start_time = time.time()

    def record_request(
        self,
        provider: str,
        model: str,
        input_tokens: int,
        output_tokens: int,
        latency_ms: float,
        status: int = 200,
        error: Optional[str] = None,
    ) -> None:
        """Record a proxy request."""
        import datetime

        request = ProxyRequest(
            timestamp=datetime.datetime.now().isoformat(),
            provider=provider,
            model=model,
            input_tokens=input_tokens,
            output_tokens=output_tokens,
            latency_ms=latency_ms,
            status=status,
            error=error,
        )
        self.requests.append(request)

        # Trigger metrics callback
        if status == 200 and len(self.requests) % 10 == 0:
            self._emit_metrics()

    def _emit_metrics(self) -> None:
        """Calculate and emit current metrics."""
        if not self.requests:
            return

        total_requests = len(self.requests)
        total_tokens = sum(r.input_tokens + r.output_tokens for r in self.requests)
        avg_latency = sum(r.latency_ms for r in self.requests) / total_requests
        error_count = sum(1 for r in self.requests if r.status != 200)
        error_rate = error_count / total_requests if total_requests > 0 else 0

        # Count by provider
        providers = {}
        for req in self.requests:
            providers[req.provider] = providers.get(req.provider, 0) + 1

        metrics = ProxyMetrics(
            total_requests=total_requests,
            total_tokens=total_tokens,
            average_latency_ms=avg_latency,
            error_rate=error_rate,
            providers=providers,
        )

        for callback in self.metrics_callbacks:
            try:
                callback(metrics)
            except Exception as err:
                logger.error(f"Metrics callback error: {err}")

    def register_metrics_callback(self, callback: Callable[[ProxyMetrics], None]) -> None:
        """Register a callback for metrics updates."""
        self.metrics_callbacks.append(callback)

    def get_metrics(self) -> ProxyMetrics:
        """Get current metrics."""
        self._emit_metrics()
        if not self.requests:
            return ProxyMetrics(
                total_requests=0,
                total_tokens=0,
                average_latency_ms=0,
                error_rate=0,
                providers={},
            )

        total_requests = len(self.requests)
        total_tokens = sum(r.input_tokens + r.output_tokens for r in self.requests)
        avg_latency = sum(r.latency_ms for r in self.requests) / total_requests
        error_count = sum(1 for r in self.requests if r.status != 200)
        error_rate = error_count / total_requests if total_requests > 0 else 0

        providers = {}
        for req in self.requests:
            providers[req.provider] = providers.get(req.provider, 0) + 1

        return ProxyMetrics(
            total_requests=total_requests,
            total_tokens=total_tokens,
            average_latency_ms=avg_latency,
            error_rate=error_rate,
            providers=providers,
        )

    def get_recent_requests(self, limit: int = 100) -> list[dict[str, Any]]:
        """Get recent requests."""
        requests = self.requests[-limit:]
        return [
            {
                "timestamp": r.timestamp,
                "provider": r.provider,
                "model": r.model,
                "input_tokens": r.input_tokens,
                "output_tokens": r.output_tokens,
                "latency_ms": r.latency_ms,
                "status": r.status,
                "error": r.error,
            }
            for r in requests
        ]

    def clear(self) -> None:
        """Clear all recorded requests."""
        self.requests = []


# Global monitor instance
_monitor: Optional[ProxyMonitor] = None


def get_proxy_monitor() -> ProxyMonitor:
    """Get or create global proxy monitor."""
    global _monitor
    if _monitor is None:
        _monitor = ProxyMonitor()
    return _monitor


def record_request(
    provider: str,
    model: str,
    input_tokens: int,
    output_tokens: int,
    latency_ms: float,
    status: int = 200,
    error: Optional[str] = None,
) -> None:
    """Record a request globally."""
    get_proxy_monitor().record_request(
        provider, model, input_tokens, output_tokens, latency_ms, status, error
    )
