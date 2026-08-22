import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SKILL = ROOT / "skills" / "portable-seo-auditor" / "SKILL.md"
REGISTRY = ROOT / "skills" / "SKILL_REGISTRY.json"


def test_portable_seo_skill_is_registered_and_enabled():
    registry = json.loads(REGISTRY.read_text(encoding="utf-8"))
    entry = registry["portable-seo-auditor"]
    assert entry["enabled"] is True
    assert entry["path"] == "skills/portable-seo-auditor"
    assert entry["entry_point"] == "/portable-seo-auditor"


def test_skill_preserves_icm_and_evidence_contract():
    text = SKILL.read_text(encoding="utf-8")
    for heading in ("### INPUT", "### PROCESS", "### OUTPUT", "### GATE", "### RECEIPT"):
        assert heading in text
    for state in ("VERIFIED", "CLIENT_STATED", "INFERRED", "UNKNOWN"):
        assert state in text
    assert "only two legal outcomes" in text
    assert "`PARTIAL` is never a gate outcome" in text


def test_skill_knows_direct_and_subagent_execution_paths():
    text = SKILL.read_text(encoding="utf-8")
    assert "executiveusa/pauli-claude-seo" in text
    assert "python scripts/pauli_seo.py audit" in text
    assert "hardened-longrun-subagent-harness" in text
    assert "technical-performance" in text
    assert "local-entity" in text
    assert "content-schema-geo" in text
    assert "sxo-conversion" in text
    assert "authority-competitors" in text
    assert "verifier" in text


def test_audit_mode_is_read_only():
    text = SKILL.read_text(encoding="utf-8")
    assert "Audit mode is read-only" in text
    assert "Do not:" in text
    assert "launch ads" in text
    assert "edit DNS" in text
    assert "modify production site code" in text
