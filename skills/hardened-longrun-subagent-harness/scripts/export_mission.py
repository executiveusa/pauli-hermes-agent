#!/usr/bin/env python3
"""Create the canonical final-report.json, export manifest, and ZIP bundle."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import tempfile
import zipfile
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

SCHEMA_VERSION = "1.0.0"


def utcnow() -> str:
    return datetime.now(timezone.utc).isoformat()


def load_json(path: Path) -> dict[str, Any]:
    with path.open("r", encoding="utf-8") as handle:
        value = json.load(handle)
    if not isinstance(value, dict):
        raise ValueError(f"Expected JSON object: {path}")
    return value


def atomic_write_json(path: Path, value: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    fd, temp_name = tempfile.mkstemp(prefix=f".{path.name}.", suffix=".tmp", dir=path.parent)
    try:
        with os.fdopen(fd, "w", encoding="utf-8") as handle:
            json.dump(value, handle, indent=2, sort_keys=True, ensure_ascii=False)
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


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for block in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def relative_files(run_dir: Path) -> list[Path]:
    excluded_names = {"export-manifest.json"}
    files: list[Path] = []
    for path in run_dir.rglob("*"):
        if not path.is_file():
            continue
        if path.name in excluded_names or path.suffix == ".zip" or ".tmp" in path.name:
            continue
        files.append(path)
    return sorted(files, key=lambda item: item.relative_to(run_dir).as_posix())


def optional_json(path: Path, default: Any) -> Any:
    return load_json(path) if path.exists() else default


def require_json(path: Path) -> dict[str, Any]:
    if not path.exists():
        raise FileNotFoundError(f"Required finalization artifact missing: {path}")
    return load_json(path)


def aggregate_usage(results: list[dict[str, Any]], state: dict[str, Any]) -> dict[str, Any]:
    input_tokens = 0
    output_tokens = 0
    estimated_cost = 0.0
    for result in results:
        usage = result.get("usage") or {}
        input_tokens += int(usage.get("input_tokens", 0) or 0)
        output_tokens += int(usage.get("output_tokens", 0) or 0)
        estimated_cost += float(usage.get("estimated_cost_usd", 0) or 0)
    return {
        "epochs": int(state.get("epoch", 0)),
        "total_attempts": sum(int(item.get("attempts", 0)) for item in state["tasks"].values()),
        "input_tokens": input_tokens,
        "output_tokens": output_tokens,
        "estimated_cost_usd": round(estimated_cost, 6),
        "runtime_minutes": float(state.get("runtime_minutes", 0) or 0),
    }


def build_report(run_dir: Path) -> tuple[dict[str, Any], dict[str, Any]]:
    mission = load_json(run_dir / "mission.json")
    state = load_json(run_dir / "state.json")
    if state.get("status") != "completed":
        raise RuntimeError(f"Mission must be finalized before export; current status: {state.get('status')}")

    synthesis = require_json(run_dir / "outputs" / "synthesis.json")
    completion = require_json(run_dir / "outputs" / "completion-criteria.json")
    if not isinstance(completion.get("criteria"), list):
        raise ValueError("outputs/completion-criteria.json must contain a criteria array")

    task_rows: list[dict[str, Any]] = []
    results: list[dict[str, Any]] = []
    evidence_index: list[dict[str, Any]] = []
    artifacts: list[dict[str, Any]] = []
    risks: list[str] = []
    unresolved: list[str] = list(state.get("unresolved", []))

    for task_id, task_state in sorted(state["tasks"].items()):
        result_path_value = task_state.get("result_path")
        result: dict[str, Any] | None = None
        if result_path_value:
            result_path = run_dir / result_path_value
            result = load_json(result_path)
            results.append(result)
            evidence_index.extend(result.get("evidence", []))
            artifacts.extend(result.get("artifacts", []))
            risks.extend(result.get("risks", []))
            unresolved.extend(result.get("unresolved", []))
        task_rows.append(
            {
                "task_id": task_id,
                "status": task_state.get("status"),
                "attempts": int(task_state.get("attempts", 0)),
                "result_path": result_path_value,
                "summary": result.get("summary", "") if result else "",
                "content_hash": task_state.get("content_hash"),
            }
        )

    reviews = {
        "spec": optional_json(run_dir / "reviews" / "spec-review.json", None),
        "quality": optional_json(run_dir / "reviews" / "quality-review.json", None),
        "security": optional_json(run_dir / "reviews" / "security-review.json", None),
    }

    rollback_doc = optional_json(
        run_dir / "outputs" / "rollback.json",
        {"steps": ["Pause the mission controller cron job.", "Restore the desired checkpoint and isolated artifacts."]},
    )
    next_doc = optional_json(run_dir / "outputs" / "next.json", {"next": "Human review of the exported mission bundle."})
    approval_doc = optional_json(
        run_dir / "outputs" / "human-approval.json",
        {
            "required": (run_dir / "candidate-memory" / "candidate-memory.json").exists(),
            "pending": ["Review candidate memory before durable insertion"]
            if (run_dir / "candidate-memory" / "candidate-memory.json").exists()
            else [],
            "approved_by": None,
            "approved_at": None,
        },
    )

    manifest_entries = []
    for path in relative_files(run_dir):
        relative = path.relative_to(run_dir).as_posix()
        if relative == "outputs/final-report.json":
            continue
        manifest_entries.append({"path": relative, "bytes": path.stat().st_size, "sha256": sha256_file(path)})
    manifest_core = {
        "schema_version": SCHEMA_VERSION,
        "mission_id": mission["mission_id"],
        "generated_at": utcnow(),
        "files": manifest_entries,
    }
    manifest_sha = sha256_bytes(json.dumps(manifest_core, sort_keys=True, separators=(",", ":")).encode("utf-8"))

    failed_criteria = [item for item in completion["criteria"] if item.get("status") == "fail"]
    conditions = any((review or {}).get("verdict") == "pass_with_conditions" for review in reviews.values() if review)
    final_status = "completed_with_conditions" if conditions else "completed"
    if failed_criteria:
        raise RuntimeError("Cannot export a completed report with failed completion criteria")

    report = {
        "schema_version": SCHEMA_VERSION,
        "mission_id": mission["mission_id"],
        "title": mission["title"],
        "objective": mission["objective"],
        "status": final_status,
        "started_at": state.get("created_at", mission["created_at"]),
        "finished_at": state.get("completed_at", utcnow()),
        "summary": synthesis.get("summary", "Mission completed; see synthesis object for details."),
        "completion_criteria": completion["criteria"],
        "tasks": task_rows,
        "synthesis": synthesis,
        "evidence_index": evidence_index,
        "artifacts": artifacts,
        "reviews": reviews,
        "usage": aggregate_usage(results, state),
        "risks": sorted(set(risks + synthesis.get("risks", []))),
        "unresolved": sorted(set(unresolved + synthesis.get("unresolved", []))),
        "rollback": rollback_doc.get("steps", []),
        "next": next_doc.get("next", "Human review."),
        "commercial_impact": mission.get("commercial_value", ""),
        "human_approval": approval_doc,
        "provenance": {
            "mission_sha256": sha256_file(run_dir / "mission.json"),
            "state_sha256": sha256_file(run_dir / "state.json"),
            "export_manifest_sha256": manifest_sha,
            "skill_version": SCHEMA_VERSION,
            "generated_by": "hardened-longrun-subagent-harness",
        },
    }
    export_manifest = {**manifest_core, "manifest_sha256": manifest_sha, "final_report": "outputs/final-report.json"}
    return report, export_manifest


def export(run_dir: Path) -> dict[str, str]:
    report, manifest = build_report(run_dir)
    outputs = run_dir / "outputs"
    outputs.mkdir(parents=True, exist_ok=True)
    report_path = outputs / "final-report.json"
    manifest_path = outputs / "export-manifest.json"
    atomic_write_json(report_path, report)
    manifest["final_report_sha256"] = sha256_file(report_path)
    atomic_write_json(manifest_path, manifest)

    mission_id = report["mission_id"]
    zip_path = outputs / f"{mission_id}.zip"
    temp_zip = outputs / f".{mission_id}.zip.tmp"
    with zipfile.ZipFile(temp_zip, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
        for path in relative_files(run_dir):
            if path == temp_zip or path == zip_path:
                continue
            archive.write(path, arcname=path.relative_to(run_dir).as_posix())
    os.replace(temp_zip, zip_path)
    return {
        "final_report": str(report_path),
        "export_manifest": str(manifest_path),
        "zip": str(zip_path),
        "zip_sha256": sha256_file(zip_path),
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="Export a completed Hermes long-running mission")
    parser.add_argument("--run", required=True)
    args = parser.parse_args()
    result = export(Path(args.run).expanduser().resolve())
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
