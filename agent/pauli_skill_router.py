"""Pauli skill router for Hermes.

The router loads a simple YAML policy, resolves the required skill files, and
classifies tasks for safety-sensitive execution.
"""

from __future__ import annotations

import logging
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import yaml

from agent.skill_commands import build_preloaded_skills_prompt
from pauli.openclaude.dispatcher import load_worker_registry

_REPO_ROOT = Path(__file__).resolve().parent.parent
DEFAULT_ROUTER_PATH = _REPO_ROOT / "config" / "pauli_skill_router.yaml"
DEFAULT_SKILL_ROOT = _REPO_ROOT / "skills" / "pauli"
logger = logging.getLogger(__name__)


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


def load_pauli_agent_policy(config: dict[str, Any] | None = None) -> dict[str, bool]:
    """Read the Pauli agent routing gates from a Hermes config payload."""
    agent_cfg = {}
    if isinstance(config, dict):
        agent_cfg = config.get("agent") or {}
        if not isinstance(agent_cfg, dict):
            agent_cfg = {}

    def _bool(name: str, default: bool = False) -> bool:
        value = agent_cfg.get(name, default)
        if isinstance(value, bool):
            return value
        if isinstance(value, (int, float)):
            return bool(value)
        if isinstance(value, str):
            lowered = value.strip().lower()
            if lowered in {"true", "1", "yes", "on"}:
                return True
            if lowered in {"false", "0", "no", "off"}:
                return False
        return default

    return {
        "pauli_profile": _bool("pauli_profile", False),
        "pauli_gateway_routing": _bool("pauli_gateway_routing", False),
        "pauli_required_skills_strict": _bool("pauli_required_skills_strict", False),
    }


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
    strict: bool | None = None,
    config_path: str | Path = DEFAULT_ROUTER_PATH,
    worker_registry_path: str | Path = _REPO_ROOT / "config" / "pauli_worker_registry.yaml",
    skill_root: str | Path = DEFAULT_SKILL_ROOT,
) -> dict[str, Any]:
    config = load_router_config(config_path)
    registry = load_worker_registry(worker_registry_path)
    router_cfg = config.get("router") or {}
    if not isinstance(router_cfg, dict):
        router_cfg = {}
    if strict is None:
        strict = bool(router_cfg.get("strict_missing_required", False))

    selected_skills = _dedupe(_select_required_skills(config))
    required_skills = list(selected_skills)
    missing_skills = [skill for skill in required_skills if not _skill_exists(skill, Path(skill_root))]
    skipped_skills = list(missing_skills)
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


def build_pauli_turn_context(
    task: str,
    context_prompt: str = "",
    *,
    task_id: str | None = None,
    config: dict[str, Any] | None = None,
    config_path: str | Path = DEFAULT_ROUTER_PATH,
    worker_registry_path: str | Path = _REPO_ROOT / "config" / "pauli_worker_registry.yaml",
    skill_root: str | Path = DEFAULT_SKILL_ROOT,
) -> dict[str, Any]:
    """Build the gateway turn payload after applying Pauli routing rules."""
    policy = load_pauli_agent_policy(config)
    route = route_task(
        task,
        strict=policy["pauli_required_skills_strict"],
        config_path=config_path,
        worker_registry_path=worker_registry_path,
        skill_root=skill_root,
    )

    if route["blocked"]:
        logger.info(
            "Pauli route blocked for task %r: selected_skills=%s required_skills=%s missing_skills=%s skipped_skills=%s block_reason=%s",
            task,
            route["selected_skills"],
            route["required_skills"],
            route["missing_skills"],
            route["skipped_skills"],
            route["block_reason"],
        )
        return {
            "enabled": policy["pauli_profile"],
            "gateway_routing": policy["pauli_gateway_routing"],
            "strict_required_skills": policy["pauli_required_skills_strict"],
            "route": route,
            "selected_skills": route["selected_skills"],
            "required_skills": route["required_skills"],
            "missing_skills": route["missing_skills"],
            "skipped_skills": route["skipped_skills"],
            "loaded_skills": [],
            "skills_prompt": "",
            "combined_ephemeral": context_prompt,
            "blocked": True,
            "block_reason": route["block_reason"],
        }

    if not (policy["pauli_profile"] and policy["pauli_gateway_routing"]):
        logger.info(
            "Pauli routing disabled for task %r: enabled=%s gateway_routing=%s selected_skills=%s required_skills=%s missing_skills=%s skipped_skills=%s",
            task,
            policy["pauli_profile"],
            policy["pauli_gateway_routing"],
            route["selected_skills"],
            route["required_skills"],
            route["missing_skills"],
            route["skipped_skills"],
        )
        return {
            "enabled": policy["pauli_profile"],
            "gateway_routing": policy["pauli_gateway_routing"],
            "strict_required_skills": policy["pauli_required_skills_strict"],
            "route": route,
            "selected_skills": route["selected_skills"],
            "required_skills": route["required_skills"],
            "missing_skills": route["missing_skills"],
            "skipped_skills": route["skipped_skills"],
            "loaded_skills": [],
            "skills_prompt": "",
            "combined_ephemeral": context_prompt,
            "blocked": False,
            "block_reason": "",
        }

    skills_prompt, loaded_skills, missing_skills = build_preloaded_skills_prompt(
        route["selected_skills"],
        task_id=task_id,
    )
    if missing_skills and not policy["pauli_required_skills_strict"]:
        logger.warning(
            "Pauli routing missing optional skill files for task %r: missing_skills=%s selected_skills=%s skipped_skills=%s",
            task,
            missing_skills,
            route["selected_skills"],
            route["skipped_skills"],
        )

    combined_ephemeral = context_prompt or ""
    if skills_prompt:
        combined_ephemeral = (combined_ephemeral + "\n\n" + skills_prompt).strip()

    logger.info(
        "Pauli route applied for task %r: selected_skills=%s required_skills=%s missing_skills=%s skipped_skills=%s loaded_skills=%s",
        task,
        route["selected_skills"],
        route["required_skills"],
        missing_skills,
        route["skipped_skills"],
        loaded_skills,
    )

    return {
        "enabled": policy["pauli_profile"],
        "gateway_routing": policy["pauli_gateway_routing"],
        "strict_required_skills": policy["pauli_required_skills_strict"],
        "route": route,
        "selected_skills": route["selected_skills"],
        "required_skills": route["required_skills"],
        "missing_skills": missing_skills,
        "skipped_skills": route["skipped_skills"],
        "loaded_skills": loaded_skills,
        "skills_prompt": skills_prompt,
        "combined_ephemeral": combined_ephemeral,
        "blocked": False,
        "block_reason": "",
    }
