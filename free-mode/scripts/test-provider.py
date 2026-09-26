#!/usr/bin/env python3
"""Test a single provider's connectivity and model availability."""

import asyncio
import json
import logging
import os
import sys
import time
from pathlib import Path

# Add parent to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from free_mode.health import check_provider_health
from free_mode.provider_registry import load_providers
from free_mode.env import is_free_mode_enabled, get_proxy_config

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(levelname)s - %(message)s"
)
logger = logging.getLogger(__name__)


async def test_provider(provider_id: str, verbose: bool = False) -> dict:
    """Test a single provider."""
    registry = load_providers()
    provider = registry.get(provider_id)

    if not provider:
        logger.error(f"Provider not found: {provider_id}")
        return {"status": "not_found"}

    logger.info(f"Testing provider: {provider.label}")

    # Get configuration
    base_url_env = provider.base_url_env
    api_key_env = provider.api_key_env
    model_env = provider.model_env

    base_url = os.environ.get(base_url_env, "")
    api_key = os.environ.get(api_key_env, "")
    model = os.environ.get(model_env, "")

    if not base_url:
        logger.warning(f"No base URL configured for {provider_id}")
        return {
            "provider": provider_id,
            "status": "not_configured",
            "reason": f"{base_url_env} not set",
        }

    if verbose:
        logger.debug(f"  Base URL: {base_url}")
        logger.debug(f"  Model: {model}")
        logger.debug(f"  Has API Key: {bool(api_key)}")

    health = await check_provider_health(
        provider_id=provider_id,
        base_url=base_url,
        api_key=api_key,
        model=model,
    )

    result = {
        "provider": provider_id,
        "label": provider.label,
        "status": health.status,
        "connected": health.connected,
        "latency_ms": health.latency_ms,
        "error": health.last_error,
        "model": health.model,
    }

    if verbose:
        logger.info(f"Result: {json.dumps(result, indent=2)}")

    return result


async def test_all_free_providers(verbose: bool = False) -> dict:
    """Test all free-mode providers."""
    registry = load_providers()
    free_providers = registry.list_free_mode()

    logger.info(f"Testing {len(free_providers)} free-mode providers...")
    results = {
        "free_mode_enabled": is_free_mode_enabled(),
        "timestamp": time.time(),
        "results": {},
    }

    for provider in free_providers:
        result = await test_provider(provider.id, verbose=verbose)
        results["results"][provider.id] = result

    # Summary
    connected = sum(1 for r in results["results"].values() if r.get("connected"))
    missing = sum(1 for r in results["results"].values() if r.get("status") == "missing_secret")
    failed = sum(1 for r in results["results"].values() if r.get("status") == "failed")

    results["summary"] = {
        "total": len(free_providers),
        "connected": connected,
        "missing_secret": missing,
        "failed": failed,
    }

    return results


def main():
    import argparse

    parser = argparse.ArgumentParser(description="Test FREE MODE providers")
    parser.add_argument("provider", nargs="?", help="Provider ID to test (omit for all)")
    parser.add_argument("--all", action="store_true", help="Test all free providers")
    parser.add_argument("--verbose", "-v", action="store_true", help="Verbose output")
    parser.add_argument("--json", action="store_true", help="JSON output")

    args = parser.parse_args()

    if args.all or not args.provider:
        results = asyncio.run(test_all_free_providers(verbose=args.verbose))

        if args.json:
            print(json.dumps(results, indent=2))
        else:
            print(f"\nFREE MODE Provider Test Summary")
            print(f"================================")
            print(f"Total: {results['summary']['total']}")
            print(f"Connected: {results['summary']['connected']}")
            print(f"Missing Secret: {results['summary']['missing_secret']}")
            print(f"Failed: {results['summary']['failed']}")
            print(f"\nDetailed Results:")
            for provider_id, result in results["results"].items():
                status = result.get("status", "unknown").upper()
                emoji = {
                    "healthy": "✅",
                    "missing_secret": "⚠️",
                    "not_configured": "⊘",
                    "auth_failed": "🔒",
                    "timeout": "⏱",
                    "failed": "❌",
                }.get(result.get("status"), "?")
                print(f"  {emoji} {provider_id}: {status}")
                if result.get("error"):
                    print(f"      Error: {result['error']}")
    else:
        result = asyncio.run(test_provider(args.provider, verbose=args.verbose))
        if args.json:
            print(json.dumps(result, indent=2))
        else:
            print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
