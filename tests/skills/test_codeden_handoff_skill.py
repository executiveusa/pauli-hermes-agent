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

    def test_role_docs_do_not_claim_owner_orchestrator(self):
        soul = (ROOT / "SOUL.md").read_text(encoding="utf-8")
        heart = (ROOT / "HEART.md").read_text(encoding="utf-8")
        self.assertIn("specification and CodeDen handoff architect", soul)
        self.assertIn("Instinct", heart)
        self.assertIn("CodeDen", heart)
        self.assertNotIn("Hermes is the permanent owner-facing orchestrator", heart)


if __name__ == "__main__":
    unittest.main()
