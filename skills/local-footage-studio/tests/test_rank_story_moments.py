from __future__ import annotations

import importlib.util
import sqlite3
from pathlib import Path

MODULE_PATH = Path(__file__).resolve().parents[1] / "scripts" / "rank_story_moments.py"
spec = importlib.util.spec_from_file_location("rank_story_moments", MODULE_PATH)
module = importlib.util.module_from_spec(spec)
assert spec and spec.loader
spec.loader.exec_module(module)


def row(**overrides):
    values = {
        "clip_id": "abc",
        "source_path": "/video.mp4",
        "filename": "video.mp4",
        "duration": 30.0,
        "vision_status": "verified-local",
        "transcript_status": "verified-local",
        "observed_text": "trees\ntemporal-change: sky darkens\n[TRANSCRIPT_START]\nhello world\n[TRANSCRIPT_END]",
        "inference_text": "likely sunset",
    }
    values.update(overrides)
    connection = sqlite3.connect(":memory:")
    connection.row_factory = sqlite3.Row
    columns = list(values)
    connection.execute(f"CREATE TABLE clip ({', '.join(f'{name} TEXT' for name in columns)})")
    connection.execute(
        f"INSERT INTO clip VALUES ({', '.join('?' for _ in columns)})",
        tuple(str(values[name]) for name in columns),
    )
    return connection.execute("SELECT * FROM clip").fetchone()


def test_transcript_text_extracts_block() -> None:
    assert module.transcript_text("x[TRANSCRIPT_START]\nhello\n[TRANSCRIPT_END]y") == "hello"


def test_enriched_clip_scores_above_metadata_only() -> None:
    enriched = module.score_clip(row())
    metadata = module.score_clip(row(vision_status="pending", transcript_status="pending", observed_text="", inference_text=""))
    assert enriched["score"] > metadata["score"]
    assert enriched["source_mutated"] is False


def test_score_is_bounded() -> None:
    result = module.score_clip(row(observed_text="temporal-change: x\n" * 100))
    assert 0 <= result["score"] <= 1
