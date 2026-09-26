import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
GOVERNOR = ROOT / "skills" / "icm-engineering-governor"
PROCESS = GOVERNOR / "workflows" / "code-review" / "PROCESS.md"
CRON = ROOT / "cron" / "icm-code-review.json"


def test_code_review_process_exists_and_requires_human_prompt():
    text = PROCESS.read_text(encoding="utf-8")
    assert "Hermes MUST ask the owner to review the code" in text
    assert "HUMAN_REVIEW_PROMPTED" in text
    assert "HUMAN_REVIEW_RESULT" in text
    assert "APPROVED" in text
    assert "CHANGES_REQUESTED" in text
    assert "DECLINED_REVIEW" in text


def test_builder_cannot_self_approve_and_two_review_axes_exist():
    text = PROCESS.read_text(encoding="utf-8")
    assert "builder may not be the final reviewer" in text
    assert "Standards review" in text
    assert "Spec review" in text
    assert "fresh reviewer" in text


def test_governor_manifest_makes_completion_gate_mandatory():
    manifest = json.loads((GOVERNOR / "manifest.json").read_text(encoding="utf-8"))
    governance = manifest["governance"]
    assert governance["mandatory_project_code_review"] is True
    assert governance["mandatory_human_review_prompt"] is True
    assert governance["human_review_pending_blocks_done"] is True
    assert governance["scheduled_review_may_dispatch_subagents"] is True
    assert governance["scheduled_review_may_merge_or_deploy"] is False
    assert manifest["scheduled_review"]["definition"] == "cron/icm-code-review.json"


def test_cron_review_is_read_review_only_and_subagent_capable():
    job = json.loads(CRON.read_text(encoding="utf-8"))
    prompt = job["prompt"].lower()
    assert "dispatch fresh independent subagents" in prompt
    assert "do not merge" in prompt
    assert "do not" in prompt and "deploy" in prompt
    assert "builder may not approve itself" in prompt
    assert job["skills"][0] == "icm-engineering-governor"


def test_completion_chain_ends_in_human_review():
    text = (GOVERNOR / "SKILL.md").read_text(encoding="utf-8")
    assert "PROOF -> HUMAN REVIEW" in text
    assert "Hermes MUST explicitly ask the owner to review the code/diff" in text
    assert "HUMAN_REVIEW_RESULT: PENDING" in text
