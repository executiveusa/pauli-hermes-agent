"""Guard rails for the bundled Surface desktop-control skill."""
from pathlib import Path
import re

SKILL = Path(__file__).resolve().parents[2] / "skills" / "surface-desktop-control" / "SKILL.md"

def test_surface_skill_contract():
    text = SKILL.read_text(encoding="utf-8")
    assert text.startswith("---\n")
    match = re.search(r"^description: (.+)$", text, re.MULTILINE)
    assert match and len(match.group(1)) <= 60 and match.group(1).endswith(".")
    assert "platforms:" in text
    for section in ("When to Use", "Prerequisites", "How to Run", "Quick Reference", "Procedure", "Pitfalls", "Verification"):
        assert f"## {section}" in text

def test_surface_skill_does_not_grant_unrestricted_input():
    text = SKILL.read_text(encoding="utf-8")
    assert 'can = "view"' in text
    assert 'can = ["view", "input"]' in text
    assert "owner authorization" in text.lower()
    assert "never claim gui access" in text.lower()
    assert "rollback" in text.lower()
