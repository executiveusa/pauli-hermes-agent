"""Pauli skill router for Hermes.

The router loads a simple YAML policy, resolves the required skill files, and
classifies tasks for safety-sensitive execution.
"""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Any

import yaml

from pauli.openclaude.dispatcher import load_worker_registry

DEFAULT_ROUTER_PATH = Path("config/pauli_skill_router.yaml")
DEFAULT_SKILL_ROOT = Path("skills/pauli")


@dataclass(frozen=True)
class PauliRouteResult:
    task: str
    task_type: str
    selected_skills: list[str]
    required_skills: list[str]
    missing_skills: list[str]
    skipped_skills: list[str]
    blocked: bool
    block_reason: str

    def to_dict(self) -> dict[str, Any]:
        return {
            "task": self.task,
            "task_type": self.task_type,
            "selected_skills": self.selected_skills,
            "required_skills": self.required_skills,
            "missing_skills": self.missing_skills,
            "skipped_skills": self.skipped_skills,
            "blocked": self.blocked,
            "block_reason": self.block_reason,
        }


def load_router_config(path: str | Path = DEFAULT_ROUTER_PATH) -> dict[str, Any]:
    router_path = Path(path)
    if not router_path.exists():
        return {}
    with router_path.open("r", encoding="utf-8") as handle:
        data = yaml.safe_load(handle) or {}
    if not isinstance(data, dict):
        return {}
    return data


def _dedupe(items: list[str]) -> list[str]:
    seen: set[str] = set()
    ordered: list[str] = []
    for item in items:
        if item not in seen:
            seen.add(item)
            ordered.append(item)
    return ordered


def _skill_exists(skill_name: str, skill_root: Path = DEFAULT_SKILL_ROOT) -> bool:
    return (skill_root / skill_name / "SKILL.md").exists()


def _select_required_skills(config: dict[str, Any]) -> list[str]:
    profile_name = str(config.get("default_profile", "hermes_operator") or "hermes_operator")
    profiles = config.get("profiles") or {}
    profile = profiles.get(profile_name) or {}
    required = profile.get("required_skills") or config.get("required_skills") or []
    if not isinstance(required, list):
        return []
    return [str(skill).strip() for skill in required if str(skill).strip()]


def _classify_task(task: str, registry: dict[str, Any]) -> str:
    task_text = str(task or "").strip().lower()
    safety_cfg = registry.get("safety") or {}
    keywords = safety_cfg.get("destructive_keywords") or []
    if not isinstance(keywords, list):
        keywords = []
    for keyword in keywords:
        if str(keyword).strip().lower() and str(keyword).strip().lower() in task_text:
            return "destructive"
    return "safe"


def route_task(
    task: str,
    strict: bool = False,
    config_path: str | Path = DEFAULT_ROUTER_PATH,
    worker_registry_path: str | Path = Path("config/pauli_worker_registry.yaml"),
    skill_root: str | Path = DEFAULT_SKILL_ROOT,
) -> dict[str, Any]:
    config = load_router_config(config_path)
    registry = load_worker_registry(worker_registry_path)
    selected_skills = _dedupe(_select_required_skills(config))
    required_skills = list(selected_skills)
    missing_skills = [skill for skill in required_skills if not _skill_exists(skill, Path(skill_root))]
    skipped_skills: list[str] = []
    task_type = _classify_task(task, registry)
    blocked = task_type == "destructive"
    block_reason = "destructive task blocked by Pauli policy" if blocked else ""

    if strict and missing_skills:
        blocked = True
        block_reason = f"missing required skills: {', '.join(missing_skills)}"

    return PauliRouteResult(
        task=task,
        task_type=task_type,
        selected_skills=selected_skills,
        required_skills=required_skills,
        missing_skills=missing_skills,
        skipped_skills=skipped_skills,
        blocked=blocked,
        block_reason=block_reason,
    ).to_dict()
