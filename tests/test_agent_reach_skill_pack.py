from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SKILL = ROOT / "skills" / "agent-reach"


def test_agent_reach_skill_pack_is_complete():
    required = [
        SKILL / "SKILL.md",
        SKILL / "UPSTREAM.md",
        SKILL / "references" / "setup.md",
        SKILL / "references" / "youtube.md",
        SKILL / "references" / "research-workflow.md",
        SKILL / "references" / "platform-routing.md",
        SKILL / "references" / "usage-prompts.md",
        SKILL / "scripts" / "bootstrap.sh",
        SKILL / "scripts" / "youtube-transcript.sh",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.is_file()]
    assert not missing, f"missing Agent Reach skill files: {missing}"


def test_agent_reach_frontmatter_and_boundaries():
    content = (SKILL / "SKILL.md").read_text(encoding="utf-8")
    assert content.startswith("---\n")
    assert "name: agent-reach" in content
    assert "agent-reach doctor --json" in content
    assert "Read-only by default" in content
    assert "Never expose credentials" in content
    assert "/tmp/agent-reach-" in content


def test_bootstrap_never_uses_sudo():
    bootstrap = (SKILL / "scripts" / "bootstrap.sh").read_text(encoding="utf-8")
    executable_lines = [
        line.strip()
        for line in bootstrap.splitlines()
        if line.strip() and not line.lstrip().startswith("#")
    ]
    assert all(not line.startswith("sudo ") for line in executable_lines)
    assert "b4d52c46c9113cb0f653d6df4cf71ebadf4930ac" in bootstrap


def test_youtube_helper_has_reliability_ladder():
    script = (SKILL / "scripts" / "youtube-transcript.sh").read_text(encoding="utf-8")
    assert "--write-sub" in script
    assert "--write-auto-sub" in script
    assert "agent-reach transcribe" in script
    assert "--no-playlist" in script
    assert "receipt.json" in script
