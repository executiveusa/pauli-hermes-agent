import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SKILL = ROOT / "skills" / "workflows" / "hyperagent-browser-operator" / "SKILL.md"
PKG = ROOT / "integrations" / "hyperagent" / "package.json"
RUNNER = ROOT / "integrations" / "hyperagent" / "runner.cjs"
README = ROOT / "integrations" / "hyperagent" / "README.md"


def test_hyperagent_skill_and_adapter_exist():
    assert SKILL.exists()
    assert PKG.exists()
    assert RUNNER.exists()
    assert README.exists()


def test_hyperagent_dependency_is_exact_pinned():
    data = json.loads(PKG.read_text())
    assert data["dependencies"]["@hyperbrowser/agent"] == "1.1.2"
    assert data["dependencies"]["zod"] == "4.1.8"


def test_skill_preserves_icm_and_max_boundary():
    text = SKILL.read_text()
    required = [
        "ICM contract",
        "Human approval required",
        "Never call a browser task complete",
        "Max boundary",
        "Do not copy, enable, or route it into Agent Max",
        "Do not invoke AI browser reasoning when a verified replay",
    ]
    for phrase in required:
        assert phrase in text


def test_runner_restricts_actions_and_routine_names():
    text = RUNNER.read_text()
    for action in ["task", "perform", "ai", "extract", "replay"]:
        assert f"'{action}'" in text
    assert "^[a-zA-Z0-9._-]+$" in text
    assert "HYPERBROWSER_API_KEY" not in text
    assert "OPENAI_API_KEY" not in text


def test_license_boundary_is_explicit():
    skill = SKILL.read_text()
    readme = README.read_text()
    assert "AGPL-3.0" in skill
    assert "AGPL-3.0" in readme
    assert "does not copy HyperAgent source code" in readme
