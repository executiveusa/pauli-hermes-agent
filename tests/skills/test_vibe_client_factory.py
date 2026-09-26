from __future__ import annotations

import importlib.util
import json
import shutil
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SKILL_DIR = REPO_ROOT / "skills" / "studio" / "vibe-client-factory"
DOCTOR_PATH = SKILL_DIR / "scripts" / "doctor.py"
REGISTRY_PATH = REPO_ROOT / "skills" / "SKILL_REGISTRY.json"


def load_doctor():
    spec = importlib.util.spec_from_file_location("vibe_client_factory_doctor", DOCTOR_PATH)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def test_vibe_client_factory_skill_passes_doctor() -> None:
    module = load_doctor()
    report = module.inspect_skill(SKILL_DIR, REGISTRY_PATH)

    assert report["status"] == "PASS", report["errors"]
    assert report["errors"] == []
    assert report["checked_files"] == 5


def test_doctor_blocks_false_ab_and_unsafe_release_contracts(tmp_path: Path) -> None:
    module = load_doctor()
    copied_skill = tmp_path / "vibe-client-factory"
    shutil.copytree(SKILL_DIR, copied_skill)

    decision_path = copied_skill / "templates" / "decision-card.schema.json"
    decision = json.loads(decision_path.read_text(encoding="utf-8"))
    decision["properties"]["options"]["maxItems"] = 3
    decision_path.write_text(json.dumps(decision, indent=2) + "\n", encoding="utf-8")

    authority_path = copied_skill / "templates" / "authority-policy.json"
    authority = json.loads(authority_path.read_text(encoding="utf-8"))
    for item in authority["classes"]:
        if item["id"] == "RELEASE":
            item["automatic"] = True
        if item["id"] == "PROHIBITED":
            item["approval"] = "OWNER_OVERRIDE"
    authority_path.write_text(json.dumps(authority, indent=2) + "\n", encoding="utf-8")

    report = module.inspect_skill(copied_skill, None)

    assert report["status"] == "BLOCKED"
    assert "Decision cards must contain exactly two options" in report["errors"]
    assert "RELEASE authority must not be automatic" in report["errors"]
    assert "PROHIBITED authority must use approval NEVER" in report["errors"]


def test_doctor_blocks_invalid_judge_and_missing_lazy_load_metadata(tmp_path: Path) -> None:
    module = load_doctor()
    copied_skill = tmp_path / "vibe-client-factory"
    shutil.copytree(SKILL_DIR, copied_skill)

    skill_path = copied_skill / "SKILL.md"
    skill_content = skill_path.read_text(encoding="utf-8")
    skill_content = skill_content.replace("triggers:\n", "activation_phrases:\n", 1)
    skill_path.write_text(skill_content, encoding="utf-8")

    receipt_path = copied_skill / "templates" / "run-receipt.schema.json"
    receipt = json.loads(receipt_path.read_text(encoding="utf-8"))
    receipt["properties"]["judge_verdict"]["enum"] = ["SHIP", "HOLD", "BLOCKED"]
    receipt_path.write_text(json.dumps(receipt, indent=2) + "\n", encoding="utf-8")

    report = module.inspect_skill(copied_skill, None)

    assert report["status"] == "BLOCKED"
    assert "SKILL.md must declare triggers for lazy loading" in report["errors"]
    assert "Judge verdict must be exactly SHIP or HOLD" in report["errors"]
