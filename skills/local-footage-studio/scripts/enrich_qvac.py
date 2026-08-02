#!/usr/bin/env python3
"""Enrich indexed footage using a verified local QVAC multimodal model.

The worker reads derived JPEG frames, calls a localhost OpenAI-compatible
endpoint, validates structured evidence, and updates only derived workspace
artifacts. Original media is never opened for writing.
"""

from __future__ import annotations

import argparse
import base64
import json
import re
import sqlite3
import sys
import urllib.error
import urllib.request
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

FRAME_TIMESTAMP_RE = re.compile(r"-t(?P<seconds>\d+(?:\.\d+)?)\.jpg$", re.IGNORECASE)
ALLOWED_CLIP_TYPES = {
    "spoken-scene",
    "silent-nature",
    "timelapse",
    "low-motion",
    "action",
    "interview",
    "screen-recording",
    "unknown",
}


def utc_now() -> str:
    return datetime.now(timezone.utc).isoformat()


def connect(workspace: Path) -> sqlite3.Connection:
    db_path = workspace / "footage.db"
    if not db_path.exists():
        raise RuntimeError(f"Missing footage database: {db_path}")
    db = sqlite3.connect(db_path)
    db.row_factory = sqlite3.Row
    return db


def load_config(workspace: Path) -> dict[str, str]:
    path = workspace / "config.json"
    if not path.exists():
        raise RuntimeError(f"Missing workspace config: {path}")
    data = json.loads(path.read_text(encoding="utf-8"))
    base_url = str(data.get("qvac_base_url") or "").rstrip("/")
    model = str(data.get("vision_model") or "").strip()
    if not base_url:
        raise RuntimeError("config.json qvac_base_url is empty")
    if not model:
        raise RuntimeError(
            "config.json vision_model is unset. Verify /v1/models and set an exact loaded multimodal alias."
        )
    return {"base_url": base_url, "model": model}


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
    except urllib.error.HTTPError as exc:
        detail = exc.read().decode("utf-8", errors="replace")
        raise RuntimeError(f"QVAC HTTP {exc.code}: {detail[:2000]}") from exc
    except urllib.error.URLError as exc:
        raise RuntimeError(f"Cannot reach QVAC at {url}: {exc.reason}") from exc
    try:
        result = json.loads(raw)
    except json.JSONDecodeError as exc:
        raise RuntimeError(f"QVAC returned invalid JSON: {raw[:1000]}") from exc
    if not isinstance(result, dict):
        raise RuntimeError("QVAC response must be a JSON object")
    return result


def verify_model(base_url: str, model: str) -> None:
    result = http_json(f"{base_url}/models", timeout=30)
    ids = {
        str(item.get("id"))
        for item in result.get("data", [])
        if isinstance(item, dict) and item.get("id") is not None
    }
    if model not in ids:
        raise RuntimeError(f"Configured vision model '{model}' is not loaded. Loaded models: {sorted(ids)}")


def frame_timestamp(path: Path) -> float:
    match = FRAME_TIMESTAMP_RE.search(path.name)
    if not match:
        raise RuntimeError(f"Frame filename lacks timestamp marker: {path.name}")
    return float(match.group("seconds"))


