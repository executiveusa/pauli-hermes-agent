#!/usr/bin/env python3
from __future__ import annotations
import json, subprocess, sys, tempfile
from datetime import datetime, timedelta, timezone
from pathlib import Path

SKILL_ROOT = Path(__file__).resolve().parents[1]
CTRL = SKILL_ROOT / "scripts" / "risk_controls.py"

def write(path: Path, value: dict):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")

def run(*args: str, expect: int = 0):
    p = subprocess.run([sys.executable, str(CTRL), *args], text=True, capture_output=True)
    if p.returncode != expect:
        raise AssertionError(f"cmd={args} rc={p.returncode} expected={expect}\nout={p.stdout}\nerr={p.stderr}")
    return p

def base_task(tid: str, effect="none"):
    return {
      "id": tid, "title": tid, "goal": f"Complete durable work for {tid}.",
      "context": f"Full context for {tid}.", "dependencies": [],
      "side_effect_class": effect,
      "acceptance_criteria": ["Produce valid JSON"],
      "required_evidence": ["Proof file"],
      "allowed_inputs": [f"sources/{tid}/**"],
      "allowed_outputs": [f"attempts/{tid}/**"],
      "prohibited_actions": ["Edit shared state"],
      "estimated_cost_usd": 0.25,
      "provider": "cheap-model"
    }

def make_run(root: Path, tasks, concurrency=4, provider_cap=2):
    run_dir = root / "mission-1"
    for directory in ["attempts", "tasks", "events", "approvals", "outputs", "reviews", "locks"]:
        (run_dir / directory).mkdir(parents=True, exist_ok=True)
    mission = {
      "schema_version": "1.0.0", "mission_id": "mission-1", "title": "Risk test mission",
      "objective": "Prove durable recovery and safe bounded delegation.", "mode": "research",
      "created_at": datetime.now(timezone.utc).isoformat(), "constraints": ["No unsafe retries"],
      "limits": {"max_epochs": 10, "max_attempts_per_task": 3, "max_concurrent_children": concurrency,
                 "provider_concurrency_cap": provider_cap, "max_iterations_per_child": 10,
                 "max_total_cost_usd": 10, "max_dispatch_cost_usd": 2},
      "completion_criteria": [{"id": "C1", "description": "done", "proof": "state"}], "tasks": tasks
    }
    state = {
      "schema_version": "1.0.0", "mission_id": "mission-1", "status": "ready", "epoch": 0,
      "created_at": datetime.now(timezone.utc).isoformat(), "updated_at": datetime.now(timezone.utc).isoformat(),
      "consecutive_no_progress_epochs": 0,
      "tasks": {task["id"]: {"status": "pending", "attempts": 0, "max_attempts": 3, "result_path": None,
                         "content_hash": None, "last_error": None, "updated_at": datetime.now(timezone.utc).isoformat()} for task in tasks},
      "ready_tasks": [], "pending_approvals": [], "unresolved": [], "last_checkpoint": None
    }
    write(run_dir / "mission.json", mission); write(run_dir / "state.json", state)
    for task in tasks:
        write(run_dir / "tasks" / f"{task['id']}.json", task)
    return run_dir

def expire(run_dir: Path, task_id: str, attempt=1):
    path = run_dir / "attempts" / task_id / f"attempt-{attempt:03d}" / "attempt-state.json"
    state = json.loads(path.read_text())
    state["lease_expires_at"] = (datetime.now(timezone.utc) - timedelta(seconds=5)).isoformat()
    write(path, state)

def checkpoint(run_dir: Path, task_id: str, attempt=1, consistent=True):
    path = run_dir / "cp.json"
    write(path, {"schema_version": "1.0.0", "mission_id": "mission-1", "task_id": task_id, "attempt": attempt,
              "completed_units": ["unit-1"], "next_unit": "unit-2", "artifacts": [], "side_effects": [],
              "local_state_consistent": consistent})
    run("heartbeat", "--run", str(run_dir), "--task-id", task_id, "--attempt", str(attempt),
        "--worker-id", "worker-1", "--worker-ttl-seconds", "1", "--checkpoint-file", str(path))

def test_concurrency_and_context():
    with tempfile.TemporaryDirectory() as temporary:
        root = Path(temporary); tasks = [base_task(f"t{i}") for i in range(1, 5)]
        run_dir = make_run(root, tasks, concurrency=4, provider_cap=2)
        output = json.loads(run("prepare-batch", "--run", str(run_dir), "--controller", "c1").stdout)
        assert len(output["dispatch"]) == 2, output
        for item in output["dispatch"]:
            packet = json.loads(Path(item["request_path"]).read_text())
            assert Path(packet["result_path"]).is_absolute()
            assert packet["context"] and packet["acceptance_criteria"] and packet["required_evidence"]
        second_root = root / "invalid"
        second_root.mkdir()
        invalid = make_run(second_root, [{**base_task("bad"), "context": ""}], concurrency=2, provider_cap=2)
        blocked = json.loads(run("prepare-batch", "--run", str(invalid), "--controller", "c2", expect=4).stdout)
        assert blocked["reason"] == "invalid_context_packet"

