#!/usr/bin/env python3
"""Write a structured Dear Diary entry through Terabithia.

Secrets are read only from environment variables and never printed.
"""
from __future__ import annotations

import argparse
import json
import os
import sys
import urllib.error
import urllib.request


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description="Record a shared operational memory through Terabithia")
    p.add_argument("--type", choices=["decision", "fact", "change", "lesson", "blocker", "outcome"], required=True)
    p.add_argument("--summary", required=True)
    p.add_argument("--reason")
    p.add_argument("--project")
    p.add_argument("--agent", action="append", dest="affected_agents", default=[])
    p.add_argument("--context-ref", action="append", dest="context_refs", default=[])
    p.add_argument("--evidence-ref", action="append", dest="evidence_refs", default=[])
    p.add_argument("--supersedes")
    p.add_argument("--sensitivity", choices=["shared", "restricted", "private"], default="shared")
    return p.parse_args()


def main() -> int:
    args = parse_args()
    if args.sensitivity == "private":
        print("Refusing to publish private memory to the shared Dear Diary ledger.", file=sys.stderr)
        return 2

    base = os.environ.get("TERABITHIA_URL", "").strip().rstrip("/")
    key = os.environ.get("TERABITHIA_API_KEY", "").strip()
    if not base or not key:
        print("TERABITHIA_URL and TERABITHIA_API_KEY are required.", file=sys.stderr)
        return 2

    payload = {
        "type": args.type,
        "summary": args.summary.strip(),
        "reason": args.reason,
        "project": args.project,
        "affected_agents": args.affected_agents,
        "context_refs": args.context_refs,
        "evidence_refs": args.evidence_refs,
        "supersedes": args.supersedes,
        "sensitivity": args.sensitivity,
    }
    payload = {k: v for k, v in payload.items() if v not in (None, [], "")}

    request = urllib.request.Request(
        f"{base}/api/v1/diary",
        data=json.dumps(payload).encode("utf-8"),
        method="POST",
        headers={
            "Authorization": f"Bearer {key}",
            "Content-Type": "application/json",
        },
    )
    try:
        with urllib.request.urlopen(request, timeout=15) as response:
            body = json.loads(response.read().decode("utf-8") or "{}")
            print(json.dumps({
                "ok": True,
                "decision_id": body.get("decision_id"),
                "status": body.get("status"),
                "type": body.get("type"),
                "summary": body.get("summary"),
            }))
            return 0
    except urllib.error.HTTPError as exc:
        safe = exc.read().decode("utf-8", errors="replace")[:500]
        print(f"Terabithia rejected Dear Diary entry ({exc.code}): {safe}", file=sys.stderr)
        return 1
    except Exception as exc:
        print(f"Dear Diary write failed: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