def select_frames(frame_dir: Path, max_frames: int) -> list[tuple[Path, float]]:
    frames = sorted(frame_dir.glob("*.jpg"))
    if not frames:
        raise RuntimeError(f"No extracted frames found in {frame_dir}")
    if len(frames) > max_frames:
        if max_frames == 1:
            frames = [frames[len(frames) // 2]]
        else:
            indices = [round(i * (len(frames) - 1) / (max_frames - 1)) for i in range(max_frames)]
            frames = [frames[index] for index in sorted(set(indices))]
    return [(frame, frame_timestamp(frame)) for frame in frames]


def image_part(path: Path) -> dict[str, Any]:
    encoded = base64.b64encode(path.read_bytes()).decode("ascii")
    return {
        "type": "image_url",
        "image_url": {"url": f"data:image/jpeg;base64,{encoded}", "detail": "low"},
    }


def parse_model_json(content: str) -> dict[str, Any]:
    text = content.strip()
    if text.startswith("```"):
        lines = text.splitlines()[1:]
        if lines and lines[-1].strip() == "```":
            lines.pop()
        text = "\n".join(lines).strip()
    try:
        result = json.loads(text)
    except json.JSONDecodeError as exc:
        raise RuntimeError(f"Vision model did not return valid JSON: {text[:1000]}") from exc
    if not isinstance(result, dict):
        raise RuntimeError("Vision result must be a JSON object")
    required = {"observed", "temporal_change", "inference", "clip_type", "confidence", "search_terms"}
    missing = required.difference(result)
    if missing:
        raise RuntimeError(f"Vision result missing fields: {sorted(missing)}")
    for field in ("observed", "temporal_change", "inference", "search_terms"):
        value = result[field]
        if not isinstance(value, list) or not all(isinstance(item, str) and item.strip() for item in value):
            raise RuntimeError(f"Vision result field '{field}' must be a list of non-empty strings")
    clip_type = str(result["clip_type"])
    if clip_type not in ALLOWED_CLIP_TYPES:
        raise RuntimeError(f"Unsupported clip_type '{clip_type}'")
    try:
        confidence = float(result["confidence"])
    except (TypeError, ValueError) as exc:
        raise RuntimeError("Vision confidence must be numeric") from exc
    if not 0 <= confidence <= 1:
        raise RuntimeError("Vision confidence must be between 0 and 1")
    result["clip_type"] = clip_type
    result["confidence"] = confidence
    return result


def analyze_clip(
    base_url: str,
    model: str,
    row: sqlite3.Row,
    selected_frames: list[tuple[Path, float]],
) -> dict[str, Any]:
    timestamps = [timestamp for _, timestamp in selected_frames]
    prompt = (
        "Analyze these ordered frames from one video clip. Each image is ordered according to the provided timestamps. "
        "The clip may contain speech, silent nature, little audio, a fixed camera, or timelapse motion. "
        "Put directly visible facts only in observed; cross-frame changes in temporal_change; uncertain meaning only in inference. "
        "Do not identify a person unless evidence is decisive. Return JSON only with exactly these keys: "
        '{"observed":["..."],"temporal_change":["..."],"inference":["..."],'
        '"clip_type":"unknown","confidence":0.0,"search_terms":["..."]}. '
        f"Allowed clip_type values: {sorted(ALLOWED_CLIP_TYPES)}. "
        f"Metadata: filename={row['filename']}; duration={row['duration']} seconds; "
        f"has_audio_track={bool(row['has_audio'])}; image_timestamps={timestamps}."
    )
    content: list[dict[str, Any]] = [{"type": "text", "text": prompt}]
    for frame, timestamp in selected_frames:
        content.append({"type": "text", "text": f"Frame timestamp: {timestamp:.3f} seconds"})
        content.append(image_part(frame))
    payload = {
        "model": model,
        "messages": [{"role": "user", "content": content}],
        "temperature": 0.1,
        "max_tokens": 900,
    }
    response = http_json(f"{base_url}/chat/completions", payload=payload)
    choices = response.get("choices") or []
    if not isinstance(choices, list) or not choices:
        raise RuntimeError(f"QVAC returned no choices: {json.dumps(response)[:1000]}")
    message = choices[0].get("message") if isinstance(choices[0], dict) else None
    message_content = message.get("content") if isinstance(message, dict) else None
    if not isinstance(message_content, str):
        raise RuntimeError("QVAC response did not contain text message content")
    return parse_model_json(message_content)


def update_clip(
    workspace: Path,
    db: sqlite3.Connection,
    row: sqlite3.Row,
    result: dict[str, Any],
    model: str,
    timestamps: list[float],
) -> Path:
    observed_lines = list(result["observed"])
    observed_lines.extend(f"temporal-change: {item}" for item in result["temporal_change"])
    observed_lines.append(f"clip-type: {result['clip_type']}")
    observed_lines.extend(f"search-term: {item}" for item in result["search_terms"])
    inference_lines = list(result["inference"])
    inference_lines.append(f"vision-confidence: {result['confidence']:.3f}")
    document = {
        "schema_version": 2,
        "clip_id": row["clip_id"],
        "source_path": row["source_path"],
        "model": model,
        "analyzed_at": utc_now(),
        "source_mutated": False,
        "frame_timestamps_used": timestamps,
        **result,
    }
    output = workspace / "descriptions" / f"{row['clip_id']}.json"
    output.parent.mkdir(parents=True, exist_ok=True)
    temporary = output.with_suffix(".json.tmp")
    temporary.write_text(json.dumps(document, indent=2), encoding="utf-8")
    temporary.replace(output)
    db.execute(
        "UPDATE clips SET vision_status = ?, observed_text = ?, inference_text = ? WHERE clip_id = ?",
        ("verified-local", "\n".join(observed_lines), "\n".join(inference_lines), row["clip_id"]),
    )
    return output


def enrich(workspace: Path, clip_id: str | None, limit: int, max_frames: int) -> None:
    config = load_config(workspace)
    verify_model(config["base_url"], config["model"])
    with connect(workspace) as db:
        if clip_id:
            rows = db.execute("SELECT * FROM clips WHERE clip_id = ?", (clip_id,)).fetchall()
        else:
            rows = db.execute(
                "SELECT * FROM clips WHERE vision_status != 'verified-local' ORDER BY indexed_at LIMIT ?",
                (limit,),
            ).fetchall()
        if not rows:
            raise RuntimeError("No matching clips require vision enrichment")
        outputs: list[dict[str, Any]] = []
        for row in rows:
            selected = select_frames(Path(row["frame_dir"]), max_frames)
            result = analyze_clip(config["base_url"], config["model"], row, selected)
            timestamps = [timestamp for _, timestamp in selected]
            output = update_clip(workspace, db, row, result, config["model"], timestamps)
            db.commit()
            outputs.append({
                "clip_id": row["clip_id"],
                "description": str(output.resolve()),
                "clip_type": result["clip_type"],
                "confidence": result["confidence"],
                "frames_used": len(selected),
                "frame_timestamps_used": timestamps,
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
        if not 1 <= args.limit <= 100:
            raise RuntimeError("--limit must be between 1 and 100")
        if not 2 <= args.max_frames <= 12:
            raise RuntimeError("--max-frames must be between 2 and 12")
        enrich(args.workspace, args.clip_id, args.limit, args.max_frames)
        return 0
    except Exception as exc:  # noqa: BLE001
        print(json.dumps({"status": "error", "error": str(exc)}, indent=2), file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
