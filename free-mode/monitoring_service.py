#!/usr/bin/env python3
"""FREE MODE Monitoring Service.

Runs alongside LiteLLM proxy to:
- Monitor requests
- Track costs
- Log usage
- Provide metrics API
"""

import asyncio
import json
import logging
import os
import sys
import time
from datetime import datetime
from pathlib import Path

try:
    from free_mode.cost_tracker import get_cost_tracker, track_request
    from free_mode.proxy_monitor import get_proxy_monitor, record_request
except ImportError:
    # Add parent to path for imports if running as script
    sys.path.insert(0, str(Path(__file__).parent.parent))
    from free_mode.cost_tracker import get_cost_tracker, track_request
    from free_mode.proxy_monitor import get_proxy_monitor, record_request

try:
    from fastapi import FastAPI, HTTPException
    from fastapi.responses import JSONResponse
    from uvicorn import run
except ImportError:
    print("ERROR: Fastapi and uvicorn required. Install with:")
    print("  pip install fastapi uvicorn")
    sys.exit(1)

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

app = FastAPI(title="FREE MODE Monitoring Service")

# Get trackers
cost_tracker = get_cost_tracker()
monitor = get_proxy_monitor()


@app.on_event("startup")
async def startup():
    """Initialize on startup."""
    logger.info("FREE MODE Monitoring Service started")
    logger.info(f"Cost tracking: {cost_tracker.cost_file}")
    logger.info(f"History: {cost_tracker.history_file}")


@app.get("/health")
async def health():
    """Health check."""
    return {
        "status": "healthy",
        "timestamp": datetime.now().isoformat(),
        "free_mode": os.environ.get("FREE_MODE", "false"),
    }


@app.post("/track/request")
async def track_request_endpoint(
    provider: str,
    model: str,
    input_tokens: int,
    output_tokens: int,
    latency_ms: float,
    status: int = 200,
    error: str = None,
):
    """Track a request."""
    try:
        # Record in monitor
        record_request(provider, model, input_tokens, output_tokens, latency_ms, status, error)

        # Track cost
        track_request(provider, input_tokens, output_tokens)

        return {
            "success": True,
            "timestamp": datetime.now().isoformat(),
            "provider": provider,
            "cost_today": cost_tracker.get_costs_today(),
        }
    except Exception as err:
        logger.error(f"Failed to track request: {err}")
        raise HTTPException(status_code=500, detail=str(err))


@app.get("/metrics")
async def metrics():
    """Get current metrics."""
    try:
        metrics = monitor.get_metrics()
        return {
            "total_requests": metrics.total_requests,
            "total_tokens": metrics.total_tokens,
            "average_latency_ms": metrics.average_latency_ms,
            "error_rate": metrics.error_rate,
            "providers": metrics.providers,
            "costs": {
                "today": cost_tracker.get_costs_today(),
                "month": cost_tracker.get_costs_month(),
            },
        }
    except Exception as err:
        logger.error(f"Failed to get metrics: {err}")
        raise HTTPException(status_code=500, detail=str(err))


@app.get("/costs")
async def costs():
    """Get cost data."""
    try:
        return cost_tracker.get_all_stats()
    except Exception as err:
        logger.error(f"Failed to get costs: {err}")
        raise HTTPException(status_code=500, detail=str(err))


@app.get("/costs/history")
async def costs_history(days: int = 1):
    """Get cost history."""
    try:
        return {"entries": cost_tracker.get_history(days=days)}
    except Exception as err:
        logger.error(f"Failed to get history: {err}")
        raise HTTPException(status_code=500, detail=str(err))


@app.get("/requests")
async def recent_requests(limit: int = 100):
    """Get recent requests."""
    try:
        return {"requests": monitor.get_recent_requests(limit=limit)}
    except Exception as err:
        logger.error(f"Failed to get requests: {err}")
        raise HTTPException(status_code=500, detail=str(err))


@app.get("/stats")
async def stats():
    """Get complete stats."""
    try:
        return {
            "costs": cost_tracker.get_all_stats(),
            "metrics": {
                "total_requests": monitor.get_metrics().total_requests,
                "total_tokens": monitor.get_metrics().total_tokens,
                "average_latency_ms": monitor.get_metrics().average_latency_ms,
                "error_rate": monitor.get_metrics().error_rate,
            },
            "timestamp": datetime.now().isoformat(),
        }
    except Exception as err:
        logger.error(f"Failed to get stats: {err}")
        raise HTTPException(status_code=500, detail=str(err))


def main():
    """Run monitoring service."""
    port = int(os.environ.get("FREE_MODE_MONITOR_PORT", "8001"))
    host = os.environ.get("FREE_MODE_MONITOR_HOST", "127.0.0.1")

    logger.info(f"Starting monitoring service on {host}:{port}")
    run(app, host=host, port=port, log_level="info")


if __name__ == "__main__":
    main()
