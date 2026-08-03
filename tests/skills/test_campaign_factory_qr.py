from __future__ import annotations

import importlib.util
import json
from pathlib import Path

import pytest

SCRIPT = (
    Path(__file__).resolve().parents[2]
    / "skills"
    / "studio"
    / "campaign-factory"
    / "scripts"
    / "generate_qr.py"
)


def load_module():
    spec = importlib.util.spec_from_file_location("campaign_factory_generate_qr", SCRIPT)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def test_slugify_and_url_guardrails() -> None:
    module = load_module()
    assert module.slugify("Community Cuts for Kids 2026") == "community-cuts-for-kids-2026"
    assert module.validate_https_url("https://asc3nd.org") == "https://asc3nd.org"

    with pytest.raises(Exception):
        module.validate_https_url("http://asc3nd.org")
    with pytest.raises(Exception):
        module.validate_https_url("https://user:secret@example.com")


def test_generates_traceable_svg_png_pdf_package(tmp_path: Path) -> None:
    module = load_module()
    receipt = module.generate_package(
        payload="https://asc3nd.org",
        campaign="Community Cuts for Kids",
        output_dir=tmp_path,
    )

    assert receipt["status"] == "GENERATED_NOT_SCAN_VERIFIED"
    assert receipt["payload"] == "https://asc3nd.org"
    assert receipt["generator"]["version"] == "1.6.6"
    assert receipt["qr"]["is_micro"] is False
    assert receipt["qr"]["border_modules"] == 4

    for suffix in ("svg", "png", "pdf"):
        path = tmp_path / "master" / f"community-cuts-for-kids-qr.{suffix}"
        assert path.is_file()
        assert path.stat().st_size > 0

    svg = (tmp_path / "master" / "community-cuts-for-kids-qr.svg").read_text(
        encoding="utf-8"
    )
    assert "<svg" in svg
    assert "path" in svg

    payload = json.loads((tmp_path / "master" / "payload.json").read_text())
    assert payload["verification"]["physical_device_scan"] == "NOT_TESTED"

    checksums = (tmp_path / "tests" / "checksums.txt").read_text()
    assert "community-cuts-for-kids-qr.svg" in checksums
    assert "community-cuts-for-kids-qr.png" in checksums
    assert "community-cuts-for-kids-qr.pdf" in checksums
