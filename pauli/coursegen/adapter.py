from __future__ import annotations

from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[2]
VENDOR_ROOT = REPO_ROOT.parent / "vendor-repos"
COURSEGEN_ROOT = VENDOR_ROOT / "codebase-to-course"


def inspect_coursegen() -> dict[str, Any]:
    skill_path = COURSEGEN_ROOT / "SKILL.md"
    readme_path = COURSEGEN_ROOT / "README.md"
    blockers: list[str] = []

    if not COURSEGEN_ROOT.exists():
        blockers.append("missing_checkout")
    if not skill_path.exists():
        blockers.append("missing_skill_definition")

    status = "installed" if COURSEGEN_ROOT.exists() and skill_path.exists() else "missing"
    if blockers and status == "installed":
        status = "partial"

    return {
        "name": "codebase-to-course",
        "path": str(COURSEGEN_ROOT),
        "path_exists": COURSEGEN_ROOT.exists(),
        "skill_path": str(skill_path),
        "skill_exists": skill_path.exists(),
        "readme_exists": readme_path.exists(),
        "status": status,
        "blockers": blockers,
        "default_outputs": {
            "course_html": "COURSE.html",
            "course_metadata": "course.json",
        },
    }
