#!/usr/bin/env python3
"""Durable JSON state utility for the hardened Hermes long-running harness.

Uses only the Python standard library. All writes are atomic. This utility does not
call an LLM or execute mission tasks; Hermes remains the orchestrator.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import sys
import tempfile
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any

SCHEMA_VERSION = "1.0.0"
TERMINAL_STATES = {"completed", "blocked", "failed", "cancelled", "rolled_back"}
VALID_RESULT_STATUSES = {"completed", "needs_review", "blocked", "failed"}


def utcnow() -> str:
    return datetime.now(timezone.utc).isoformat()


def parse_time(value: str) -> datetime:
    return datetime.fromisoformat(value.replace("Z", "+00:00"))


def load_json(path: Path) -> dict[str, Any]:
    with path.open("r", encoding="utf-8") as handle:
        data = json.load(handle)
    if not isinstance(data, dict):
        raise ValueError(f"Expected JSON object: {path}")
    return data


def atomic_write_json(path: Path, data: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    fd, temp_name = tempfile.mkstemp(prefix=f".{path.name}.", suffix=".tmp", dir=path.parent)
    try:
        with os.fdopen(fd, "w", encoding="utf-8") as handle:
            json.dump(data, handle, indent=2, sort_keys=True, ensure_ascii=False)
            handle.write("\n")
            handle.flush()
            os.fsync(handle.fileno())
        os.replace(temp_name, path)
    except Exception:
        try:
            os.unlink(temp_name)
        except FileNotFoundError:
            pass
        raise


def append_event(run_dir: Path, event_type: str, payload: dict[str, Any]) -> None:
    event = {"at": utcnow(), "type": event_type, "payload": payload}
    events_path = run_dir / "events.jsonl"
    events_path.parent.mkdir(parents=True, exist_ok=True)
    with events_path.open("a", encoding="utf-8") as handle:
        handle.write(json.dumps(event, sort_keys=True, ensure_ascii=False) + "\n")
        handle.flush()
        os.fsync(handle.fileno())


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for block in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def validate_mission(mission: dict[str, Any]) -> None:
    required = {
        "schema_version",
        "mission_id",
        "title",
        "objective",
        "mode",
        "created_at",
        "limits",
        "completion_criteria",
        "tasks",
    }
    missing = sorted(required - mission.keys())
    if missing:
        raise ValueError(f"Mission missing required fields: {', '.join(missing)}")
    if mission["schema_version"] != SCHEMA_VERSION:
        raise ValueError(f"Unsupported schema_version: {mission['schema_version']}")
    if not isinstance(mission["tasks"], list) or not mission["tasks"]:
        raise ValueError("Mission tasks must be a non-empty array")
    task_ids = [task.get("id") for task in mission["tasks"]]
    if None in task_ids or len(set(task_ids)) != len(task_ids):
        raise ValueError("Every task needs a unique id")
    known = set(task_ids)
    for task in mission["tasks"]:
        unknown = set(task.get("dependencies", [])) - known
        if unknown:
            raise ValueError(f"Task {task['id']} has unknown dependencies: {sorted(unknown)}")
        if task["id"] in task.get("dependencies", []):
            raise ValueError(f"Task {task['id']} depends on itself")


def initial_state(mission: dict[str, Any]) -> dict[str, Any]:
    default_attempts = int(mission["limits"]["max_attempts_per_task"])
    tasks: dict[str, Any] = {}
    for task in mission["tasks"]:
        tasks[task["id"]] = {
            "status": "pending",
            "attempts": 0,
            "max_attempts": int(task.get("max_attempts", default_attempts)),
            "result_path": None,
            "content_hash": None,
            "last_error": None,
            "updated_at": utcnow(),
        }
    return {
        "schema_version": SCHEMA_VERSION,
        "mission_id": mission["mission_id"],
        "status": "ready",
        "epoch": 0,
        "created_at": utcnow(),
        "updated_at": utcnow(),
        "consecutive_no_progress_epochs": 0,
        "tasks": tasks,
        "ready_tasks": [],
        "pending_approvals": [],
        "unresolved": [],
        "last_checkpoint": None,
    }


def cmd_init(args: argparse.Namespace) -> int:
    mission_path = Path(args.mission_file).expanduser().resolve()
    mission = load_json(mission_path)
    validate_mission(mission)
    run_root = Path(args.run_root).expanduser().resolve()
    run_dir = run_root / mission["mission_id"]
    if run_dir.exists() and any(run_dir.iterdir()) and not args.force:
        raise FileExistsError(f"Run already exists: {run_dir}; use --force only for an intentional reset")
    if run_dir.exists() and args.force:
        backup = run_dir.with_name(f"{run_dir.name}.backup-{datetime.now().strftime('%Y%m%d%H%M%S')}")
        shutil.move(str(run_dir), str(backup))
    for relative in (
        "locks",
        "tasks",
        "attempts",
        "checkpoints",
        "artifacts",
        "candidate-memory",
        "reviews",
        "outputs",
    ):
        (run_dir / relative).mkdir(parents=True, exist_ok=True)
    atomic_write_json(run_dir / "mission.json", mission)
    for task in mission["tasks"]:
        atomic_write_json(run_dir / "tasks" / f"{task['id']}.json", task)
    state = initial_state(mission)
    atomic_write_json(run_dir / "state.json", state)
    append_event(run_dir, "mission_initialized", {"mission_id": mission["mission_id"]})
    print(json.dumps({"run_dir": str(run_dir), "status": state["status"]}, indent=2))
    return 0


def cmd_lock(args: argparse.Namespace) -> int:
    run_dir = Path(args.run).expanduser().resolve()
    lock_path = run_dir / "locks" / "controller.lock"
    lock_path.parent.mkdir(parents=True, exist_ok=True)
    now = datetime.now(timezone.utc)
    payload = {
        "owner": args.owner,
        "acquired_at": now.isoformat(),
        "expires_at": (now + timedelta(seconds=args.ttl_seconds)).isoformat(),
        "pid": os.getpid(),
    }
    try:
        fd = os.open(lock_path, os.O_CREAT | os.O_EXCL | os.O_WRONLY, 0o600)
    except FileExistsError:
        existing = load_json(lock_path)
        expires = parse_time(existing["expires_at"])
        if expires > now:
            print(json.dumps({"acquired": False, "lock": existing}, indent=2))
            return 2
        stale = lock_path.with_name(f"controller.lock.stale-{int(now.timestamp())}")
        os.replace(lock_path, stale)
        append_event(run_dir, "stale_lock_recovered", {"previous": existing, "stale_path": str(stale)})
        fd = os.open(lock_path, os.O_CREAT | os.O_EXCL | os.O_WRONLY, 0o600)
    with os.fdopen(fd, "w", encoding="utf-8") as handle:
        json.dump(payload, handle, indent=2, sort_keys=True)
        handle.write("\n")
        handle.flush()
        os.fsync(handle.fileno())
    append_event(run_dir, "controller_lock_acquired", payload)
    print(json.dumps({"acquired": True, "lock": payload}, indent=2))
    return 0


def cmd_unlock(args: argparse.Namespace) -> int:
    run_dir = Path(args.run).expanduser().resolve()
    lock_path = run_dir / "locks" / "controller.lock"
    if not lock_path.exists():
        print(json.dumps({"released": False, "reason": "no_lock"}, indent=2))
        return 0
    existing = load_json(lock_path)
    if existing.get("owner") != args.owner and not args.force:
        print(json.dumps({"released": False, "reason": "owner_mismatch", "lock": existing}, indent=2))
        return 2
    lock_path.unlink()
    append_event(run_dir, "controller_lock_released", {"owner": args.owner, "forced": args.force})
    print(json.dumps({"released": True}, indent=2))
    return 0


def result_is_valid(result: dict[str, Any], mission_id: str, task_id: str, attempt: int) -> tuple[bool, str | None]:
    required = {
        "schema_version",
        "mission_id",
        "task_id",
        "attempt",
        "status",
        "summary",
        "findings",
        "artifacts",
        "evidence",
        "files_touched",
        "validations",
        "risks",
        "unresolved",
        "side_effects",
        "idempotency_key",
        "content_hash",
    }
    missing = sorted(required - result.keys())
    if missing:
        return False, f"missing fields: {', '.join(missing)}"
    if result["schema_version"] != SCHEMA_VERSION:
        return False, "schema version mismatch"
    if result["mission_id"] != mission_id or result["task_id"] != task_id or int(result["attempt"]) != attempt:
        return False, "identity mismatch"
    if result["status"] not in VALID_RESULT_STATUSES:
        return False, "invalid status"
    if any(item.get("status") == "fail" for item in result.get("validations", [])) and result["status"] == "completed":
        return False, "completed result contains failed validation"
    return True, None


def compute_ready(mission: dict[str, Any], state: dict[str, Any]) -> list[str]:
    ready: list[str] = []
    for task in mission["tasks"]:
        task_state = state["tasks"][task["id"]]
        if task_state["status"] not in {"pending", "retry_scheduled", "ready"}:
            continue
        if all(state["tasks"][dep]["status"] == "completed" for dep in task.get("dependencies", [])):
            ready.append(task["id"])
    return ready


def cmd_reconcile(args: argparse.Namespace) -> int:
    run_dir = Path(args.run).expanduser().resolve()
    mission = load_json(run_dir / "mission.json")
    state = load_json(run_dir / "state.json")
    before_completed = sum(1 for item in state["tasks"].values() if item["status"] == "completed")

    for task in mission["tasks"]:
        task_id = task["id"]
        task_state = state["tasks"][task_id]
        attempt_root = run_dir / "attempts" / task_id
        if not attempt_root.exists():
            continue
        for attempt_dir in sorted(attempt_root.glob("attempt-*")):
            try:
                attempt = int(attempt_dir.name.split("-")[-1])
            except ValueError:
                continue
            result_path = attempt_dir / "result.json"
            if not result_path.exists():
                continue
            try:
                result = load_json(result_path)
                valid, error = result_is_valid(result, mission["mission_id"], task_id, attempt)
            except Exception as exc:  # malformed JSON is evidence, not a crash loop
                valid, error, result = False, str(exc), {}
            if not valid:
                task_state["last_error"] = f"Invalid result {result_path}: {error}"
                append_event(run_dir, "worker_result_rejected", {"task_id": task_id, "attempt": attempt, "reason": error})
                continue
            if attempt < task_state["attempts"] and task_state["status"] == "completed":
                continue
            task_state["attempts"] = max(task_state["attempts"], attempt)
            task_state["result_path"] = str(result_path.relative_to(run_dir))
            task_state["content_hash"] = result.get("content_hash")
            task_state["last_error"] = None
            task_state["updated_at"] = utcnow()
            if result["status"] == "completed":
                task_state["status"] = "completed"
            elif result["status"] == "needs_review":
                task_state["status"] = "needs_review"
            elif result["status"] == "blocked":
                task_state["status"] = "blocked"
            else:
                task_state["status"] = (
                    "failed" if task_state["attempts"] >= task_state["max_attempts"] else "retry_scheduled"
                )
            append_event(
                run_dir,
                "worker_result_reconciled",
                {"task_id": task_id, "attempt": attempt, "status": task_state["status"], "result_path": task_state["result_path"]},
            )

    state["ready_tasks"] = compute_ready(mission, state)
    for task_id in state["ready_tasks"]:
        if state["tasks"][task_id]["status"] in {"pending", "retry_scheduled"}:
            state["tasks"][task_id]["status"] = "ready"

    after_completed = sum(1 for item in state["tasks"].values() if item["status"] == "completed")
    if after_completed > before_completed:
        state["consecutive_no_progress_epochs"] = 0
    else:
        state["consecutive_no_progress_epochs"] = int(state.get("consecutive_no_progress_epochs", 0)) + 1
    blocked = [task_id for task_id, item in state["tasks"].items() if item["status"] in {"blocked", "failed", "unknown"}]
    if blocked:
        state["status"] = "blocked"
        state["unresolved"] = sorted(set(state.get("unresolved", []) + [f"Blocked task: {task_id}" for task_id in blocked]))
    elif all(item["status"] == "completed" for item in state["tasks"].values()):
        state["status"] = "reviewing"
    else:
        state["status"] = "ready"
    state["updated_at"] = utcnow()
    atomic_write_json(run_dir / "state.json", state)
    append_event(run_dir, "mission_reconciled", {"status": state["status"], "ready_tasks": state["ready_tasks"]})
    print(json.dumps(state, indent=2, sort_keys=True))
    return 0


def cmd_checkpoint(args: argparse.Namespace) -> int:
    run_dir = Path(args.run).expanduser().resolve()
    state = load_json(run_dir / "state.json")
    state["epoch"] = int(state.get("epoch", 0)) + 1
    state["updated_at"] = utcnow()
    checkpoint = {
        "schema_version": SCHEMA_VERSION,
        "mission_id": state["mission_id"],
        "epoch": state["epoch"],
        "created_at": utcnow(),
        "note": args.note,
        "state": state,
    }
    checkpoint_path = run_dir / "checkpoints" / f"epoch-{state['epoch']:04d}.json"
    atomic_write_json(checkpoint_path, checkpoint)
    state["last_checkpoint"] = str(checkpoint_path.relative_to(run_dir))
    atomic_write_json(run_dir / "state.json", state)
    append_event(run_dir, "checkpoint_written", {"epoch": state["epoch"], "path": state["last_checkpoint"], "note": args.note})
    print(json.dumps({"checkpoint": str(checkpoint_path), "epoch": state["epoch"]}, indent=2))
    return 0


def review_verdict(path: Path) -> str | None:
    if not path.exists():
        return None
    data = load_json(path)
    return data.get("verdict")


def cmd_finalize(args: argparse.Namespace) -> int:
    run_dir = Path(args.run).expanduser().resolve()
    mission = load_json(run_dir / "mission.json")
    state = load_json(run_dir / "state.json")
    incomplete = [task_id for task_id, item in state["tasks"].items() if item["status"] != "completed"]
    if incomplete:
        raise RuntimeError(f"Cannot finalize; incomplete tasks: {', '.join(incomplete)}")
    required_reviews = ["spec-review.json", "quality-review.json"]
    if any(task.get("side_effect_class") != "none" for task in mission["tasks"]):
        required_reviews.append("security-review.json")
    failures: list[str] = []
    for review_name in required_reviews:
        verdict = review_verdict(run_dir / "reviews" / review_name)
        if verdict not in {"pass", "pass_with_conditions"}:
            failures.append(f"{review_name}: {verdict or 'missing'}")
    if failures:
        raise RuntimeError("Cannot finalize; review gates not passed: " + "; ".join(failures))
    state["status"] = "completed"
    state["completed_at"] = utcnow()
    state["updated_at"] = utcnow()
    atomic_write_json(run_dir / "state.json", state)
    append_event(run_dir, "mission_finalized", {"status": "completed"})
    print(json.dumps({"mission_id": state["mission_id"], "status": "completed"}, indent=2))
    return 0


def cmd_status(args: argparse.Namespace) -> int:
    run_dir = Path(args.run).expanduser().resolve()
    state = load_json(run_dir / "state.json")
    print(json.dumps(state, indent=2, sort_keys=True))
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Durable mission state manager")
    sub = parser.add_subparsers(dest="command", required=True)

    init = sub.add_parser("init")
    init.add_argument("--mission-file", required=True)
    init.add_argument("--run-root", required=True)
    init.add_argument("--force", action="store_true")
    init.set_defaults(func=cmd_init)

    lock = sub.add_parser("lock")
    lock.add_argument("--run", required=True)
    lock.add_argument("--owner", required=True)
    lock.add_argument("--ttl-seconds", type=int, default=1800)
    lock.set_defaults(func=cmd_lock)

    unlock = sub.add_parser("unlock")
    unlock.add_argument("--run", required=True)
    unlock.add_argument("--owner", required=True)
    unlock.add_argument("--force", action="store_true")
    unlock.set_defaults(func=cmd_unlock)

    reconcile = sub.add_parser("reconcile")
    reconcile.add_argument("--run", required=True)
    reconcile.set_defaults(func=cmd_reconcile)

    checkpoint = sub.add_parser("checkpoint")
    checkpoint.add_argument("--run", required=True)
    checkpoint.add_argument("--note", required=True)
    checkpoint.set_defaults(func=cmd_checkpoint)

    finalize = sub.add_parser("finalize")
    finalize.add_argument("--run", required=True)
    finalize.set_defaults(func=cmd_finalize)

    status = sub.add_parser("status")
    status.add_argument("--run", required=True)
    status.set_defaults(func=cmd_status)
    return parser


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()
    try:
        return int(args.func(args))
    except Exception as exc:
        print(json.dumps({"error": type(exc).__name__, "message": str(exc)}, indent=2), file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
