"""Safe OpenClaude dispatcher shim for Pauli.

The real command execution layer is intentionally conservative:
- load a worker registry from config/pauli_worker_registry.yaml
- classify tasks as safe or destructive
- block destructive tasks unless the caller explicitly handles approval
"""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
from typing import Any, Iterable

import yaml

DEFAULT_WORKER_REGISTRY_PATH = Path("config/pauli_worker_registry.yaml")

_DEFAULT_DANGEROUS_KEYWORDS = (
    "delete",
    "destroy",
    "drop database",
    "rm -rf",
    "git push",
    "deploy",
    "auth",
    "payment",
    "secret",
    "password",
)


def load_worker_registry(path: str | Path = DEFAULT_WORKER_REGISTRY_PATH) -> dict[str, Any]:
    registry_path = Path(path)
    if not registry_path.exists():
        return {
            "version": 1,
            "workers": {
                "openclaude": {
                    "enabled": True,
                    "model_priority": ["local/free", "cheap", "standard"],
                    "denied_task_types": ["deploy", "git_push", "file_delete", "db_migration", "auth_edit", "payment_edit"],
                    "approval_required_task_types": ["deploy", "git_push", "file_delete", "db_migration", "auth_edit", "payment_edit"],
                }
            },
            "safety": {"destructive_keywords": list(_DEFAULT_DANGEROUS_KEYWORDS)},
        }
    with registry_path.open("r", encoding="utf-8") as handle:
        data = yaml.safe_load(handle) or {}
    if not isinstance(data, dict):
        return {}
    return data


def _flatten_keywords(values: Iterable[str]) -> tuple[str, ...]:
    items: list[str] = []
    for value in values:
        text = str(value or "").strip().lower()
        if text:
            items.append(text)
    return tuple(items)


class OpenClaudeDispatcher:
    """Minimal dispatch shim that classifies tasks against a worker registry."""

    def __init__(self, registry_path: str | Path = DEFAULT_WORKER_REGISTRY_PATH) -> None:
        self.registry_path = Path(registry_path)
        self.registry = load_worker_registry(self.registry_path)

    def classify_task(self, task: str) -> str:
        task_text = str(task or "").strip().lower()
        keywords = self._destructive_keywords()
        if any(keyword in task_text for keyword in keywords):
            return "destructive"
        return "safe"

    def dispatch(self, task: str) -> dict[str, Any]:
        task_type = self.classify_task(task)
        worker_cfg = (self.registry.get("workers") or {}).get("openclaude", {})
        allowed = bool(worker_cfg.get("enabled", True)) and task_type != "destructive"
        reason = "safe task routed to OpenClaude" if allowed else "destructive task blocked by worker registry"
        return {
            "task": task,
            "task_type": task_type,
            "allowed": allowed,
            "reason": reason,
            "registry_path": str(self.registry_path),
            "model_priority": worker_cfg.get("model_priority", []),
        }

    def _destructive_keywords(self) -> tuple[str, ...]:
        safety_cfg = self.registry.get("safety", {})
        keywords = safety_cfg.get("destructive_keywords", list(_DEFAULT_DANGEROUS_KEYWORDS))
        if not isinstance(keywords, list):
            keywords = list(_DEFAULT_DANGEROUS_KEYWORDS)
        return _flatten_keywords(keywords)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Dispatch a task through the Pauli OpenClaude safety shim.")
    parser.add_argument("task", help="Task to classify and dispatch")
    parser.add_argument("--registry", default=str(DEFAULT_WORKER_REGISTRY_PATH), help="Worker registry path")
    args = parser.parse_args(argv)

    dispatcher = OpenClaudeDispatcher(args.registry)
    print(json.dumps(dispatcher.dispatch(args.task), indent=2, sort_keys=True))
    return 0


def doctor(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Inspect the OpenClaude registry and task safety rules.")
    parser.add_argument("--registry", default=str(DEFAULT_WORKER_REGISTRY_PATH), help="Worker registry path")
    args = parser.parse_args(argv)

    registry = load_worker_registry(args.registry)
    summary = {
        "registry_path": str(Path(args.registry)),
        "enabled": bool((registry.get("workers") or {}).get("openclaude", {}).get("enabled", True)),
        "model_priority": (registry.get("workers") or {}).get("openclaude", {}).get("model_priority", []),
        "destructive_keywords": (registry.get("safety") or {}).get("destructive_keywords", []),
    }
    print(json.dumps(summary, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":  # pragma: no cover - script entry point
    raise SystemExit(main())
