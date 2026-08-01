#!/usr/bin/env python3
"""Enrich Local Footage Studio clips with a verified QVAC multimodal model.

This script is intentionally dependency-light and non-destructive:
- reads indexed frame JPEGs and clip metadata;
- calls a localhost OpenAI-compatible chat/completions endpoint;
- requires strict JSON evidence separating observations from inferences;
- updates only the derived SQLite index and description files;
- never changes source footage.
"""

from __future__ import annotations

import argparse
import base64
import json
import sqlite3
import sys
import urllib.error
import urllib.request
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


def utc_now() -> str:
    return datetime.now(timezone.utc).isoformat()


def connect(workspace: Path) -> sqlite3.Connection:
    db = sqlite3.connect(workspace / "footage.db")
    db.row_factory = sqlite3.Row
    return db


def load_config(workspace: Path) -> dict[str, Any]:
    path = workspace / "config.json"
    if not path.exists():
        raise RuntimeError(f"Missing workspace config: {path}")
    data = json.loads(path.read_text(encoding="utf-8"))
    base_url = str(data.get("qvac_base_url") or "").rstrip("/")
    model = data.get("vision_model")
    if not base_url:
        raise RuntimeError("config.json qvac_base_url is empty")
    if not model:
        raise RuntimeError(
            "config.json vision_model is unset. First verify /v1/models and set a loaded multimodal model alias."
        )
    return {"base_url": base_url, "model": str(model)}


def http_json(url: str, payload: dict[str, Any] | None = None, timeout: int = 180) -> dict[str, Any]:
    body = None if payload is None else json.dumps(payload).encode("utf-8")
    request = urllib.request.Request(
        url,
        data=body,
        headers={"Content-Type": "application/json"},
        method="GET" if payload is None else "POST",
    )
    try:
        with urllib.request.urlopen(request, timeout=timeout) as response:
            raw = response.read().decode("utf-8")
            return json.loads(raw)
    except urllib.error.HTTPError as exc:
        detail = exc.read().decode("utf-8", errors="replace")
        raise RuntimeError(f"QVAC HTTP {exc.code}: {detail[:2000]}") from exc
    except urllib.error.URLError as exc:
        raise RuntimeError(f"Cannot reach QVAC at {url}: {exc.reason}") from exc


def verify_model(base_url: str, model: str) -> None:
    result = http_json(f"{base_url}/models", timeout=30)
    ids = {str(item.get("id")) for item in result.get("data", [])}
    if model not in ids:
        raise RuntimeError(f"Configured vision model '{model}' is not loaded. Loaded models: {sorted(ids)}")


