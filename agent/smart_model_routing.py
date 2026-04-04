"""Helpers for optional cheap-vs-strong model routing.

Includes cost-law tiers for the Studio Orchestrator:
  - trivial: greetings, simple lookups → mercury/gemini-flash
  - standard: coding, analysis → sonnet/gpt-4o
  - complex: architecture, long reasoning → opus/o1
"""

from __future__ import annotations

import os
import re
from typing import Any, Dict, Optional

# ---------------------------------------------------------------------------
# Cost-law tier classification (used by Studio Orchestrator cost-law skill)
# ---------------------------------------------------------------------------

COST_TIERS = {
    "trivial": {
        "description": "Simple chat, greetings, lookups",
        "default_model": "openrouter/google/gemini-2.5-flash-preview",
        "max_chars": 160,
        "max_words": 28,
    },
    "standard": {
        "description": "Coding, analysis, research",
        "default_model": "anthropic/claude-sonnet-4-20250514",
    },
    "complex": {
        "description": "Architecture, long reasoning, multi-step planning",
        "default_model": "anthropic/claude-opus-4-20250514",
    },
}


def classify_task_tier(message: str) -> str:
    """Classify a user message into a cost tier: trivial, standard, or complex.

    This is a heuristic classifier. The smart_model_routing config-based
    system takes precedence — this is for informational/logging purposes
    and for the cost-law skill's routing table.
    """
    text = (message or "").strip()
    if not text:
        return "trivial"

    words = text.lower().split()
    word_count = len(words)

    # Complex indicators
    complex_keywords = {
        "architect", "architecture", "design", "redesign", "refactor",
        "migrate", "migration", "plan", "planning", "strategy",
        "multi-step", "multi-agent", "orchestrate", "deploy",
        "production", "infrastructure", "kubernetes", "terraform",
    }
    if {w.strip(".,;:!?") for w in words} & complex_keywords:
        return "complex"

    # Long messages are likely complex
    if word_count > 100 or len(text) > 800:
        return "complex"

    # Code blocks indicate standard+
    if "```" in text or text.count("\n") > 5:
        return "standard"

    # Standard indicators
    standard_keywords = {
        "debug", "implement", "fix", "test", "analyze", "code",
        "function", "class", "error", "exception", "optimize",
        "review", "search", "write", "create", "build",
    }
    if {w.strip(".,;:!?") for w in words} & standard_keywords:
        return "standard"

    # Short simple messages
    if word_count <= 28 and len(text) <= 160:
        return "trivial"

    return "standard"

_COMPLEX_KEYWORDS = {
    "debug",
    "debugging",
    "implement",
    "implementation",
    "refactor",
    "patch",
    "traceback",
    "stacktrace",
    "exception",
    "error",
    "analyze",
    "analysis",
    "investigate",
    "architecture",
    "design",
    "compare",
    "benchmark",
    "optimize",
    "optimise",
    "review",
    "terminal",
    "shell",
    "tool",
    "tools",
    "pytest",
    "test",
    "tests",
    "plan",
    "planning",
    "delegate",
    "subagent",
    "cron",
    "docker",
    "kubernetes",
}

_URL_RE = re.compile(r"https?://|www\.", re.IGNORECASE)


def _coerce_bool(value: Any, default: bool = False) -> bool:
    if value is None:
        return default
    if isinstance(value, bool):
        return value
    if isinstance(value, str):
        return value.strip().lower() in {"1", "true", "yes", "on"}
    return bool(value)


def _coerce_int(value: Any, default: int) -> int:
    try:
        return int(value)
    except (TypeError, ValueError):
        return default


def choose_cheap_model_route(user_message: str, routing_config: Optional[Dict[str, Any]]) -> Optional[Dict[str, Any]]:
    """Return the configured cheap-model route when a message looks simple.

    Conservative by design: if the message has signs of code/tool/debugging/
    long-form work, keep the primary model.
    """
    cfg = routing_config or {}
    if not _coerce_bool(cfg.get("enabled"), False):
        return None

    cheap_model = cfg.get("cheap_model") or {}
    if not isinstance(cheap_model, dict):
        return None
    provider = str(cheap_model.get("provider") or "").strip().lower()
    model = str(cheap_model.get("model") or "").strip()
    if not provider or not model:
        return None

    text = (user_message or "").strip()
    if not text:
        return None

    max_chars = _coerce_int(cfg.get("max_simple_chars"), 160)
    max_words = _coerce_int(cfg.get("max_simple_words"), 28)

    if len(text) > max_chars:
        return None
    if len(text.split()) > max_words:
        return None
    if text.count("\n") > 1:
        return None
    if "```" in text or "`" in text:
        return None
    if _URL_RE.search(text):
        return None

    lowered = text.lower()
    words = {token.strip(".,:;!?()[]{}\"'`") for token in lowered.split()}
    if words & _COMPLEX_KEYWORDS:
        return None

    route = dict(cheap_model)
    route["provider"] = provider
    route["model"] = model
    route["routing_reason"] = "simple_turn"
    return route


def resolve_turn_route(user_message: str, routing_config: Optional[Dict[str, Any]], primary: Dict[str, Any]) -> Dict[str, Any]:
    """Resolve the effective model/runtime for one turn.

    Returns a dict with model/runtime/signature/label fields.
    """
    route = choose_cheap_model_route(user_message, routing_config)
    if not route:
        return {
            "model": primary.get("model"),
            "runtime": {
                "api_key": primary.get("api_key"),
                "base_url": primary.get("base_url"),
                "provider": primary.get("provider"),
                "api_mode": primary.get("api_mode"),
                "command": primary.get("command"),
                "args": list(primary.get("args") or []),
            },
            "label": None,
            "signature": (
                primary.get("model"),
                primary.get("provider"),
                primary.get("base_url"),
                primary.get("api_mode"),
                primary.get("command"),
                tuple(primary.get("args") or ()),
            ),
        }

    from hermes_cli.runtime_provider import resolve_runtime_provider

    explicit_api_key = None
    api_key_env = str(route.get("api_key_env") or "").strip()
    if api_key_env:
        explicit_api_key = os.getenv(api_key_env) or None

    try:
        runtime = resolve_runtime_provider(
            requested=route.get("provider"),
            explicit_api_key=explicit_api_key,
            explicit_base_url=route.get("base_url"),
        )
    except Exception:
        return {
            "model": primary.get("model"),
            "runtime": {
                "api_key": primary.get("api_key"),
                "base_url": primary.get("base_url"),
                "provider": primary.get("provider"),
                "api_mode": primary.get("api_mode"),
                "command": primary.get("command"),
                "args": list(primary.get("args") or []),
            },
            "label": None,
            "signature": (
                primary.get("model"),
                primary.get("provider"),
                primary.get("base_url"),
                primary.get("api_mode"),
                primary.get("command"),
                tuple(primary.get("args") or ()),
            ),
        }

    return {
        "model": route.get("model"),
        "runtime": {
            "api_key": runtime.get("api_key"),
            "base_url": runtime.get("base_url"),
            "provider": runtime.get("provider"),
            "api_mode": runtime.get("api_mode"),
            "command": runtime.get("command"),
            "args": list(runtime.get("args") or []),
        },
        "label": f"smart route → {route.get('model')} ({runtime.get('provider')})",
        "signature": (
            route.get("model"),
            runtime.get("provider"),
            runtime.get("base_url"),
            runtime.get("api_mode"),
            runtime.get("command"),
            tuple(runtime.get("args") or ()),
        ),
    }
