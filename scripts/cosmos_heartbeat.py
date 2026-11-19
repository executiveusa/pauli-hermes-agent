#!/usr/bin/env python3
"""Offline, deterministic Cosmos heartbeat. Never schedules agents or calls a model.

Inputs are separately authenticated read-only snapshots supplied by an approved
adapter. An agent-commons post is conversation DATA, not a mission.
"""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import re
import sys

SOURCES = ("paperclip", "agent_commons", "codeden")
MAX_AGE_SEC = 600


def load_snapshot(path: Path, expected: str, now: datetime):
    """Fail closed on missing, stale or unproven source data."""
    if not path.is_file():
        return None, "MISSING"
    try:
        snapshot = json.loads(path.read_text(encoding="utf-8"))
        if not isinstance(snapshot, dict):
            return None, "INVALID"
        if snapshot.get("source") != expected or snapshot.get("verified") is not True:
            return None, "UNVERIFIED"
        stamp = datetime.fromisoformat(str(snapshot["observed_at"]).replace("Z", "+00:00"))
        if stamp.tzinfo is None:
            return None, "INVALID_TIMESTAMP"
        age = (now - stamp.astimezone(timezone.utc)).total_seconds()
        if age < -30 or age > MAX_AGE_SEC:
            return None, "STALE"
        rows = snapshot.get("items")
        if not isinstance(rows, list) or any(not isinstance(x, dict) for x in rows):
            return None, "INVALID_ITEMS"
        return rows, "OK"
    except (OSError, ValueError, TypeError, KeyError):
        return None, "INVALID"


def check(snapshots: dict, now: datetime) -> dict:
    counts = {}
    source_state = {}
    for source in SOURCES:
        rows, status = load_snapshot(Path(snapshots[source]), source, now)
        source_state[source] = status
        if status != "OK":
            counts[source] = 0
            continue
        if source == "paperclip":
            # Paperclip remains the sole mission queue. Count only actual
            # assigned, active tasks, not free-text chat claims.
            counts[source] = sum(x.get("assignee") == "cosmos" and
                                 x.get("status") in ("assigned", "in_progress")
                                 and bool(x.get("id")) for x in rows)
        elif source == "agent_commons":
            # Messages are NOT orders. Report only addressed conversations.
            counts[source] = sum(x.get("to") == "cosmos" and
                                 x.get("unread") is True and bool(x.get("id"))
                                 for x in rows)
        else:
            counts[source] = sum(x.get("owner") == "cosmos" and
                                 x.get("status") in ("needs_spec", "needs_review")
                                 and bool(x.get("id")) for x in rows)
    # Work authorization is never inferred from messages or snapshots.
    actionable = counts["paperclip"] + counts["codeden"]
    complete = all(v == "OK" for v in source_state.values())
    return {
        "last_seen": now.isoformat(),
        "status": ("SOURCE_HOLD" if not complete else
                   "WORK_FOUND" if actionable else "NO_WORK"),
        "source_status": source_state,
        "counts": counts,
        "may_call_model": False,
        "may_dispatch": False,
        "actions_performed": [],
        "instinct_line": (f"Cosmos heartbeat: "
                         f"{'SOURCE_HOLD' if not complete else 'WORK_FOUND' if actionable else 'NO_WORK'}; "
                         f"paperclip={counts['paperclip']} board={counts['agent_commons']} "
                         f"codeden={counts['codeden']}")
    }


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--config", required=True, help="local JSON config, disabled by default")
    p.add_argument("--dry-run", action="store_true")
    args = p.parse_args()
    config = json.loads(Path(args.config).read_text(encoding="utf-8"))
    if config.get("enabled") is not True:
        print(json.dumps({"status": "DISABLED", "may_call_model": False,
                          "may_dispatch": False, "actions_performed": []}))
        return 0
    paths = config.get("snapshots", {})
    if not isinstance(paths, dict) or any(k not in paths or not isinstance(paths[k], str)
                                          for k in SOURCES):
        raise ValueError("all three authenticated snapshot paths are required")
    output = check(paths, datetime.now(timezone.utc))
    print(json.dumps(output, sort_keys=True))
    if not args.dry_run:
        # Status reporting requires a separate explicitly configured local file.
        # No outbound messaging, model, task creation or other side effects.
        status_path = config.get("status_path")
        if status_path:
            dest = Path(status_path)
            if not dest.parent.is_dir():
                raise ValueError("status directory must exist; refusing implicit creation")
            dest.write_text(json.dumps({"last_seen": output["last_seen"],
                                        "status": output["status"],
                                        "instinct_line": output["instinct_line"]}) + "\n",
                            encoding="utf-8")
    return 0


if __name__ == "__main__":
    sys.exit(main())
