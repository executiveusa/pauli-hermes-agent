#!/usr/bin/env python3
"""Rank indexed footage for story usefulness without modifying source media."""
from __future__ import annotations

import argparse
import json
import math
import sqlite3
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


def utc_now() -> str:
    return datetime.now(timezone.utc).isoformat()


def connect(workspace: Path) -> sqlite3.Connection:
    db = sqlite3.connect(workspace / "footage.db")
    db.row_factory = sqlite3.Row
    return db


def transcript_text(observed: str) -> str:
    start = observed.find("[TRANSCRIPT_START]")
    end = observed.find("[TRANSCRIPT_END]")
    if start < 0 or end < start:
        return ""
    return observed[start + len("[TRANSCRIPT_START]"):end].strip()


def score_clip(row: sqlite3.Row) -> dict[str, Any]:
    observed = str(row["observed_text"] or "")
    inference = str(row["inference_text"] or "")
    transcript = transcript_text(observed)
    duration = max(float(row["duration"] or 0), 0.0)
    has_vision = str(row["vision_status"]) == "verified-local"
    has_transcript = str(row["transcript_status"]) == "verified-local" and bool(transcript)

    evidence_score = (3.0 if has_vision else 0.0) + (2.0 if has_transcript else 0.0)
    temporal_score = min(observed.lower().count("temporal-change:"), 3) * 0.8
    specificity_score = min(len(set((observed + " " + transcript).lower().split())) / 80.0, 2.0)
    duration_score = 1.0 if 3 <= duration <= 180 else 0.4 if duration > 0 else 0.0
    uncertainty_penalty = min(inference.lower().count("unknown") + inference.lower().count("uncertain"), 2) * 0.5
    total = max(0.0, evidence_score + temporal_score + specificity_score + duration_score - uncertainty_penalty)
    normalized = round(min(total / 10.0, 1.0), 3)

    reasons = []
    if has_vision:
        reasons.append("verified visual evidence")
    if has_transcript:
        reasons.append("searchable speech evidence")
    if temporal_score:
        reasons.append("visible temporal change")
    if duration_score >= 1:
        reasons.append("editor-friendly duration")
    if not reasons:
        reasons.append("metadata only; requires enrichment")

    return {
        "clip_id": row["clip_id"],
        "source_path": row["source_path"],
        "filename": row["filename"],
        "duration": duration,
        "score": normalized,
        "reasons": reasons,
        "vision_status": row["vision_status"],
        "transcript_status": row["transcript_status"],
        "source_mutated": False,
    }


def rank(workspace: Path, limit: int, minimum: float) -> None:
    with connect(workspace) as db:
        rows = db.execute("SELECT * FROM clips ORDER BY indexed_at").fetchall()
    ranked = sorted((score_clip(row) for row in rows), key=lambda item: (-item["score"], item["filename"]))
    selected = [item for item in ranked if item["score"] >= minimum][:limit]
    report = {
        "schema_version": 1,
        "created_at": utc_now(),
        "status": "PROPOSED",
        "ranking_method": "deterministic-evidence-v1",
        "human_review_required": True,
        "source_mutation_allowed": False,
        "results": selected,
    }
    output = workspace / "manifests" / f"story-ranking-{datetime.now().strftime('%Y%m%d-%H%M%S')}.json"
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(json.dumps({"status": "story-ranking-created", "path": str(output.resolve()), **report}, indent=2))


def main() -> int:
    parser = argparse.ArgumentParser(description="Rank indexed clips for human-reviewed story selection")
    parser.add_argument("--workspace", required=True, type=Path)
    parser.add_argument("--limit", type=int, default=20)
    parser.add_argument("--minimum", type=float, default=0.0)
    args = parser.parse_args()
    try:
        if not 1 <= args.limit <= 500:
            raise RuntimeError("--limit must be between 1 and 500")
        if not 0 <= args.minimum <= 1:
            raise RuntimeError("--minimum must be between 0 and 1")
        rank(args.workspace, args.limit, args.minimum)
        return 0
    except Exception as exc:  # noqa: BLE001
        print(json.dumps({"status": "error", "error": str(exc)}, indent=2), file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
