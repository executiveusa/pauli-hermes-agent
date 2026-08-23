from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SKILL = ROOT / "skills" / "youtube-channel-operator" / "SKILL.md"
ARCHETYPES = ROOT / "skills" / "youtube-channel-operator" / "references" / "story-archetypes.md"
PRODUCTION_TEST = ROOT / "skills" / "youtube-channel-operator" / "references" / "production-test.md"


def read(path: Path) -> str:
    assert path.exists(), f"missing required file: {path}"
    return path.read_text(encoding="utf-8")


def test_locked_stage_order_is_present():
    text = read(SKILL)
    expected = "00 INTAKE → 01 RESEARCH → 02 POSITIONING → 03 CHANNEL SPEC → 04 ASSET GENERATION → 05 BROWSER SETUP → 06 CONTENT STARTER PACK → 07 VERIFY → 08 MONETIZATION READINESS → 09 HANDOFF"
    assert expected in text


def test_locked_story_engine_is_present():
    assert "HOOK → TENSION → PAYOFF → ACTION" in read(SKILL)


def test_all_ten_story_types_are_present_and_interactive_choice_is_experimental():
    text = read(ARCHETYPES)
    names = [
        "Overcame It",
        "Nobody Saw This Coming",
        "The Person Behind It",
        "Before It Was Fixed",
        "One Decision Changed Everything",
        "Why This Matters",
        "Receipts",
        "Challenge",
        "Hidden Opportunity",
        "Interactive Choice",
    ]
    for name in names:
        assert name in text
    assert "EXPERIMENT SHELF" in text
    assert "NOT a core publishing dependency" in text


def test_story_type_is_a_hard_pre_script_gate():
    text = read(ARCHETYPES)
    assert "primary_story_type" in text
    assert 'If `primary_story_type` is missing: **STOP**.' in text


def test_human_gates_cover_consequential_youtube_actions():
    text = read(SKILL).lower()
    required = [
        "public publishing",
        "monetization",
        "adsense",
        "payment/tax/identity",
        "ownership",
    ]
    for phrase in required:
        assert phrase in text


def test_production_test_forbids_public_publish():
    text = read(PRODUCTION_TEST).lower()
    assert "public publishing count is zero" in text
    assert "do not publish them" in text


def test_monetization_must_be_refreshed_from_official_youtube_help():
    text = read(SKILL).lower()
    assert "fetch current official youtube help requirements every run" in text
    assert "eligible" in text
    assert "not_yet_eligible" in text
    assert "action_required" in text


def test_learning_record_preserves_story_type_dimension():
    text = read(SKILL)
    assert "primary_story_type" in text
    assert "hook_variant" in text
    assert "thumbnail_grammar" in text
    assert "keep_or_discard" in text
