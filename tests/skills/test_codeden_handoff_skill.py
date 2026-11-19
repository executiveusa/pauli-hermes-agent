"""Static contract checks for Cosmos CodeDen handoff skill and templates."""
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[2]
SKILL = ROOT / "skills" / "codeden-handoff" / "SKILL.md"
TEMPLATES = ROOT / "skills" / "codeden-handoff" / "templates"


class CosmosHandoffContractTests(unittest.TestCase):
    def test_skill_frontmatter_and_role(self):
        text = SKILL.read_text(encoding="utf-8")
        self.assertTrue(text.startswith("---\nname: codeden-handoff"))
        for phrase in ("CodeDen", "Paperclip", "independent", "rollback"):
            self.assertIn(phrase.lower(), text.lower())

    def test_issue_template_contains_required_gates(self):
        text = (TEMPLATES / "CODEDEN-ISSUE.md").read_text(encoding="utf-8")
        for phrase in ("Paperclip mission", "acceptance criteria", "Protected",
                       "UI work", "Budget", "Rollback", "Owner approval"):
            self.assertIn(phrase.lower(), text.lower())

    def test_style_kit_is_reviewable_and_existing_policy_is_preserved(self):
        text = (TEMPLATES / "REPO-STYLE-KIT.md").read_text(encoding="utf-8")
        self.assertIn("AGENTS.md", text)
        self.assertIn("CLAUDE.md", text)
        self.assertIn(".codeden/VISION.md", text)
        self.assertIn("Do not blindly overwrite", text)

    def test_icm_prd_has_proof_and_commercial_gate(self):
        text = (TEMPLATES / "PRD-DESIGN-DOC.md").read_text(encoding="utf-8")
        for phrase in ("commercial", "Acceptance", "independent", "rollback"):
            self.assertIn(phrase.lower(), text.lower())

    def test_cosmos_persona_has_no_orchestrator_or_direct_owner_notification(self):
        import re

        persona_dir = ROOT / "agents" / "personas" / "cosmos"
        soul = (persona_dir / "SOUL.md").read_text(encoding="utf-8")
        heart = (persona_dir / "HEART.md").read_text(encoding="utf-8")
        operating = (persona_dir / "OPERATING_SYSTEM.md").read_text(encoding="utf-8")
        yaml_text = (persona_dir / "persona.yaml").read_text(encoding="utf-8")

        # Reject positive authority assertions, but allow explicit negations.
        for doc in (soul, heart, operating):
            self.assertNotRegex(doc, re.compile(
                r"(?im)^\s*(?:you are|cosmos is|hermes is|role:)\s+"
                r"(?:the |a )?(?:personal |chief )?(?:orchestrator|orchestration layer)\b"
            ))
            self.assertNotIn("chief orchestrator for bambu", doc.lower())
        self.assertNotRegex(yaml_text, r"(?im)^\s*role:\s*.*orchestrat")
        self.assertNotRegex(yaml_text, r"(?im)^\s*builder_lane:\s*tars\s*$")
        self.assertRegex(yaml_text, r"(?im)^\s*builder_lane:\s*codeden\s*$")
        self.assertRegex(yaml_text, r"(?im)^\s*recipient:\s*instinct\s*$")
        self.assertRegex(yaml_text, r"(?im)^\s*status:\s*draft-v2\s*$")
        self.assertNotRegex(yaml_text, r"(?im)^\s*-\s*(?:sms|voice_call|owner_direct)\s*$")
        self.assertNotRegex(yaml_text, r"(?im)^\s*isolated_execution:\s*.*sandcastle")
        self.assertIn("Instinct relays", heart)
        self.assertIn("no direct owner intake", heart)

    def test_role_docs_do_not_claim_owner_orchestrator(self):
        soul = (ROOT / "SOUL.md").read_text(encoding="utf-8")
        heart = (ROOT / "HEART.md").read_text(encoding="utf-8")
        self.assertIn("specification and CodeDen handoff architect", soul)
        self.assertIn("Instinct", heart)
        self.assertIn("CodeDen", heart)
        self.assertNotIn("Hermes is the permanent owner-facing orchestrator", heart)


if __name__ == "__main__":
    unittest.main()
