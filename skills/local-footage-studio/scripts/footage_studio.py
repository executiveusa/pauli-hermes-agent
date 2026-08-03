#!/usr/bin/env python3
"""Local Footage Studio prototype.

Deterministic first slice:
- initializes a private workspace and SQLite FTS5 index;
- probes media with ffprobe;
- extracts bounded representative frames with ffmpeg;
- indexes metadata without modifying source footage;
- searches indexed evidence;
- creates reversible JSON edit plans.

Model-based transcription and vision enrichment are deliberately separate follow-up steps.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import shutil
import sqlite3
import subprocess
import sys
import uuid
from dataclasses import asdict, dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Iterable

VIDEO_EXTENSIONS = {".mp4", ".mov", ".mkv", ".webm", ".avi", ".m4v", ".mts", ".m2ts"}


@dataclass
class ClipRecord:
    clip_id: str
    source_path: str
    filename: str
    duration: float
    width: int | None
    height: int | None
    fps: float | None
    has_audio: bool
    size_bytes: int
    modified_ns: int
    frame_dir: str
    frame_timestamps: list[float]
    transcript_status: str = "pending"
    vision_status: str = "pending"
    observed_text: str = ""
    inference_text: str = ""
    indexed_at: str = ""


def utc_now() -> str:
    return datetime.now(timezone.utc).isoformat()


def run(command: list[str]) -> subprocess.CompletedProcess[str]:
    try:
        return subprocess.run(command, check=True, capture_output=True, text=True)
    except FileNotFoundError as exc:
        raise RuntimeError(f"Required executable not found: {command[0]}") from exc
    except subprocess.CalledProcessError as exc:
        detail = (exc.stderr or exc.stdout or "unknown error").strip()
        raise RuntimeError(f"Command failed: {' '.join(command)}\n{detail}") from exc


def require_tools() -> None:
    missing = [tool for tool in ("ffmpeg", "ffprobe") if shutil.which(tool) is None]
    if missing:
        raise RuntimeError(
            "Missing required tools: " + ", ".join(missing) +
            ". Install FFmpeg and ensure ffmpeg/ffprobe are on PATH."
        )


def connect(workspace: Path) -> sqlite3.Connection:
    workspace.mkdir(parents=True, exist_ok=True)
    db_path = workspace / "footage.db"
    connection = sqlite3.connect(db_path)
    connection.row_factory = sqlite3.Row
    connection.execute("PRAGMA journal_mode=WAL")
    connection.execute("PRAGMA foreign_keys=ON")
    return connection


def initialize(workspace: Path) -> None:
    for folder in ("frames", "audio", "transcripts", "descriptions", "edit-plans", "logs", "manifests"):
        (workspace / folder).mkdir(parents=True, exist_ok=True)
    with connect(workspace) as db:
        db.executescript(
            """
            CREATE TABLE IF NOT EXISTS clips (
                clip_id TEXT PRIMARY KEY,
                source_path TEXT UNIQUE NOT NULL,
                filename TEXT NOT NULL,
                duration REAL NOT NULL,
                width INTEGER,
                height INTEGER,
                fps REAL,
                has_audio INTEGER NOT NULL,
                size_bytes INTEGER NOT NULL,
                modified_ns INTEGER NOT NULL,
                frame_dir TEXT NOT NULL,
                frame_timestamps TEXT NOT NULL,
                transcript_status TEXT NOT NULL,
                vision_status TEXT NOT NULL,
                observed_text TEXT NOT NULL,
                inference_text TEXT NOT NULL,
                indexed_at TEXT NOT NULL
            );
            CREATE VIRTUAL TABLE IF NOT EXISTS clips_fts USING fts5(
                clip_id UNINDEXED,
                filename,
                source_path,
                observed_text,
                inference_text,
                content='clips',
                content_rowid='rowid'
            );
            CREATE TRIGGER IF NOT EXISTS clips_ai AFTER INSERT ON clips BEGIN
              INSERT INTO clips_fts(rowid, clip_id, filename, source_path, observed_text, inference_text)
              VALUES (new.rowid, new.clip_id, new.filename, new.source_path, new.observed_text, new.inference_text);
            END;
            CREATE TRIGGER IF NOT EXISTS clips_ad AFTER DELETE ON clips BEGIN
              INSERT INTO clips_fts(clips_fts, rowid, clip_id, filename, source_path, observed_text, inference_text)
              VALUES('delete', old.rowid, old.clip_id, old.filename, old.source_path, old.observed_text, old.inference_text);
            END;
            CREATE TRIGGER IF NOT EXISTS clips_au AFTER UPDATE ON clips BEGIN
              INSERT INTO clips_fts(clips_fts, rowid, clip_id, filename, source_path, observed_text, inference_text)
              VALUES('delete', old.rowid, old.clip_id, old.filename, old.source_path, old.observed_text, old.inference_text);
              INSERT INTO clips_fts(rowid, clip_id, filename, source_path, observed_text, inference_text)
              VALUES (new.rowid, new.clip_id, new.filename, new.source_path, new.observed_text, new.inference_text);
            END;
            """
        )
    config = workspace / "config.json"
    if not config.exists():
        config.write_text(json.dumps({
            "schema_version": 1,
            "created_at": utc_now(),
            "source_mutation_allowed": False,
            "qvac_base_url": "http://127.0.0.1:11434/v1",
            "vision_model": None,
            "transcription_model": None
        }, indent=2), encoding="utf-8")
    print(json.dumps({"status": "initialized", "workspace": str(workspace.resolve())}, indent=2))


def parse_fraction(value: str | None) -> float | None:
    if not value or value in {"0/0", "N/A"}:
        return None
    try:
        numerator, denominator = value.split("/", 1)
        denominator_float = float(denominator)
        return float(numerator) / denominator_float if denominator_float else None
    except (ValueError, ZeroDivisionError):
        return None


def probe(path: Path) -> dict[str, Any]:
    result = run([
        "ffprobe", "-v", "error", "-print_format", "json",
        "-show_format", "-show_streams", str(path)
    ])
    data = json.loads(result.stdout)
    streams = data.get("streams", [])
    video = next((stream for stream in streams if stream.get("codec_type") == "video"), {})
    audio = next((stream for stream in streams if stream.get("codec_type") == "audio"), None)
    duration_raw = data.get("format", {}).get("duration") or video.get("duration") or 0
    return {
        "duration": max(float(duration_raw), 0.0),
        "width": video.get("width"),
        "height": video.get("height"),
        "fps": parse_fraction(video.get("avg_frame_rate") or video.get("r_frame_rate")),
        "has_audio": audio is not None,
    }


def stable_clip_id(path: Path, stat: Any) -> str:
    payload = f"{path.resolve()}|{stat.st_size}|{stat.st_mtime_ns}".encode("utf-8", errors="surrogatepass")
    return hashlib.sha256(payload).hexdigest()[:20]


def sample_timestamps(duration: float, max_frames: int) -> list[float]:
    if duration <= 0 or max_frames < 1:
        return [0.0]
    count = min(max_frames, max(3, int(duration // 30) + 3))
    if count == 1:
        return [min(duration / 2, max(duration - 0.05, 0))]
    end = max(duration - 0.10, 0)
    points = [end * index / (count - 1) for index in range(count)]
    return sorted({round(point, 3) for point in points})


def extract_frames(path: Path, destination: Path, timestamps: Iterable[float], width: int) -> list[str]:
    destination.mkdir(parents=True, exist_ok=True)
    outputs: list[str] = []
    for index, timestamp in enumerate(timestamps):
        output = destination / f"frame-{index:03d}-t{timestamp:.3f}.jpg"
        run([
            "ffmpeg", "-hide_banner", "-loglevel", "error", "-y",
            "-ss", f"{timestamp:.3f}", "-i", str(path), "-frames:v", "1",
            "-vf", f"scale={width}:-2", "-q:v", "3", str(output)
        ])
        outputs.append(str(output))
    return outputs


def discover_videos(source: Path) -> list[Path]:
    if source.is_file():
        return [source] if source.suffix.lower() in VIDEO_EXTENSIONS else []
    return sorted(
        path for path in source.rglob("*")
        if path.is_file() and path.suffix.lower() in VIDEO_EXTENSIONS
    )


def upsert_clip(db: sqlite3.Connection, record: ClipRecord) -> None:
    values = asdict(record)
    values["has_audio"] = int(record.has_audio)
    values["frame_timestamps"] = json.dumps(record.frame_timestamps)
    columns = list(values)
    placeholders = ", ".join(f":{column}" for column in columns)
    updates = ", ".join(f"{column}=excluded.{column}" for column in columns if column != "clip_id")
    db.execute(
        f"INSERT INTO clips ({', '.join(columns)}) VALUES ({placeholders}) "
        f"ON CONFLICT(source_path) DO UPDATE SET {updates}", values
    )


def index_source(workspace: Path, source: Path, max_frames: int, frame_width: int) -> None:
    require_tools()
    initialize(workspace)
    videos = discover_videos(source)
    if not videos:
        raise RuntimeError(f"No supported video files found under: {source}")
    results: list[dict[str, Any]] = []
    with connect(workspace) as db:
        for path in videos:
            stat = path.stat()
            metadata = probe(path)
            clip_id = stable_clip_id(path, stat)
            timestamps = sample_timestamps(metadata["duration"], max_frames)
            frame_dir = workspace / "frames" / clip_id
            frames = extract_frames(path, frame_dir, timestamps, frame_width)
            observed = "audio-present" if metadata["has_audio"] else "silent-or-no-audio-track"
            record = ClipRecord(
                clip_id=clip_id,
                source_path=str(path.resolve()),
                filename=path.name,
                duration=metadata["duration"],
                width=metadata["width"],
                height=metadata["height"],
                fps=metadata["fps"],
                has_audio=metadata["has_audio"],
                size_bytes=stat.st_size,
                modified_ns=stat.st_mtime_ns,
                frame_dir=str(frame_dir.resolve()),
                frame_timestamps=timestamps,
                observed_text=observed,
                indexed_at=utc_now(),
            )
            upsert_clip(db, record)
            results.append({
                "clip_id": clip_id,
                "source_path": record.source_path,
                "duration": record.duration,
                "has_audio": record.has_audio,
                "frames": frames,
                "vision_status": record.vision_status,
                "transcript_status": record.transcript_status,
            })
        db.commit()
    manifest = workspace / "manifests" / f"index-{datetime.now().strftime('%Y%m%d-%H%M%S')}.json"
    manifest.write_text(json.dumps(results, indent=2), encoding="utf-8")
    print(json.dumps({
        "status": "index-created",
        "clips": len(results),
        "manifest": str(manifest.resolve()),
        "source_files_modified": False,
        "limitations": ["vision enrichment pending", "transcription pending"]
    }, indent=2))


def search(workspace: Path, query: str, limit: int) -> None:
    with connect(workspace) as db:
        try:
            rows = db.execute(
                """
                SELECT c.*, bm25(clips_fts) AS score
                FROM clips_fts JOIN clips c ON c.rowid = clips_fts.rowid
                WHERE clips_fts MATCH ?
                ORDER BY score LIMIT ?
                """, (query, limit)
            ).fetchall()
        except sqlite3.OperationalError:
            wildcard = f"%{query}%"
            rows = db.execute(
                "SELECT *, 0 AS score FROM clips WHERE filename LIKE ? OR source_path LIKE ? OR observed_text LIKE ? LIMIT ?",
                (wildcard, wildcard, wildcard, limit)
            ).fetchall()
    output = []
    for row in rows:
        item = dict(row)
        item["frame_timestamps"] = json.loads(item["frame_timestamps"])
        output.append(item)
    print(json.dumps({"query": query, "count": len(output), "results": output}, indent=2))


def create_plan(workspace: Path, project: str, clip_id: str, start: float, end: float, reason: str) -> None:
    if end <= start:
        raise RuntimeError("Edit-plan out point must be greater than in point")
    with connect(workspace) as db:
        row = db.execute("SELECT * FROM clips WHERE clip_id = ?", (clip_id,)).fetchone()
    if row is None:
        raise RuntimeError(f"Unknown clip_id: {clip_id}")
    if end > float(row["duration"]) + 0.001:
        raise RuntimeError(f"Out point {end} exceeds clip duration {row['duration']}")
    plan = {
        "schema_version": 1,
        "project": project,
        "created_at": utc_now(),
        "status": "PROPOSED",
        "source_mutation_allowed": False,
        "source_clips": [{
            "clip_id": clip_id,
            "path": row["source_path"],
            "in": start,
            "out": end,
            "reason": reason,
            "evidence": [start, end]
        }],
        "transitions": "cuts-only",
        "rendered": False
    }
    safe_name = "".join(character if character.isalnum() or character in "-_" else "-" for character in project)
    output = workspace / "edit-plans" / f"{safe_name}-{uuid.uuid4().hex[:8]}.json"
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(plan, indent=2), encoding="utf-8")
    print(json.dumps({"status": "edit-plan-created", "path": str(output.resolve()), "plan": plan}, indent=2))


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Private local footage indexing prototype")
    subparsers = parser.add_subparsers(dest="command", required=True)

    init_parser = subparsers.add_parser("init")
    init_parser.add_argument("--workspace", required=True, type=Path)

    index_parser = subparsers.add_parser("index")
    index_parser.add_argument("--workspace", required=True, type=Path)
    index_parser.add_argument("--source", required=True, type=Path)
    index_parser.add_argument("--max-frames", type=int, default=8)
    index_parser.add_argument("--frame-width", type=int, default=512)

    search_parser = subparsers.add_parser("search")
    search_parser.add_argument("--workspace", required=True, type=Path)
    search_parser.add_argument("--query", required=True)
    search_parser.add_argument("--limit", type=int, default=10)

    plan_parser = subparsers.add_parser("plan")
    plan_parser.add_argument("--workspace", required=True, type=Path)
    plan_parser.add_argument("--project", required=True)
    plan_parser.add_argument("--clip-id", required=True)
    plan_parser.add_argument("--start", required=True, type=float)
    plan_parser.add_argument("--end", required=True, type=float)
    plan_parser.add_argument("--reason", required=True)
    return parser


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()
    try:
        if args.command == "init":
            initialize(args.workspace)
        elif args.command == "index":
            if args.max_frames < 1 or args.max_frames > 100:
                raise RuntimeError("--max-frames must be between 1 and 100")
            if args.frame_width < 128 or args.frame_width > 2048:
                raise RuntimeError("--frame-width must be between 128 and 2048")
            index_source(args.workspace, args.source, args.max_frames, args.frame_width)
        elif args.command == "search":
            search(args.workspace, args.query, args.limit)
        elif args.command == "plan":
            create_plan(args.workspace, args.project, args.clip_id, args.start, args.end, args.reason)
        return 0
    except Exception as exc:  # CLI boundary: return actionable failure without traceback noise.
        print(json.dumps({"status": "error", "error": str(exc)}, indent=2), file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
