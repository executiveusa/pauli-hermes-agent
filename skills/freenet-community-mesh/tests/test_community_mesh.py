from __future__ import annotations

import importlib.util
from pathlib import Path

MODULE_PATH = Path(__file__).parents[1] / "scripts" / "community_mesh.py"
spec = importlib.util.spec_from_file_location("community_mesh", MODULE_PATH)
assert spec and spec.loader
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


def valid_record() -> dict:
    return {
        "resource_id": "r1",
        "organization_id": "org1",
        "category": "translation",
        "service": "Public interpretation referral",
        "location": "Seattle, WA",
        "languages": ["en", "es"],
        "availability": "By appointment",
        "contact_method": "public@example.org",
        "updated_at": "2026-08-06T00:00:00+00:00",
        "status": "active",
    }


def test_demo_preserves_private_isolation_and_revocation(tmp_path: Path) -> None:
    result = module.demo(tmp_path / "mesh")
    assert result["status"] == "PASS"
    assert result["revocation_verified"] is True
    assert result["private_leakage_detected"] is False
    assert result["live_freenet_verified"] is False
    destination = module.Node(tmp_path / "mesh" / "everett-node")
    shared_text = destination.shared_path.read_text(encoding="utf-8")
    assert "PRIVATE-A-001" not in shared_text
    assert "Never publish" not in shared_text


def test_rejects_unknown_private_field(tmp_path: Path) -> None:
    node = module.Node(tmp_path / "node")
    node.init()
    record = valid_record()
    record["client_name"] = "Private Person"
    try:
        node.publish(record)
    except ValueError as exc:
        assert "unknown fields" in str(exc) or "prohibited field" in str(exc)
    else:
        raise AssertionError("private field was accepted")


def test_rejects_secret_pattern(tmp_path: Path) -> None:
    node = module.Node(tmp_path / "node")
    node.init()
    record = valid_record()
    record["service"] = "Accidental token 21st_sk_abcdefgh12345678"
    try:
        node.publish(record)
    except ValueError as exc:
        assert "sensitive pattern" in str(exc)
    else:
        raise AssertionError("secret pattern was accepted")


def test_tampered_record_is_rejected(tmp_path: Path) -> None:
    source = module.Node(tmp_path / "source")
    destination = module.Node(tmp_path / "destination")
    source.init()
    destination.init()
    published = source.publish(valid_record())
    published["service"] = "Tampered service"
    source.write_atomic(source.shared_path, [published])
    try:
        module.sync(source, destination)
    except ValueError as exc:
        assert "invalid signature" in str(exc)
    else:
        raise AssertionError("tampered record was synchronized")
