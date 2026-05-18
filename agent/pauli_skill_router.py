from __future__ import annotations

import logging
import os
import re
from pathlib import Path
from typing import Any

import yaml

logger = logging.getLogger(__name__)

_PRODUCTION_DEPLOY_RE = re.compile(
    r"\b(prod|production)\b.*\b(deploy|deployment|release|rollback)\b"
    r"|\b(deploy|deployment|release|rollback)\b.*\b(prod|production)\b",
    re.IGNORECASE,
)
_COMPLEX_REPO_RE = re.compile(
    r"\b(repo|repository|codebase|scan|inventory|audit|overview|pull request|pr|ci)\b",
    re.IGNORECASE,
)


def _repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def _default_router_config_path() -> Path:
    return _repo_root() / "config" / "pauli_skill_router.yaml"


def _default_profiles_config_path() -> Path:
    return _repo_root() / "config" / "pauli_profiles.yaml"


def _load_yaml(path: Path) -> dict[str, Any]:
    if not path.exists():
        return {}
    data = yaml.safe_load(path.read_text(encoding="utf-8")) or {}
    return data if isinstance(data, dict) else {}


def load_pauli_skill_router(config_path: str | Path | None = None) -> dict[str, Any]:
    path = Path(config_path) if config_path else _default_router_config_path()
    return _load_yaml(path)


def load_pauli_profiles(config_path: str | Path | None = None) -> dict[str, Any]:
    path = Path(config_path) if config_path else _default_profiles_config_path()
    return _load_yaml(path)


def router_available(config_path: str | Path | None = None) -> bool:
    path = Path(config_path) if config_path else _default_router_config_path()
    return path.exists()


def _custom_skill_aliases(repo_root: Path | None = None) -> dict[str, Path]:
    root = repo_root or _repo_root()
    skills_root = root / "skills" / "pauli"
    return {
        "zero-touch-engineer": skills_root / "zero-touch-engineer",
        "pauli-zero-touch-engineer": skills_root / "zero-touch-engineer",
        "jcodemunch": skills_root / "jcodemunch",
        "pauli-jcodemunch": skills_root / "jcodemunch",
        "pauli-coolify-ops": skills_root / "coolify-ops",
        "pauli-hostinger-vps": skills_root / "coolify-ops",
        "pauli-vercel-ops": skills_root / "vercel-ops",
        "pauli-infisical-secrets": skills_root / "infisical-secrets",
        "pauli-twilio-voice": skills_root / "twilio-voice",
        "pauli-supabase-memory": skills_root / "supabase-memory",
        "pauli-open-design": skills_root / "open-design",
        "pauli-taste-skill": skills_root / "taste-skill",
        "pauli-impeccable-design": skills_root / "impeccable-design",
        "pauli-video-watch": skills_root / "video-watch",
        "pauli-openmontage-studio": skills_root / "openmontage-studio",
        "pauli-fal-ai": skills_root / "fal-ai",
        "pauli-mythos-reasoning": skills_root / "mythos-reasoning",
    }


def resolve_skill_identifier(skill_name: str, repo_root: str | Path | None = None) -> str:
    mapping = _custom_skill_aliases(Path(repo_root) if repo_root else None)
    custom = mapping.get(skill_name)
    if custom and (custom / "SKILL.md").exists():
        return str(custom.resolve())
    return skill_name


def build_redacted_env_status(
    required_env: list[str] | tuple[str, ...],
    env: dict[str, str] | None = None,
) -> dict[str, str]:
    source = env if env is not None else os.environ
    return {
        key: "present" if bool(source.get(key)) else "missing"
        for key in required_env
    }


def _matched_trigger(text: str, triggers: list[str]) -> str | None:
    for trigger in triggers:
        normalized = trigger.strip()
        if not normalized:
            continue
        if re.fullmatch(r"[\w -]+", normalized):
            pattern = r"\b" + re.escape(normalized) + r"\b"
            if re.search(pattern, text, re.IGNORECASE):
                return trigger
            continue
        if normalized.lower() in text.lower():
            return trigger
    return None


def _select_budget_section(task_text: str, matched_routes: list[str]) -> str:
    if _PRODUCTION_DEPLOY_RE.search(task_text):
        return "production_deploy"
    if "video" in matched_routes:
        return "video_task"
    if _COMPLEX_REPO_RE.search(task_text):
        return "complex_repo_task"
    return "default"


def route_skills_for_task(
    task_text: str,
    *,
    explicit_skills: list[str] | None = None,
    profile: str | None = None,
    router_config_path: str | Path | None = None,
    profiles_config_path: str | Path | None = None,
    repo_root: str | Path | None = None,
) -> dict[str, Any]:
    router = load_pauli_skill_router(router_config_path)
    profiles = load_pauli_profiles(profiles_config_path)
    root = Path(repo_root) if repo_root else _repo_root()

    selected: list[str] = []
    reasons: list[str] = []
    matched_routes: list[str] = []
    always_load = list(router.get("always_load") or [])

    for skill in explicit_skills or []:
        if skill and skill not in selected:
            selected.append(skill)
            reasons.append(f"explicit skill: {skill}")

    profile_cfg = profiles.get(profile or "", {}) if profile else {}
    for skill in profile_cfg.get("default_loaded_skills", []) or []:
        if skill not in selected:
            selected.append(skill)
            reasons.append(f"profile {profile}: {skill}")

    for route_name, rule in (router.get("lazy_load_rules") or {}).items():
        if not isinstance(rule, dict):
            continue
        trigger = _matched_trigger(task_text, list(rule.get("triggers") or []))
        if not trigger:
            continue
        matched_routes.append(route_name)
        for skill in rule.get("skills", []) or []:
            if skill not in selected:
                selected.append(skill)
                reasons.append(f"route {route_name} via trigger '{trigger}': {skill}")

    budget_name = _select_budget_section(task_text, matched_routes)
    budget_cfg = ((router.get("tool_budget") or {}).get(budget_name) or {})
    default_budget_cfg = ((router.get("tool_budget") or {}).get("default") or {})
    max_skills = int(
        budget_cfg.get(
            "max_skills_loaded",
            default_budget_cfg.get("max_skills_loaded", len(selected) or 0),
        )
        or 0
    )

    kept = selected[:max_skills] if max_skills > 0 else list(selected)
    skipped = selected[max_skills:] if max_skills > 0 else []

    result = {
        "always_load": always_load,
        "selected_skills": kept,
        "resolved_skill_identifiers": [resolve_skill_identifier(skill, root) for skill in kept],
        "matched_routes": matched_routes,
        "selection_reasons": reasons[: len(kept)],
        "skipped_skills": skipped,
        "budget_name": budget_name,
        "max_skills_loaded": max_skills,
        "required_first": list(budget_cfg.get("required_first") or []),
        "paid_generation_default": budget_cfg.get("paid_generation_default", True),
        "approval_required": bool(
            budget_name == "production_deploy"
            and budget_cfg.get("requires_human_approval")
        ),
        "retrieval_mode": "search_only" if "memory" in matched_routes else "normal",
    }
    logger.info(
        "Pauli skill router selected routes=%s skills=%s skipped=%s",
        matched_routes,
        kept,
        skipped,
    )
    return result
