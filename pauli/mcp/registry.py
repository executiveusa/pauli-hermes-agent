from __future__ import annotations

import os
import platform
from dataclasses import dataclass, asdict
from pathlib import Path
from typing import Any


REPO_ROOT = Path(__file__).resolve().parents[2]
VENDOR_ROOT = REPO_ROOT.parent / "vendor-repos"


@dataclass(frozen=True)
class ModuleSpec:
    name: str
    vendor_dir: str
    category: str
    required_env: tuple[str, ...] = ()
    supported_platforms: tuple[str, ...] = ()
    notes: str = ""


MODULE_SPECS: tuple[ModuleSpec, ...] = (
    ModuleSpec(
        name="OpenChronicle",
        vendor_dir="OpenChronicle",
        category="memory_layer",
        supported_platforms=("Darwin",),
        notes="Preferred browser/work-context memory on supported macOS hosts.",
    ),
    ModuleSpec(
        name="browser-harness",
        vendor_dir="browser-harness",
        category="browser_harness",
        required_env=("BROWSER_USE_API_KEY",),
        notes="Primary browser-control path. API key only required for remote cloud browsers.",
    ),
    ModuleSpec(
        name="jcodemunch-mcp",
        vendor_dir="jcodemunch-mcp",
        category="mcp_server",
        notes="Structured retrieval/token-saver. Commercial license required for business use.",
    ),
    ModuleSpec(
        name="ralphy",
        vendor_dir="ralphy",
        category="orchestrator",
        notes="Autonomous PRD/task execution loop to adapt into Pauli flywheel wrappers.",
    ),
    ModuleSpec(
        name="ext-apps",
        vendor_dir="ext-apps",
        category="mcp_apps",
        notes="MCP external-app bridge candidates.",
    ),
    ModuleSpec(
        name="mcp2cli",
        vendor_dir="mcp2cli",
        category="mcp_bridge",
        notes="Bridge layer for MCP-to-CLI integration.",
    ),
    ModuleSpec(
        name="supabase-mcp",
        vendor_dir="supabase-mcp",
        category="mcp_server",
        required_env=("SUPABASE_URL", "SUPABASE_ACCESS_TOKEN"),
        notes="Supabase MCP support for memory and data operations.",
    ),
    ModuleSpec(
        name="mattpocock-skills",
        vendor_dir="mattpocock-skills",
        category="skill_library",
        notes="Reusable skill references and patterns.",
    ),
    ModuleSpec(
        name="codebase-to-course",
        vendor_dir="codebase-to-course",
        category="course_generator",
        notes="Interactive single-page course generation reference implementation.",
    ),
)


def _platform_supported(spec: ModuleSpec) -> tuple[bool, str | None]:
    if not spec.supported_platforms:
        return True, None
    current = platform.system()
    if current in spec.supported_platforms:
        return True, None
    return False, f"platform_blocked:{current}"


def _env_status(spec: ModuleSpec) -> dict[str, str]:
    statuses: dict[str, str] = {}
    for key in spec.required_env:
        value = os.environ.get(key)
        statuses[key] = "present" if value else "missing"
    return statuses


def inspect_module(spec: ModuleSpec) -> dict[str, Any]:
    path = VENDOR_ROOT / spec.vendor_dir
    platform_ok, platform_reason = _platform_supported(spec)
    env_status = _env_status(spec)
    env_ok = all(status == "present" for status in env_status.values()) if env_status else True

    status = "installed" if path.exists() else "missing"
    blockers: list[str] = []

    if not path.exists():
        blockers.append("missing_checkout")
    if not platform_ok and platform_reason:
        blockers.append(platform_reason)
    if spec.required_env and not env_ok:
        blockers.append("missing_env")

    if blockers:
        if path.exists():
            status = "blocked"

    payload = asdict(spec)
    payload.update(
        {
            "path": str(path),
            "path_exists": path.exists(),
            "env_status": env_status,
            "platform_ok": platform_ok,
            "status": status,
            "blockers": blockers,
        }
    )
    return payload


def inspect_all_modules() -> list[dict[str, Any]]:
    return [inspect_module(spec) for spec in MODULE_SPECS]
