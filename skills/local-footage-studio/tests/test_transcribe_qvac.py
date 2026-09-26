from __future__ import annotations

import importlib.util
from pathlib import Path

MODULE_PATH = Path(__file__).resolve().parents[1] / "scripts" / "transcribe_qvac.py"
spec = importlib.util.spec_from_file_location("transcribe_qvac", MODULE_PATH)
module = importlib.util.module_from_spec(spec)
assert spec and spec.loader
spec.loader.exec_module(module)


def test_replace_transcript_block_adds_block() -> None:
    result = module.replace_transcript_block("visible trees", "hello world")
    assert "visible trees" in result
    assert "[TRANSCRIPT_START]\nhello world\n[TRANSCRIPT_END]" in result


def test_replace_transcript_block_replaces_existing() -> None:
    original = "visual evidence\n[TRANSCRIPT_START]\nold words\n[TRANSCRIPT_END]"
    result = module.replace_transcript_block(original, "new words")
    assert "old words" not in result
    assert result.count("[TRANSCRIPT_START]") == 1
    assert "new words" in result


def test_multipart_contains_required_fields(tmp_path: Path) -> None:
    audio = tmp_path / "clip.wav"
    audio.write_bytes(b"RIFF-test")
    body, boundary = module.multipart_body("whisper-local", audio)
    assert boundary.encode() in body
    assert b'name="model"' in body
    assert b"whisper-local" in body
    assert b'name="file"' in body
    assert b"RIFF-test" in body
