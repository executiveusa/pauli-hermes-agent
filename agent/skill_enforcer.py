"""Skill enforcement helpers for Hermes.

Provides lightweight checks for skills declared in `skills/SKILL_REGISTRY.json`.
These helpers are intentionally conservative: they never raise and they provide
small, easily-testable primitives callers can use to gate execution or prompt
operators before performing sensitive actions.
"""
from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Dict, List, Tuple


def _registry_path() -> Path:
    # repo root is parent of the `agent` package directory
    return Path(__file__).parent.parent.joinpath("skills", "SKILL_REGISTRY.json")


def load_registry() -> Dict[str, dict]:
    """Load SKILL_REGISTRY.json, returning an empty dict on error."""
    try:
        p = _registry_path()
        if not p.exists():
            return {}
        return json.loads(p.read_text(encoding="utf-8")) or {}
    except Exception:
        return {}


def load_required_skills() -> List[str]:
    reg = load_registry()
    return [name for name, meta in (reg or {}).items() if isinstance(meta, dict) and meta.get("required")]


def verify_required_skills() -> Tuple[bool, List[str], List[str]]:
    """Verify presence of required skills on disk.

    Returns (all_present, present_list, missing_list).
    Never raises.
    """
    try:
        required = load_required_skills()
        present = []
        missing = []
        skills_dir = Path(__file__).parent.parent.joinpath("skills")
        for name in required:
            if (skills_dir / name).exists():
                present.append(name)
            else:
                missing.append(name)
        return (len(missing) == 0, present, missing)
    except Exception:
        return (False, [], load_required_skills())


def prompt_model_selection(clarify_callback=None, default: str | None = None) -> str | None:
    """Prompt the operator to select a model when a switch is recommended.

    If `clarify_callback` is provided, it will be called as
        clarify_callback(prompt_text, options)
    and must return the chosen option string. Otherwise falls back to `input()`.
    """
    prompt = (
        "Model selection recommended before performing code execution.\n"
        "Options: [1] gpt-5-mini (fast, token-optimized), [2] gpt-5 (full capability), [3] keep current\n"
        "Choose 1/2/3: "
    )
    if clarify_callback:
        try:
            choice = clarify_callback("Select model for this task", ["gpt-5-mini", "gpt-5", "keep_current"])
            return choice
        except Exception:
            pass
    if not sys.stdin or not sys.stdin.isatty():
        return default
    try:
        val = input(prompt).strip()
        if val == "1":
            return "gpt-5-mini"
        if val == "2":
            return "gpt-5"
        return default
    except Exception:
        return default
