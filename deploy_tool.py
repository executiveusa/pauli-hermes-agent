"""Coolify deployment helper."""

from __future__ import annotations

import logging
import os
from typing import Any, Dict

import requests

logger = logging.getLogger(__name__)


STACK_CONFIG = {
    "nextjs": {"runtime": "node", "build_command": "npm run build", "start_command": "npm run start"},
    "node": {"runtime": "node", "build_command": "npm install", "start_command": "npm start"},
    "python": {"runtime": "python", "build_command": "pip install -r requirements.txt", "start_command": "uvicorn main:app --host 0.0.0.0 --port 8000"},
    "static": {"runtime": "static", "build_command": "", "start_command": ""},
}


def _require_env(name: str) -> str:
    value = os.getenv(name, "").strip()
    if not value:
        raise RuntimeError(f"Missing required environment variable: {name}")
    return value


def deploy_repo(repo_url: str, stack: str) -> Dict[str, Any]:
    """Trigger deployment via Coolify and return deployment metadata."""

    logging.basicConfig(level=logging.INFO)

    api_key = _require_env("COOLIFY_API_KEY")
    base_url = _require_env("COOLIFY_BASE_URL").rstrip("/")

    config = STACK_CONFIG.get(stack)
    if not config:
        raise ValueError(f"Unsupported stack '{stack}'. Expected one of: {', '.join(STACK_CONFIG)}")

    payload = {
        "repository_url": repo_url,
        "stack": stack,
        "config": config,
    }
    headers = {
        "Authorization": f"Bearer {api_key}",
        "Content-Type": "application/json",
        "Accept": "application/json",
    }

    logger.info("Queueing deployment for %s (stack=%s)", repo_url, stack)

    try:
        response = requests.post(f"{base_url}/api/v1/deploy", json=payload, headers=headers, timeout=30)
        response.raise_for_status()
        data = response.json()
        deployment_id = str(data.get("deployment_id") or data.get("id") or "")
        result = {"deployment_id": deployment_id, "status": data.get("status", "queued")}
        logger.info("Deployment queued: %s", result)
        return result
    except requests.RequestException:
        logger.exception("Coolify deployment request failed for %s", repo_url)
        raise


if __name__ == "__main__":
    import sys

    if len(sys.argv) < 3:
        raise SystemExit("Usage: python deploy_tool.py <repo_url> <stack>")
    print(deploy_repo(sys.argv[1], sys.argv[2]))
