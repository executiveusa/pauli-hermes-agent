#!/usr/bin/env python3
"""Read-only 21st.dev CLI and MCP connection doctor.

The doctor never prints secret values. It checks local prerequisites, CLI presence,
authentication status, environment-variable presence, and common MCP config files.
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Any

MCP_ENDPOINT = "https://21st.dev/api/mcp"
CONFIG_CANDIDATES = (
    Path(".mcp.json"),
    Path.home() / ".cursor" / "mcp.json",
    Path.home() / ".codex" / "config.toml",
)


def run(command: list[str], timeout: int = 30) -> dict[str, Any]:
    try:
        result = subprocess.run(
            command,
            check=False,
            capture_output=True,
            text=True,
            timeout=timeout,
        )
    except (OSError, subprocess.TimeoutExpired) as exc:
        return {"ok": False, "error": str(exc)}
    combined = "\n".join(part for part in (result.stdout, result.stderr) if part).strip()
    return {
        "ok": result.returncode == 0,
        "returncode": result.returncode,
        "output": combined[:2000],
    }


def inspect_config(path: Path) -> dict[str, Any]:
    if not path.exists():
        return {"path": str(path), "exists": False, "mentions_21st": False}
    try:
        text = path.read_text(encoding="utf-8", errors="replace")
    except OSError as exc:
        return {"path": str(path), "exists": True, "readable": False, "error": str(exc)}
    lowered = text.lower()
    mentions = "21st.dev/api/mcp" in lowered or "api_key_21st" in lowered or "21st-dev" in lowered
    leaked_key_pattern = "21st_sk_" in text
    return {
        "path": str(path),
        "exists": True,
        "readable": True,
        "mentions_21st": mentions,
        "contains_literal_key_pattern": leaked_key_pattern,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="Read-only 21st.dev CLI/MCP doctor")
    parser.add_argument("--project", type=Path, default=Path.cwd())
    args = parser.parse_args()

    node = shutil.which("node")
    npm = shutil.which("npm")
    cli = shutil.which("21st")
    report: dict[str, Any] = {
        "status": "21ST_NOT_INSTALLED",
        "project": str(args.project.resolve()),
        "mcp_endpoint": MCP_ENDPOINT,
        "node_present": bool(node),
        "npm_present": bool(npm),
        "cli_present": bool(cli),
        "api_key_env_present": bool(os.environ.get("API_KEY_21ST")),
        "secret_values_printed": False,
        "configs": [],
    }

    candidates = [args.project / ".mcp.json"] + [p for p in CONFIG_CANDIDATES if p != Path(".mcp.json")]
    report["configs"] = [inspect_config(path) for path in candidates]

    if any(item.get("contains_literal_key_pattern") for item in report["configs"]):
        report["status"] = "21ST_CONFIG_SECRET_RISK"
        report["recommended_action"] = "Remove literal 21st_sk_ values and replace them with API_KEY_21ST references."
        print(json.dumps(report, indent=2))
        return 2

    if not node or not npm:
        report["status"] = "21ST_NODE_REQUIRED"
        report["recommended_action"] = "Install an approved Node.js LTS runtime before installing the 21st CLI."
        print(json.dumps(report, indent=2))
        return 1

    if not cli:
        report["recommended_action"] = "Run: npm i -g @21st-dev/cli"
        print(json.dumps(report, indent=2))
        return 1

    report["status"] = "21ST_CLI_INSTALLED"
    version = run([cli, "--version"])
    report["cli_version_check"] = version

    whoami = run([cli, "whoami"])
    report["auth_check"] = {"ok": whoami.get("ok", False)}
    if whoami.get("ok"):
        report["status"] = "21ST_AUTH_VERIFIED"
    else:
        report["status"] = "21ST_AUTH_REQUIRED"
        report["recommended_action"] = "Run 21st login and complete authentication in the owner-controlled browser."

    configured = any(item.get("mentions_21st") for item in report["configs"])
    report["mcp_config_present"] = configured
    if configured and report["status"] == "21ST_AUTH_VERIFIED":
        report["status"] = "21ST_MCP_CONFIGURED"
        report["verification_note"] = (
            "Configuration is present. A real authenticated MCP tool-list or search request is still required "
            "before reporting 21ST_MCP_VERIFIED."
        )

    print(json.dumps(report, indent=2))
    return 0 if report["status"] in {"21ST_AUTH_VERIFIED", "21ST_MCP_CONFIGURED"} else 1


if __name__ == "__main__":
    raise SystemExit(main())
