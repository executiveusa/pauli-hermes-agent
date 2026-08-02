#!/usr/bin/env python3
"""Transcribe indexed footage through a local QVAC OpenAI-compatible endpoint.

Non-destructive behavior:
- extracts temporary mono 16 kHz WAV audio with ffmpeg;
- treats clips without an audio stream as a valid no-audio result;
- verifies the configured transcription model through /v1/models;
- calls /v1/audio/transcriptions using multipart/form-data;
- writes derived transcript JSON/text and updates searchable evidence;
- never modifies source footage.
"""
from __future__ import annotations

import argparse
import json
import os
import shutil
import sqlite3
import subprocess
import sys
import tempfile
import urllib.error
import urllib.request
import uuid
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

TRANSCRIPT_START = "[TRANSCRIPT_START]"
TRANSCRIPT_END = "[TRANSCRIPT_END]"


def utc_now() -> str:
    return datetime.now(timezone.utc).isoformat()


def connect(workspace: Path) -> sqlite3.Connection:
    db = sqlite3.connect(workspace / "footage.db")
    db.row_factory = sqlite3.Row
    return db


def load_config(workspace: Path) -> dict[str, str]:
    path = workspace / "config.json"
    if not path.exists():
        raise RuntimeError(f"Missing workspace config: {path}")
    data = json.loads(path.read_text(encoding="utf-8"))
    base_url = str(data.get("qvac_base_url") or "").rstrip("/")
    model = str(data.get("transcription_model") or "")
    if not base_url:
        raise RuntimeError("config.json qvac_base_url is empty")
    if not model:
        raise RuntimeError("config.json transcription_model is unset")
    return {"base_url": base_url, "model": model}


def http_json(url: str, timeout: int = 30) -> dict[str, Any]:
    try:
        with urllib.request.urlopen(url, timeout=timeout) as response:
            value = json.loads(response.read().decode("utf-8"))
    except urllib.error.HTTPError as exc:
        detail = exc.read().decode("utf-8", errors="replace")
        raise RuntimeError(f"QVAC HTTP {exc.code}: {detail[:2000]}") from exc
    except urllib.error.URLError as exc:
        raise RuntimeError(f"Cannot reach QVAC at {url}: {exc.reason}") from exc
    if not isinstance(value, dict):
        raise RuntimeError("QVAC returned non-object JSON")
    return value


def verify_model(base_url: str, model: str) -> None:
    result = http_json(f"{base_url}/models")
    ids = {str(item.get("id")) for item in result.get("data", []) if isinstance(item, dict)}
    if model not in ids:
        raise RuntimeError(f"Configured transcription model '{model}' is not loaded. Loaded models: {sorted(ids)}")


def extract_audio(source: Path, destination: Path) -> None:
    if shutil.which("ffmpeg") is None:
        raise RuntimeError("ffmpeg is required and was not found on PATH")
    command = [
        "ffmpeg", "-hide_banner", "-loglevel", "error", "-y", "-i", str(source),
        "-vn", "-ac", "1", "-ar", "16000", "-c:a", "pcm_s16le", str(destination),
    ]
    try:
        subprocess.run(command, check=True, capture_output=True, text=True)
    except subprocess.CalledProcessError as exc:
        detail = (exc.stderr or exc.stdout or "unknown ffmpeg error").strip()
        raise RuntimeError(f"Audio extraction failed: {detail}") from exc


def multipart_body(model: str, audio_path: Path) -> tuple[bytes, str]:
    boundary = f"----PauliFootage{uuid.uuid4().hex}"
    chunks: list[bytes] = []
    def field(name: str, value: str) -> None:
        chunks.extend([
            f"--{boundary}\r\n".encode(),
            f'Content-Disposition: form-data; name="{name}"\r\n\r\n'.encode(),
            value.encode("utf-8"), b"\r\n",
        ])
    field("model", model)
    field("response_format", "verbose_json")
    chunks.extend([
        f"--{boundary}\r\n".encode(),
        f'Content-Disposition: form-data; name="file"; filename="{audio_path.name}"\r\n'.encode(),
        b"Content-Type: audio/wav\r\n\r\n",
        audio_path.read_bytes(), b"\r\n",
        f"--{boundary}--\r\n".encode(),
    ])
    return b"".join(chunks), boundary


