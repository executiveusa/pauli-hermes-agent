from __future__ import annotations

import shutil
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[2]
VENDOR_ROOT = REPO_ROOT.parent / "vendor-repos"
RALPHY_ROOT = VENDOR_ROOT / "ralphy"


def inspect_ralphy() -> dict[str, Any]:
    script_path = RALPHY_ROOT / "ralphy.sh"
    global_cli = shutil.which("ralphy")
    readme_path = RALPHY_ROOT / "README.md"

    blockers: list[str] = []
    if not RALPHY_ROOT.exists():
        blockers.append("missing_checkout")
    if not script_path.exists():
        blockers.append("missing_script")
    if not global_cli:
        blockers.append("missing_global_cli")

    status = "installed" if RALPHY_ROOT.exists() and script_path.exists() else "missing"
    if blockers and status == "installed":
        status = "partial"

    return {
        "name": "ralphy",
        "path": str(RALPHY_ROOT),
        "path_exists": RALPHY_ROOT.exists(),
        "script_path": str(script_path),
        "script_exists": script_path.exists(),
        "global_cli": global_cli or "",
        "readme_exists": readme_path.exists(),
        "status": status,
        "blockers": blockers,
    }


def build_flywheel_bootstrap_command() -> list[str]:
    info = inspect_ralphy()
    if info["global_cli"]:
        return [info["global_cli"], "--init"]
    return [str(RALPHY_ROOT / "ralphy.sh"), "--init"]
