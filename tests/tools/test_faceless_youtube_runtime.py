import os
from pathlib import Path

import pytest

from tools.faceless_youtube_runtime.runtime import FacelessYouTubeRuntime
from tools.faceless_youtube_runtime.youtube import upload_video


def test_job_creation_is_idempotent(tmp_path: Path):
    runtime = FacelessYouTubeRuntime(home=tmp_path / "home", repo_root=tmp_path)
    payload = {"channel": "test", "topic": "one"}
    first = runtime.create_job(payload)
    second = runtime.create_job(payload)
    assert first["id"] == second["id"]
    assert first["status"] == "queued"


def test_job_transition_and_receipt(tmp_path: Path):
    runtime = FacelessYouTubeRuntime(home=tmp_path / "home", repo_root=tmp_path)
    job = runtime.create_job({"channel": "test"})
    changed = runtime.update_job(job["id"], stage="script", status="complete")
    assert changed["stage"] == "script"
    receipt = runtime.write_receipt(job["id"], "test", {"ok": True})
    assert receipt.exists()
    assert receipt.stat().st_size > 0


def test_unknown_stage_fails(tmp_path: Path):
    runtime = FacelessYouTubeRuntime(home=tmp_path / "home", repo_root=tmp_path)
    job = runtime.create_job({"channel": "test"})
    with pytest.raises(ValueError):
        runtime.update_job(job["id"], stage="not-a-stage")


def test_publish_requires_explicit_approval(tmp_path: Path, monkeypatch):
    monkeypatch.setenv("HERMES_YOUTUBE_PUBLISH_ENABLED", "1")
    video = tmp_path / "video.mp4"
    video.write_bytes(b"fake")
    with pytest.raises(PermissionError, match="explicit publish approval"):
        upload_video(str(video), {"title": "test"}, approved=False)


def test_doctor_does_not_expose_secrets(tmp_path: Path, monkeypatch):
    monkeypatch.setenv("HYPERBROWSER_API_KEY", "do-not-leak")
    runtime = FacelessYouTubeRuntime(home=tmp_path / "home", repo_root=tmp_path)
    text = str(runtime.doctor())
    assert "do-not-leak" not in text
