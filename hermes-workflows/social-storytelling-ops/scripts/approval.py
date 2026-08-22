#!/usr/bin/env python3
"""Approval-loop helper for hermes-workflows/social-storytelling-ops.

Pure stdlib, no gateway/session dependency, so it can be called from:
  - cron/daily-content-digest.json (build the digest message)
  - a gateway slash-command handler (/content-approve, /content-reject)
  - a test harness, directly

Operates on runs/<run_id>/manifest.json, matching
schemas/reel-manifest.json. Does not talk to Postiz or the gateway itself —
callers own delivery/side effects; this module only owns manifest state and
the rejection-reason -> lessons.md write (see Part 6: the learning loop).
"""

from __future__ import annotations

import argparse
import json
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

DIGEST_STATUSES = ("NEEDS-REVIEW", "PENDING-APPROVAL")
APPROVABLE_STATUSES = ("NEEDS-REVIEW", "PENDING-APPROVAL")


def _manifest_path(run_dir: Path) -> Path:
    return run_dir / "manifest.json"


def load_manifest(run_dir: Path) -> dict[str, Any]:
    path = _manifest_path(run_dir)
    if not path.exists():
        raise FileNotFoundError(f"no manifest at {path}")
    return json.loads(path.read_text(encoding="utf-8"))


def save_manifest(run_dir: Path, manifest: dict[str, Any]) -> None:
    _manifest_path(run_dir).write_text(
        json.dumps(manifest, indent=2, sort_keys=False) + "\n", encoding="utf-8"
    )


def _find_reel(manifest: dict[str, Any], reel_id: str) -> dict[str, Any]:
    for reel in manifest.get("reels", []):
        if reel.get("reel_id") == reel_id:
            return reel
    raise KeyError(f"no reel '{reel_id}' in manifest")


def digest_queue(manifest: dict[str, Any]) -> list[dict[str, Any]]:
    """Reels awaiting the daily human digest — NEEDS-REVIEW or PENDING-APPROVAL."""
    return [r for r in manifest.get("reels", []) if r.get("status") in DIGEST_STATUSES]


def format_digest(manifest: dict[str, Any]) -> str:
    """One summary line per queued draft: platform, hook, series role, link."""
    queue = digest_queue(manifest)
    if not queue:
        return f"No drafts awaiting review/approval in run {manifest.get('run_id', '?')}."
    lines = [f"Daily content digest — run {manifest.get('run_id', '?')} ({manifest.get('client', '?')})", ""]
    for reel in queue:
        lines.append(
            f"- [{reel.get('status')}] {reel.get('platform', '?')} / "
            f"{reel.get('series_role', '?')} — \"{reel.get('hook', '(no hook)')}\" "
            f"({reel.get('reel_id')}) -> {reel.get('draft_path', '(no path)')}"
        )
    lines.append("")
    lines.append("Reply /content-approve <reel_id> or /content-reject <reel_id> <one-line reason>.")
    return "\n".join(lines)


def approve(run_dir: Path, reel_id: str) -> dict[str, Any]:
    manifest = load_manifest(run_dir)
    reel = _find_reel(manifest, reel_id)
    if reel.get("status") not in APPROVABLE_STATUSES:
        raise ValueError(
            f"reel '{reel_id}' has status '{reel.get('status')}', "
            f"expected one of {APPROVABLE_STATUSES}"
        )
    reel["status"] = "APPROVED"
    save_manifest(run_dir, manifest)
    return reel


def _next_revision_id(manifest: dict[str, Any], base_reel_id: str) -> str:
    existing = {r.get("reel_id") for r in manifest.get("reels", [])}
    n = 1
    while f"{base_reel_id}-rev{n}" in existing:
        n += 1
    return f"{base_reel_id}-rev{n}"


def reject(run_dir: Path, reel_id: str, reason: str, lessons_path: Path | None = None) -> dict[str, Any]:
    """Reject a reel with a one-line reason.

    Creates a bounded revision task (a new DRAFT-status manifest entry
    pointing back at the rejected one via revision_of) rather than
    restarting the pipeline, per AGENTS.md's retry policy: "Editorial
    review failure: create a bounded revision task; do not restart the
    whole pipeline."

    Appends the reason to lessons.md (Part 6) so it durably improves
    future drafts rather than only fixing this one.
    """
    if not reason or not reason.strip():
        raise ValueError("rejection requires a one-line reason")

    manifest = load_manifest(run_dir)
    original = _find_reel(manifest, reel_id)
    original["status"] = "REJECTED"
    original["rejection_reason"] = reason.strip()

    revision_id = _next_revision_id(manifest, reel_id)
    revision = {
        "reel_id": revision_id,
        "platform": original.get("platform"),
        "hook": original.get("hook"),
        "series_role": original.get("series_role"),
        "draft_path": None,
        "status": "DRAFT",
        "rejection_reason": None,
        "revision_of": reel_id,
    }
    manifest.setdefault("reels", []).append(revision)
    save_manifest(run_dir, manifest)

    if lessons_path is not None:
        _append_lesson(lessons_path, manifest.get("run_id", "?"), reel_id, reason.strip())

    return revision


def _append_lesson(lessons_path: Path, run_id: str, reel_id: str, reason: str) -> None:
    lessons_path.parent.mkdir(parents=True, exist_ok=True)
    timestamp = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    entry = f"- [{timestamp}] run={run_id} reel={reel_id}: {reason}\n"
    with lessons_path.open("a", encoding="utf-8") as f:
        if lessons_path.stat().st_size == 0:
            f.write("# Rejection Lessons\n\n")
            f.write(
                "Durable rejection reasons, read by taste-reviewer and "
                "story-miner before their next run (see Part 6 in the "
                "workflow build notes). Append-only.\n\n"
            )
        f.write(entry)


def _cli() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="cmd", required=True)

    p_digest = sub.add_parser("digest", help="print the daily digest message")
    p_digest.add_argument("run_dir", type=Path)

    p_approve = sub.add_parser("approve", help="approve one reel")
    p_approve.add_argument("run_dir", type=Path)
    p_approve.add_argument("reel_id")

    p_reject = sub.add_parser("reject", help="reject one reel with a reason")
    p_reject.add_argument("run_dir", type=Path)
    p_reject.add_argument("reel_id")
    p_reject.add_argument("reason")
    p_reject.add_argument("--lessons-path", type=Path, default=None)

    args = parser.parse_args()

    if args.cmd == "digest":
        print(format_digest(load_manifest(args.run_dir)))
    elif args.cmd == "approve":
        reel = approve(args.run_dir, args.reel_id)
        print(f"APPROVED {reel['reel_id']}")
    elif args.cmd == "reject":
        revision = reject(args.run_dir, args.reel_id, args.reason, args.lessons_path)
        print(f"REJECTED {args.reel_id} -> created revision task {revision['reel_id']}")

    return 0


if __name__ == "__main__":
    sys.exit(_cli())