def transcribe(base_url: str, model: str, audio_path: Path) -> dict[str, Any]:
    body, boundary = multipart_body(model, audio_path)
    request = urllib.request.Request(
        f"{base_url}/audio/transcriptions", data=body, method="POST",
        headers={"Content-Type": f"multipart/form-data; boundary={boundary}"},
    )
    try:
        with urllib.request.urlopen(request, timeout=600) as response:
            result = json.loads(response.read().decode("utf-8"))
    except urllib.error.HTTPError as exc:
        detail = exc.read().decode("utf-8", errors="replace")
        raise RuntimeError(f"QVAC transcription HTTP {exc.code}: {detail[:2000]}") from exc
    except urllib.error.URLError as exc:
        raise RuntimeError(f"Cannot reach QVAC transcription endpoint: {exc.reason}") from exc
    if not isinstance(result, dict):
        raise RuntimeError("Transcription response must be a JSON object")
    text = result.get("text")
    if not isinstance(text, str):
        raise RuntimeError("Transcription response is missing text")
    result["text"] = text.strip()
    return result


def replace_transcript_block(existing: str, transcript: str) -> str:
    start = existing.find(TRANSCRIPT_START)
    end = existing.find(TRANSCRIPT_END)
    if start >= 0 and end >= start:
        existing = (existing[:start] + existing[end + len(TRANSCRIPT_END):]).strip()
    block = f"{TRANSCRIPT_START}\n{transcript}\n{TRANSCRIPT_END}"
    return f"{existing}\n{block}".strip() if existing else block


def process_clip(workspace: Path, db: sqlite3.Connection, row: sqlite3.Row, config: dict[str, str]) -> dict[str, Any]:
    clip_id = str(row["clip_id"])
    output_json = workspace / "transcripts" / f"{clip_id}.json"
    output_txt = workspace / "transcripts" / f"{clip_id}.txt"
    output_json.parent.mkdir(parents=True, exist_ok=True)

    if not bool(row["has_audio"]):
        document = {"schema_version": 1, "clip_id": clip_id, "status": "no-audio-track", "text": "", "source_mutated": False, "processed_at": utc_now()}
        output_json.write_text(json.dumps(document, indent=2), encoding="utf-8")
        output_txt.write_text("", encoding="utf-8")
        db.execute("UPDATE clips SET transcript_status = ? WHERE clip_id = ?", ("no-audio-track", clip_id))
        return {"clip_id": clip_id, "status": "no-audio-track"}

    verify_model(config["base_url"], config["model"])
    with tempfile.TemporaryDirectory(prefix="pauli-footage-") as temp_dir:
        audio_path = Path(temp_dir) / f"{clip_id}.wav"
        extract_audio(Path(row["source_path"]), audio_path)
        result = transcribe(config["base_url"], config["model"], audio_path)

    text = str(result["text"]).strip()
    status = "verified-local" if text else "verified-local-empty"
    document = {"schema_version": 1, "clip_id": clip_id, "source_path": row["source_path"], "model": config["model"], "status": status, "text": text, "segments": result.get("segments", []), "language": result.get("language"), "source_mutated": False, "processed_at": utc_now()}
    temp_output = output_json.with_suffix(".json.tmp")
    temp_output.write_text(json.dumps(document, indent=2), encoding="utf-8")
    os.replace(temp_output, output_json)
    output_txt.write_text(text, encoding="utf-8")
    observed = replace_transcript_block(str(row["observed_text"] or ""), text)
    db.execute("UPDATE clips SET transcript_status = ?, observed_text = ? WHERE clip_id = ?", (status, observed, clip_id))
    return {"clip_id": clip_id, "status": status, "characters": len(text), "language": result.get("language")}


def run_batch(workspace: Path, clip_id: str | None, limit: int) -> None:
    config = load_config(workspace)
    with connect(workspace) as db:
        if clip_id:
            rows = db.execute("SELECT * FROM clips WHERE clip_id = ?", (clip_id,)).fetchall()
        else:
            rows = db.execute("SELECT * FROM clips WHERE transcript_status = 'pending' ORDER BY indexed_at LIMIT ?", (limit,)).fetchall()
        if not rows:
            raise RuntimeError("No matching clips require transcription")
        outputs = []
        for row in rows:
            outputs.append(process_clip(workspace, db, row, config))
            db.commit()
    print(json.dumps({"status": "transcription-complete", "provider": "qvac-local", "clips": outputs, "source_files_modified": False}, indent=2))


def main() -> int:
    parser = argparse.ArgumentParser(description="Transcribe indexed footage with local QVAC")
    parser.add_argument("--workspace", required=True, type=Path)
    parser.add_argument("--clip-id")
    parser.add_argument("--limit", type=int, default=1)
    args = parser.parse_args()
    try:
        if args.limit < 1 or args.limit > 100:
            raise RuntimeError("--limit must be between 1 and 100")
        run_batch(args.workspace, args.clip_id, args.limit)
        return 0
    except Exception as exc:  # noqa: BLE001
        print(json.dumps({"status": "error", "error": str(exc)}, indent=2), file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
