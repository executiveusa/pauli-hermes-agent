#!/usr/bin/env python3
"""Load canonical YAPPYVERSE multiplayer ICM context for Hermes.

This adapter keeps Hermes thin. Canonical multiplayer context lives in
executiveusa/YAPPYVERSE-FACTORY and is loaded at runtime from GitHub raw URLs
(or from a local checkout when YAPPYVERSE_ROOT is set).
"""

from __future__ import annotations

import argparse
import json
import os
import sys
import urllib.error
import urllib.request
from pathlib import Path

REPO = "executiveusa/YAPPYVERSE-FACTORY"
DEFAULT_REF = os.environ.get("YAPPYVERSE_REF", "main")
LOCAL_ROOT = os.environ.get("YAPPYVERSE_ROOT")
BASE = "ICM/multiplayer"
FILES = {
    "manifest": f"{BASE}/manifest.yaml",
    "system_prompt": f"{BASE}/MULTIPLAYER_SYSTEM_PROMPT.md",
    "team_graph": f"{BASE}/TEAM_GRAPH.json",
    "hermes_bridge": f"{BASE}/HERMES_BRIDGE.md",
    "client_state_contract": f"{BASE}/CLIENT_STATE_CONTRACT.md",
    "credit_workflow": f"{BASE}/credit/CREDIT_WORKFLOW.md",
}


def raw_url(path: str, ref: str) -> str:
    return f"https://raw.githubusercontent.com/{REPO}/{ref}/{path}"


def read_remote(path: str, ref: str) -> tuple[str, str]:
    url = raw_url(path, ref)
    req = urllib.request.Request(url, headers={"User-Agent": "pauli-hermes-yappyverse-multiplayer/1.0"})
    with urllib.request.urlopen(req, timeout=20) as resp:
        return resp.read().decode("utf-8"), url


def read_local(path: str) -> tuple[str, str]:
    root = Path(LOCAL_ROOT).resolve()
    candidate = (root / path).resolve()
    if root not in candidate.parents and candidate != root:
        raise ValueError("resolved path escaped YAPPYVERSE_ROOT")
    return candidate.read_text(encoding="utf-8"), str(candidate)


def read_path(path: str, ref: str) -> tuple[str, str]:
    if LOCAL_ROOT:
        return read_local(path)
    return read_remote(path, ref)


def validate_packet(packet: dict) -> None:
    required = ["manifest", "system_prompt", "team_graph", "hermes_bridge", "client_state_contract"]
    missing = [k for k in required if not packet["content"].get(k)]
    if missing:
        raise ValueError(f"missing required multiplayer context: {', '.join(missing)}")

    graph = json.loads(packet["content"]["team_graph"])
    people = {p.get("name") for p in graph.get("people", [])}
    if "Bambú" not in people:
        raise ValueError("team graph does not contain canonical Bambú identity")
    if "Stavarai" not in people:
        raise ValueError("team graph does not contain canonical Stavarai identity")

    companies = {c.get("name") for c in graph.get("companies", [])}
    if "Kupuri Media" not in companies:
        raise ValueError("team graph does not contain canonical Kupuri Media company")


def load(include_credit: bool, ref: str) -> dict:
    keys = ["manifest", "system_prompt", "team_graph", "hermes_bridge", "client_state_contract"]
    if include_credit:
        keys.append("credit_workflow")

    content = {}
    sources = {}
    for key in keys:
        text, source = read_path(FILES[key], ref)
        content[key] = text
        sources[key] = source

    packet = {
        "mode": "multiplayer-business-os",
        "orchestrator": {"name": "Bambú", "slug": "bambu", "aliases": ["Bambu", "Bamboo"]},
        "canonical_repo": REPO,
        "ref": ref,
        "local_root": LOCAL_ROOT,
        "include_credit": include_credit,
        "sources": sources,
        "content": content,
    }
    validate_packet(packet)
    return packet


def cmd_load(args: argparse.Namespace) -> int:
    try:
        packet = load(args.include_credit, args.ref)
    except (OSError, ValueError, json.JSONDecodeError, urllib.error.URLError) as exc:
        print(json.dumps({"status": "BLOCK", "error": str(exc), "repo": REPO, "ref": args.ref}, indent=2), file=sys.stderr)
        return 2

    print(json.dumps({"status": "PASS", **packet}, ensure_ascii=False, indent=2))
    return 0


def cmd_sources(args: argparse.Namespace) -> int:
    keys = ["manifest", "system_prompt", "team_graph", "hermes_bridge", "client_state_contract", "credit_workflow"]
    result = {k: (str((Path(LOCAL_ROOT) / FILES[k]).resolve()) if LOCAL_ROOT else raw_url(FILES[k], args.ref)) for k in keys}
    print(json.dumps({"repo": REPO, "ref": args.ref, "sources": result}, indent=2))
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(prog="yappyverse-multiplayer")
    sub = parser.add_subparsers(dest="command", required=True)

    p_load = sub.add_parser("load", help="load and validate canonical multiplayer context")
    p_load.add_argument("--include-credit", action="store_true")
    p_load.add_argument("--ref", default=DEFAULT_REF)
    p_load.set_defaults(func=cmd_load)

    p_sources = sub.add_parser("sources", help="show canonical source paths without fetching")
    p_sources.add_argument("--ref", default=DEFAULT_REF)
    p_sources.set_defaults(func=cmd_sources)

    args = parser.parse_args()
    return args.func(args)


if __name__ == "__main__":
    raise SystemExit(main())
