from __future__ import annotations


def test_load_pauli_agent_policy_reads_agent_flags():
    from agent.pauli_skill_router import load_pauli_agent_policy

    policy = load_pauli_agent_policy(
        {
            "agent": {
                "pauli_profile": True,
                "pauli_gateway_routing": True,
                "pauli_required_skills_strict": False,
            }
        }
    )

    assert policy == {
        "pauli_profile": True,
        "pauli_gateway_routing": True,
        "pauli_required_skills_strict": False,
    }


def test_build_pauli_turn_context_preloads_selected_skills_and_logs_selection(monkeypatch, caplog):
    from agent import pauli_skill_router as router

    calls: list[list[str]] = []

    def fake_route_task(task, strict=None, **kwargs):
        assert task == "audit this repo"
        assert strict is False
        return {
            "task": task,
            "task_type": "safe",
            "selected_skills": ["zero-touch-engineer-prime-directive", "tool-use-budget-policy"],
            "required_skills": ["zero-touch-engineer-prime-directive", "tool-use-budget-policy"],
            "missing_skills": ["tool-use-budget-policy"],
            "skipped_skills": ["tool-use-budget-policy"],
            "blocked": False,
            "block_reason": "",
        }

    def fake_build_preloaded_skills_prompt(skill_identifiers, task_id=None):
        calls.append(list(skill_identifiers))
        assert task_id == "task-123"
        return (
            "[skill payload]",
            ["zero-touch-engineer-prime-directive"],
            ["tool-use-budget-policy"],
        )

    monkeypatch.setattr(router, "route_task", fake_route_task)
    monkeypatch.setattr(router, "build_preloaded_skills_prompt", fake_build_preloaded_skills_prompt)

    with caplog.at_level("INFO"):
        result = router.build_pauli_turn_context(
            "audit this repo",
            context_prompt="Base context",
            task_id="task-123",
            config={
                "agent": {
                    "pauli_profile": True,
                    "pauli_gateway_routing": True,
                    "pauli_required_skills_strict": False,
                }
            },
        )

    assert calls == [["zero-touch-engineer-prime-directive", "tool-use-budget-policy"]]
    assert result["blocked"] is False
    assert result["combined_ephemeral"] == "Base context\n\n[skill payload]"
    assert result["selected_skills"] == ["zero-touch-engineer-prime-directive", "tool-use-budget-policy"]
    assert result["missing_skills"] == ["tool-use-budget-policy"]
    assert result["skipped_skills"] == ["tool-use-budget-policy"]
    assert "selected_skills" in caplog.text
    assert "missing_skills" in caplog.text


def test_build_pauli_turn_context_blocks_when_strict_required_skill_missing(monkeypatch):
    from agent import pauli_skill_router as router

    def fake_route_task(task, strict=None, **kwargs):
        assert strict is True
        return {
            "task": task,
            "task_type": "safe",
            "selected_skills": ["missing-policy-skill"],
            "required_skills": ["missing-policy-skill"],
            "missing_skills": ["missing-policy-skill"],
            "skipped_skills": ["missing-policy-skill"],
            "blocked": True,
            "block_reason": "missing required skills: missing-policy-skill",
        }

    monkeypatch.setattr(router, "route_task", fake_route_task)

    result = router.build_pauli_turn_context(
        "audit this repo",
        context_prompt="Base context",
        task_id="task-456",
        config={
            "agent": {
                "pauli_profile": True,
                "pauli_gateway_routing": True,
                "pauli_required_skills_strict": True,
            }
        },
    )

    assert result["blocked"] is True
    assert result["combined_ephemeral"] == "Base context"
    assert result["block_reason"] == "missing required skills: missing-policy-skill"


def test_build_pauli_turn_context_skips_preload_when_gate_disabled(monkeypatch):
    from agent import pauli_skill_router as router

    def fake_route_task(task, strict=None, **kwargs):
        return {
            "task": task,
            "task_type": "safe",
            "selected_skills": ["zero-touch-engineer-prime-directive"],
            "required_skills": ["zero-touch-engineer-prime-directive"],
            "missing_skills": [],
            "skipped_skills": [],
            "blocked": False,
            "block_reason": "",
        }

    def fail_if_called(*args, **kwargs):
        raise AssertionError("skill preloading should be skipped when the Pauli gate is disabled")

    monkeypatch.setattr(router, "route_task", fake_route_task)
    monkeypatch.setattr(router, "build_preloaded_skills_prompt", fail_if_called)

    result = router.build_pauli_turn_context(
        "audit this repo",
        context_prompt="Base context",
        task_id="task-789",
        config={
            "agent": {
                "pauli_profile": False,
                "pauli_gateway_routing": False,
                "pauli_required_skills_strict": False,
            }
        },
    )

    assert result["blocked"] is False
    assert result["combined_ephemeral"] == "Base context"
    assert result["loaded_skills"] == []
    assert result["skills_prompt"] == ""
