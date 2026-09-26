import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SKILL_DIR = ROOT / "skills" / "icm-engineering-governor"


def test_icm_engineering_governor_has_progressive_disclosure_layers():
    skill = (SKILL_DIR / "SKILL.md").read_text(encoding="utf-8")
    manifest = json.loads((SKILL_DIR / "manifest.json").read_text(encoding="utf-8"))

    assert "name: icm-engineering-governor" in skill
    assert manifest["activation"] == "progressive-disclosure"
    assert manifest["default_state"] == "sleeping"
    assert "references/engineering.md" in skill
    assert "references/productivity.md" in skill
    assert "references/misc.md" in skill
    assert "references/beta.md" in skill


def test_every_manifest_capability_is_present_in_its_dormant_reference_family():
    manifest = json.loads((SKILL_DIR / "manifest.json").read_text(encoding="utf-8"))

    for family, capabilities in manifest["families"].items():
        reference = (SKILL_DIR / "references" / f"{family}.md").read_text(encoding="utf-8")
        for capability in capabilities:
            assert f"## {capability}" in reference


def test_beta_capabilities_require_an_explicit_release_gate():
    beta = (SKILL_DIR / "references" / "beta.md").read_text(encoding="utf-8")
    manifest = json.loads((SKILL_DIR / "manifest.json").read_text(encoding="utf-8"))

    assert "Beta release gate" in beta
    assert manifest["load_policy"]["beta"] == "never auto-select for production-critical work"


def test_governance_preserves_icm_safety_boundaries():
    manifest = json.loads((SKILL_DIR / "manifest.json").read_text(encoding="utf-8"))
    governance = manifest["governance"]

    assert governance["inspect_before_change"] is True
    assert governance["builder_cannot_self_approve"] is True
    assert governance["rollback_required_for_material_changes"] is True
    assert governance["proof_before_done"] is True
    assert governance["owner_control_required"] is True
