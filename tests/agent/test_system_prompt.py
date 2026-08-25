"""Tests for agent/system_prompt.py — context-file cwd wiring."""

from types import SimpleNamespace
from unittest.mock import patch

from agent.system_prompt import build_system_prompt_parts


def _make_agent(**overrides):
    base = dict(
        load_soul_identity=False,
        skip_context_files=False,
        valid_tool_names=[],
        _task_completion_guidance=False,
        _tool_use_enforcement=False,
        _environment_probe=False,
        _kanban_worker_guidance="",
        _memory_store=None,
        _memory_manager=None,
        model="",
        provider="",
        platform="",
        pass_session_id=False,
        session_id="",
    )
    base.update(overrides)
    return SimpleNamespace(**base)


def _captured_context_cwd(agent):
    """The cwd build_system_prompt_parts hands to build_context_files_prompt."""
    captured = {}

    def fake_context_files(cwd=None, skip_soul=False):
        captured["cwd"] = cwd
        return ""

    with (
        patch("run_agent.load_soul_md", return_value=""),
        patch("run_agent.build_nous_subscription_prompt", return_value=""),
        patch("run_agent.build_environment_hints", return_value=""),
        patch("run_agent.build_context_files_prompt", side_effect=fake_context_files),
    ):
        build_system_prompt_parts(agent)
    return captured["cwd"]


class TestContextFileCwd:
    def test_none_when_terminal_cwd_unset(self, monkeypatch):
        # Unset → None, so discovery falls back to the launch dir inside
        # build_context_files_prompt (the local-CLI #19242 contract).
        monkeypatch.delenv("TERMINAL_CWD", raising=False)
        assert _captured_context_cwd(_make_agent()) is None

    def test_configured_dir_when_terminal_cwd_set(self, monkeypatch, tmp_path):
        monkeypatch.setenv("TERMINAL_CWD", str(tmp_path))
        assert _captured_context_cwd(_make_agent()) == tmp_path


def _build_stable_text(agent):
    """Assemble just the stable tier, with the usual heavy calls stubbed out."""
    with (
        patch("run_agent.load_soul_md", return_value=""),
        patch("run_agent.build_nous_subscription_prompt", return_value=""),
        patch("run_agent.build_environment_hints", return_value=""),
        patch("run_agent.build_context_files_prompt", return_value=""),
    ):
        parts = build_system_prompt_parts(agent)
    return parts["stable"]


class TestCodingContextIntegration:
    """agent/coding_context.py wiring into the stable system-prompt tier.

    Covers the coding-posture addition to build_system_prompt_parts: present
    on a cli/tui/acp/desktop surface sitting in a code workspace, absent
    otherwise, and never fatal to prompt assembly if resolution fails.
    """

    def test_coding_brief_present_in_code_workspace(self, tmp_path, monkeypatch):
        (tmp_path / "pyproject.toml").write_text('[project]\nname = "x"\n')
        monkeypatch.delenv("TERMINAL_CWD", raising=False)
        agent = _make_agent(platform="cli", model="anthropic/claude-x")
        with patch("agent.runtime_cwd.resolve_agent_cwd", return_value=tmp_path):
            text = _build_stable_text(agent)
        assert "coding agent pairing with the user" in text

    def test_coding_brief_absent_outside_code_workspace(self, tmp_path, monkeypatch):
        monkeypatch.delenv("TERMINAL_CWD", raising=False)
        agent = _make_agent(platform="cli", model="anthropic/claude-x")
        with patch("agent.runtime_cwd.resolve_agent_cwd", return_value=tmp_path):
            text = _build_stable_text(agent)
        assert "coding agent pairing with the user" not in text

    def test_coding_brief_absent_on_noninteractive_platform(self, tmp_path, monkeypatch):
        (tmp_path / "pyproject.toml").write_text('[project]\nname = "x"\n')
        monkeypatch.delenv("TERMINAL_CWD", raising=False)
        agent = _make_agent(platform="telegram", model="anthropic/claude-x")
        with patch("agent.runtime_cwd.resolve_agent_cwd", return_value=tmp_path):
            text = _build_stable_text(agent)
        assert "coding agent pairing with the user" not in text

    def test_resolution_failure_never_blocks_prompt_build(self, tmp_path, monkeypatch):
        """A raising resolve_runtime_mode must not take down prompt assembly."""
        monkeypatch.delenv("TERMINAL_CWD", raising=False)
        agent = _make_agent(platform="cli", model="anthropic/claude-x")
        with patch(
            "agent.coding_context.resolve_runtime_mode",
            side_effect=RuntimeError("boom"),
        ):
            # Must not raise.
            text = _build_stable_text(agent)
        assert isinstance(text, str)
