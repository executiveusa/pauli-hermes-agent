#!/usr/bin/env python3
from __future__ import annotations

import hashlib
import json
import subprocess
import sys
import tempfile
from datetime import datetime, timezone
from pathlib import Path

SKILL_ROOT = Path(__file__).resolve().parents[1]
STATE_SCRIPT = SKILL_ROOT / "scripts" / "mission_state.py"
EXPORT_SCRIPT = SKILL_ROOT / "scripts" / "export_mission.py"
EXAMPLE = SKILL_ROOT / "examples" / "mission.example.json"


def run(*args: str, expect: int = 0) -> subprocess.CompletedProcess[str]:
    completed = subprocess.run([sys.executable, *args], text=True, capture_output=True)
    if completed.returncode != expect:
        raise AssertionError(
            f"Command failed ({completed.returncode}, expected {expect}): {args}\n"
            f"stdout:\n{completed.stdout}\nstderr:\n{completed.stderr}"
        )
    return completed


def write_json(path: Path, value: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")


def result_for(mission_id: str, task_id: str, attempt: int) -> dict:
    now = datetime.now(timezone.utc).isoformat()
    payload = {
        "schema_version": "1.0.0",
        "mission_id": mission_id,
        "task_id": task_id,
        "attempt": attempt,
        "worker_id": f"smoke-{task_id}",
        "status": "completed",
        "started_at": now,
        "completed_at": now,
        "summary": f"Completed {task_id} in the smoke test.",
        "findings": [
            {
                "id": f"{task_id}-finding-1",
                "statement": "The assigned task produced a deterministic verified result.",
                "type": "fact",
                "confidence": 1.0,
                "source_refs": ["smoke-test-fixture"],
                "tags": ["verification"],
            }
        ],
        "artifacts": [],
        "evidence": [
            {
                "id": f"{task_id}-evidence-1",
                "source": "smoke-test-fixture",
                "location": task_id,
                "captured_at": now,
                "excerpt_or_summary": "Synthetic evidence used only for harness verification.",
                "primary_source": True,
            }
        ],
        "files_touched": [],
        "commands_run": [],
        "validations": [{"name": "smoke-validation", "status": "pass", "evidence": "fixture"}],
        "risks": [],
        "unresolved": [],
        "suggested_next_tasks": [],
        "side_effects": [
            {
                "description": "No external side effect",
                "status": "none",
                "idempotency_key": f"{mission_id}:{task_id}:{attempt}:none",
                "rollback": "No rollback required",
            }
        ],
        "usage": {
            "provider": "test",
            "model": "fixture",
            "input_tokens": 0,
            "output_tokens": 0,
            "estimated_cost_usd": 0,
        },
        "idempotency_key": f"{mission_id}:{task_id}:{attempt}:smoke",
        "content_hash": "",
    }
    hash_input = json.dumps({**payload, "content_hash": ""}, sort_keys=True).encode("utf-8")
    payload["content_hash"] = hashlib.sha256(hash_input).hexdigest()
    return payload


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="hermes-longrun-smoke-") as temporary:
        run_root = Path(temporary)
        mission = json.loads(EXAMPLE.read_text(encoding="utf-8"))
        mission_id = mission["mission_id"]
        run(str(STATE_SCRIPT), "init", "--mission-file", str(EXAMPLE), "--run-root", str(run_root))
        run_dir = run_root / mission_id

        run(str(STATE_SCRIPT), "lock", "--run", str(run_dir), "--owner", "smoke-controller", "--ttl-seconds", "60")
        second_lock = run(
            str(STATE_SCRIPT),
            "lock",
            "--run",
            str(run_dir),
            "--owner",
            "competing-controller",
            "--ttl-seconds",
            "60",
            expect=2,
        )
        assert '"acquired": false' in second_lock.stdout.lower()

        run(str(STATE_SCRIPT), "checkpoint", "--run", str(run_dir), "--note", "Initial smoke checkpoint")
        run(str(STATE_SCRIPT), "unlock", "--run", str(run_dir), "--owner", "smoke-controller")

        for task_id in ("lesson-01", "lesson-02"):
            write_json(run_dir / "attempts" / task_id / "attempt-001" / "result.json", result_for(mission_id, task_id, 1))
        reconcile = run(str(STATE_SCRIPT), "reconcile", "--run", str(run_dir))
        state = json.loads(reconcile.stdout)
        assert state["tasks"]["lesson-01"]["status"] == "completed"
        assert state["tasks"]["lesson-02"]["status"] == "completed"
        assert "module-01-synthesis" in state["ready_tasks"]

        write_json(
            run_dir / "attempts" / "module-01-synthesis" / "attempt-001" / "result.json",
            result_for(mission_id, "module-01-synthesis", 1),
        )
        reconcile = run(str(STATE_SCRIPT), "reconcile", "--run", str(run_dir))
        state = json.loads(reconcile.stdout)
        assert state["status"] == "reviewing"
        assert all(task["status"] == "completed" for task in state["tasks"].values())

        write_json(
            run_dir / "outputs" / "synthesis.json",
            {"summary": "Smoke-test synthesis completed.", "risks": [], "unresolved": []},
        )
        write_json(
            run_dir / "outputs" / "completion-criteria.json",
            {
                "criteria": [
                    {
                        "id": item["id"],
                        "description": item["description"],
                        "status": "pass",
                        "proof": ["smoke-test"],
                    }
                    for item in mission["completion_criteria"]
                ]
            },
        )
        review = {
            "reviewer": "independent-smoke-reviewer",
            "reviewed_at": datetime.now(timezone.utc).isoformat(),
            "verdict": "pass",
            "findings": [],
            "proof": ["smoke-test"],
        }
        write_json(run_dir / "reviews" / "spec-review.json", review)
        write_json(run_dir / "reviews" / "quality-review.json", review)
        write_json(run_dir / "reviews" / "security-review.json", review)

        run(str(STATE_SCRIPT), "finalize", "--run", str(run_dir))
        exported = run(str(EXPORT_SCRIPT), "--run", str(run_dir))
        paths = json.loads(exported.stdout)
        for key in ("final_report", "export_manifest", "zip"):
            assert Path(paths[key]).is_file(), f"Missing export: {key}"
        final_report = json.loads(Path(paths["final_report"]).read_text(encoding="utf-8"))
        assert final_report["status"] == "completed"
        assert len(final_report["tasks"]) == len(mission["tasks"])
        assert final_report["provenance"]["mission_sha256"]
        assert final_report["provenance"]["export_manifest_sha256"]

    print("hardened-longrun-subagent-harness smoke test: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