def frame_files(frame_dir: Path, max_frames: int) -> list[Path]:
    frames = sorted(frame_dir.glob("*.jpg"))
    if not frames:
        raise RuntimeError(f"No extracted frames found in {frame_dir}")
    if len(frames) <= max_frames:
        return frames
    if max_frames == 1:
        return [frames[len(frames) // 2]]
    indices = [round(i * (len(frames) - 1) / (max_frames - 1)) for i in range(max_frames)]
    return [frames[index] for index in sorted(set(indices))]


def image_part(path: Path) -> dict[str, Any]:
    encoded = base64.b64encode(path.read_bytes()).decode("ascii")
    return {
        "type": "image_url",
        "image_url": {"url": f"data:image/jpeg;base64,{encoded}", "detail": "low"},
    }


def parse_model_json(content: str) -> dict[str, Any]:
    text = content.strip()
    if text.startswith("```"):
        lines = text.splitlines()
        if lines and lines[0].startswith("```"):
            lines = lines[1:]
        if lines and lines[-1].strip() == "```":
            lines = lines[:-1]
        text = "\n".join(lines).strip()
    try:
        result = json.loads(text)
    except json.JSONDecodeError as exc:
        raise RuntimeError(f"Vision model did not return valid JSON: {text[:1000]}") from exc
    required = {"observed", "temporal_change", "inference", "clip_type", "confidence", "search_terms"}
    missing = required.difference(result)
    if missing:
        raise RuntimeError(f"Vision result missing fields: {sorted(missing)}")
    for field in ("observed", "temporal_change", "inference", "search_terms"):
        if not isinstance(result[field], list) or not all(isinstance(item, str) for item in result[field]):
            raise RuntimeError(f"Vision result field '{field}' must be a list of strings")
    confidence = float(result["confidence"])
    if confidence < 0 or confidence > 1:
        raise RuntimeError("Vision confidence must be between 0 and 1")
    result["confidence"] = confidence
    result["clip_type"] = str(result["clip_type"])
    return result


def analyze_clip(base_url: str, model: str, row: sqlite3.Row, frames: list[Path]) -> dict[str, Any]:
    timestamps = json.loads(row["frame_timestamps"])
    prompt = (
        "Analyze these ordered frames from one video clip. They represent timestamps from beginning to end. "
        "This may be spoken footage, silent nature footage, low-audio footage, a fixed-camera clip, or a timelapse. "
        "Only report directly visible facts under observed. Describe changes across frames under temporal_change. "
        "Put uncertain interpretation only under inference. Do not identify a person unless visual evidence is decisive. "
        "Classify clip_type as one of: spoken-scene, silent-nature, timelapse, low-motion, action, interview, screen-recording, unknown. "
        "Return JSON only with this exact shape: "
        '{"observed":["..."],"temporal_change":["..."],"inference":["..."],'
        '"clip_type":"unknown","confidence":0.0,"search_terms":["..."]}. '
        f"Metadata: filename={row['filename']}; duration={row['duration']} seconds; "
        f"has_audio_track={bool(row['has_audio'])}; sampled_timestamps={timestamps}."
    )
    content: list[dict[str, Any]] = [{"type": "text", "text": prompt}]
    for frame in frames:
        content.append(image_part(frame))
    payload = {
        "model": model,
        "messages": [{"role": "user", "content": content}],
        "temperature": 0.1,
        "max_tokens": 900,
    }
    response = http_json(f"{base_url}/chat/completions", payload=payload)
    choices = response.get("choices") or []
    if not choices:
        raise RuntimeError(f"QVAC returned no choices: {json.dumps(response)[:1000]}")
    message_content = choices[0].get("message", {}).get("content")
    if not isinstance(message_content, str):
        raise RuntimeError("QVAC response did not contain text message content")
    return parse_model_json(message_content)


def update_clip(workspace: Path, db: sqlite3.Connection, row: sqlite3.Row, result: dict[str, Any], model: str) -> Path:
    observed_lines = list(result["observed"]) + [f"temporal-change: {item}" for item in result["temporal_change"]]
    observed_lines.append(f"clip-type: {result['clip_type']}")
    observed_lines.extend(f"search-term: {item}" for item in result["search_terms"])
    inference_lines = list(result["inference"]) + [f"vision-confidence: {result['confidence']:.3f}"]
    observed_text = "\n".join(observed_lines)
    inference_text = "\n".join(inference_lines)
    db.execute(
        "UPDATE clips SET vision_status = ?, observed_text = ?, inference_text = ? WHERE clip_id = ?",
        ("verified-local", observed_text, inference_text, row["clip_id"]),
    )
    document = {
        "schema_version": 1,
        "clip_id": row["clip_id"],
        "source_path": row["source_path"],
        "model": model,
        "analyzed_at": utc_now(),
        "source_mutated": False,
        **result,
    }
    output = workspace / "descriptions" / f"{row['clip_id']}.json"
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(document, indent=2), encoding="utf-8")
    return output


def enrich(workspace: Path, clip_id: str | None, limit: int, max_frames: int) -> None:
    config = load_config(workspace)
    verify_model(config["base_url"], config["model"])
    with connect(workspace) as db:
        if clip_id:
            rows = db.execute("SELECT * FROM clips WHERE clip_id = ?", (clip_id,)).fetchall()
        else:
            rows = db.execute(
                "SELECT * FROM clips WHERE vision_status != 'verified-local' ORDER BY indexed_at LIMIT ?", (limit,)
            ).fetchall()
        if not rows:
            raise RuntimeError("No matching clips require vision enrichment")
        outputs = []
        for row in rows:
            frames = frame_files(Path(row["frame_dir"]), max_frames)
            result = analyze_clip(config["base_url"], config["model"], row, frames)
            output = update_clip(workspace, db, row, result, config["model"])
            db.commit()
            outputs.append({
                "clip_id": row["clip_id"],
                "description": str(output.resolve()),
                "clip_type": result["clip_type"],
                "confidence": result["confidence"],
                "frames_used": len(frames),
            })
    print(json.dumps({
        "status": "vision-enriched",
        "provider": "qvac-local",
        "model": config["model"],
        "clips": outputs,
        "source_files_modified": False,
    }, indent=2))


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Enrich indexed footage with a local QVAC vision model")
    parser.add_argument("--workspace", required=True, type=Path)
    parser.add_argument("--clip-id")
    parser.add_argument("--limit", type=int, default=1)
    parser.add_argument("--max-frames", type=int, default=8)
    return parser


def main() -> int:
    args = build_parser().parse_args()
    try:
        if args.limit < 1 or args.limit > 100:
            raise RuntimeError("--limit must be between 1 and 100")
        if args.max_frames < 2 or args.max_frames > 12:
            raise RuntimeError("--max-frames must be between 2 and 12")
        enrich(args.workspace, args.clip_id, args.limit, args.max_frames)
        return 0
    except Exception as exc:  # noqa: BLE001
        print(json.dumps({"status": "error", "error": str(exc)}, indent=2), file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
