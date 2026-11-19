"""Legacy autonomous deployment entrypoint — FAIL CLOSED by owner decision.

Cosmos/Hermes is the spec-and-handoff architect, not a deployment runner.
CodeDen is the sole coding factory. This compatibility shim intentionally
contains no model, database, deploy, cron, or networking calls.

Do not reactivate by setting an environment variable, providing a local
approval JSON, or editing a branch. A future owner-approved PR must integrate
a *verified external privileged policy boundary* and independent review.
"""
from __future__ import annotations

import json
import os
from typing import Any, Dict


class OwnerGateRequired(RuntimeError):
    """Direct autonomous deployment is disabled pending external owner gate."""


def _require_env(name: str) -> str:
    """Preserved pure helper for callers that previously imported it."""
    value = os.getenv(name, "").strip()
    if not value:
        raise RuntimeError(f"Missing required environment variable: {name}")
    return value


def _parse_agent_decision(content: str) -> Dict[str, Any]:
    """Pure legacy parser; a model claim never grants deployment authority."""
    try:
        result = json.loads(content)
        return result if isinstance(result, dict) else {"deploy": False, "reason": "Invalid decision shape"}
    except (json.JSONDecodeError, TypeError):
        return {"deploy": False, "reason": "Invalid model decision"}


def run_autonomous_loop(sleep_seconds: int = 60, max_retries: int = 3) -> None:
    """HOLD before any side effects. No environment flag can override this.

    The owner gate is deliberately not simulated with a bool, JSON file,
    GitHub issue comment, agent message, or local secret. A future reviewed
    trusted policy integration must authorize each exact action, repository,
    revision, cost and release. Cosmos must never become a parallel factory.
    """
    raise OwnerGateRequired(
        "DISABLED: Cosmos cannot run autonomous deployment pipelines. "
        "Use an authenticated Paperclip mission and an owner-approved CodeDen "
        "handoff; a separately verified privileged owner-release gate is required."
    )


if __name__ == "__main__":
    run_autonomous_loop()