def test_checkpoint_recovery():
    with tempfile.TemporaryDirectory() as temporary:
        run_dir = make_run(Path(temporary), [base_task("read-only")], concurrency=1, provider_cap=1)
        run("prepare-batch", "--run", str(run_dir), "--controller", "c")
        checkpoint(run_dir, "read-only"); expire(run_dir, "read-only")
        recovered = json.loads(run("recover", "--run", str(run_dir)).stdout)
        assert recovered["status"] == "ready", recovered
        state = json.loads((run_dir / "state.json").read_text())
        assert state["tasks"]["read-only"]["status"] == "retry_scheduled"
        resume = state["tasks"]["read-only"]["resume_from"]
        assert resume and (run_dir / resume).exists()
        dispatch = json.loads(run("prepare-batch", "--run", str(run_dir), "--controller", "c2").stdout)["dispatch"][0]
        assert dispatch["resume_from"] and Path(dispatch["resume_from"]).exists()

def test_external_side_effect_reconciliation():
    with tempfile.TemporaryDirectory() as temporary:
        run_dir = make_run(Path(temporary), [base_task("publish", "approval_required")], concurrency=1, provider_cap=1)
        write(run_dir / "approvals" / "publish.json", {"mission_id": "mission-1", "task_id": "publish", "approved": True,
              "approved_by": "human", "expires_at": (datetime.now(timezone.utc) + timedelta(hours=1)).isoformat()})
        run("prepare-batch", "--run", str(run_dir), "--controller", "c")
        expire(run_dir, "publish")
        recovered = json.loads(run("recover", "--run", str(run_dir)).stdout)
        assert recovered["status"] == "blocked"
        state = json.loads((run_dir / "state.json").read_text())
        assert state["tasks"]["publish"]["status"] == "needs_reconciliation"
        run("prepare-batch", "--run", str(run_dir), "--controller", "c2", expect=3)
        attempt_state = json.loads((run_dir / "attempts" / "publish" / "attempt-001" / "attempt-state.json").read_text())
        receipt = run_dir / "receipt.json"
        write(receipt, {"schema_version": "1.0.0", "mission_id": "mission-1", "task_id": "publish", "attempt": 1,
              "idempotency_key": attempt_state["idempotency_key"], "outcome": "not_applied",
              "verified_at": datetime.now(timezone.utc).isoformat(), "verified_by": "human", "evidence": ["provider lookup"]})
        run("reconcile-side-effect", "--run", str(run_dir), "--task-id", "publish", "--attempt", "1", "--receipt-file", str(receipt))
        state = json.loads((run_dir / "state.json").read_text())
        assert state["tasks"]["publish"]["status"] == "retry_scheduled"
        output = json.loads(run("prepare-batch", "--run", str(run_dir), "--controller", "c3").stdout)
        assert len(output["dispatch"]) == 1 and output["dispatch"][0]["attempt"] == 2

def test_rate_limit_backpressure():
    with tempfile.TemporaryDirectory() as temporary:
        run_dir = make_run(Path(temporary), [base_task(f"t{i}") for i in range(4)], concurrency=4, provider_cap=4)
        rate = json.loads(run("record-rate-limit", "--run", str(run_dir), "--provider", "cheap-model", "--retry-after-seconds", "60").stdout)
        assert rate["concurrency_limit"] == 2 and rate["adaptive_concurrency"] == 2
        output = json.loads(run("prepare-batch", "--run", str(run_dir), "--controller", "c").stdout)
        assert len(output["dispatch"]) == 0
        state = json.loads((run_dir / "state.json").read_text())
        state["providers"]["cheap-model"]["cooldown_until"] = (datetime.now(timezone.utc) - timedelta(seconds=1)).isoformat()
        write(run_dir / "state.json", state)
        output = json.loads(run("prepare-batch", "--run", str(run_dir), "--controller", "c2").stdout)
        assert len(output["dispatch"]) <= 2

def test_hard_cap():
    with tempfile.TemporaryDirectory() as temporary:
        run_dir = make_run(Path(temporary), [base_task("t")], concurrency=99, provider_cap=99)
        error = run("prepare-batch", "--run", str(run_dir), "--controller", "c", expect=1)
        assert "between 1 and 8" in error.stderr

def main():
    test_concurrency_and_context()
    test_checkpoint_recovery()
    test_external_side_effect_reconciliation()
    test_rate_limit_backpressure()
    test_hard_cap()
    print("hardened longrun risk controls: PASS")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
