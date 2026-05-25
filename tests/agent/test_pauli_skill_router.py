from pathlib import Path
import importlib


def test_pauli_package_and_router_are_importable():
    pauli = importlib.import_module("pauli")
    router = importlib.import_module("agent.pauli_skill_router")

    assert pauli.__name__ == "pauli"
    assert hasattr(router, "route_task")


def test_pauli_router_blocks_destructive_tasks_and_selects_core_policies():
    from agent.pauli_skill_router import route_task

    result = route_task("please delete the production database", strict=False)

    assert result["blocked"] is True
    assert result["selected_skills"]
    assert "zero-touch-engineer-prime-directive" in result["selected_skills"]
    assert "destructive" in result["block_reason"].lower() or "delete" in result["block_reason"].lower()


def test_pauli_skill_router_config_and_worker_registry_exist():
    router_cfg = Path("config/pauli_skill_router.yaml")
    worker_cfg = Path("config/pauli_worker_registry.yaml")

    assert router_cfg.exists()
    assert worker_cfg.exists()


def test_route_task_uses_config_strict_missing_required_flag(tmp_path):
    from agent.pauli_skill_router import route_task

    router_cfg = tmp_path / "pauli_skill_router.yaml"
    router_cfg.write_text(
        """
version: 1
default_profile: hermes_operator
profiles:
  hermes_operator:
    required_skills:
      - missing-policy-skill
router:
  strict_missing_required: true
""".strip(),
        encoding="utf-8",
    )

    worker_cfg = tmp_path / "pauli_worker_registry.yaml"
    worker_cfg.write_text(
        """
version: 1
workers:
  openclaude:
    enabled: true
safety:
  destructive_keywords: []
""".strip(),
        encoding="utf-8",
    )

    skill_root = tmp_path / "skills" / "pauli"
    skill_root.mkdir(parents=True)

    result = route_task(
        "review this repo",
        strict=None,
        config_path=router_cfg,
        worker_registry_path=worker_cfg,
        skill_root=skill_root,
    )

    assert result["blocked"] is True
    assert result["missing_skills"] == ["missing-policy-skill"]
    assert "missing required skills" in result["block_reason"]


def test_route_task_marks_missing_skills_skipped_when_not_strict(tmp_path):
    from agent.pauli_skill_router import route_task

    router_cfg = tmp_path / "pauli_skill_router.yaml"
    router_cfg.write_text(
        """
version: 1
default_profile: hermes_operator
profiles:
  hermes_operator:
    required_skills:
      - missing-policy-skill
router:
  strict_missing_required: false
""".strip(),
        encoding="utf-8",
    )

    worker_cfg = tmp_path / "pauli_worker_registry.yaml"
    worker_cfg.write_text(
        """
version: 1
workers:
  openclaude:
    enabled: true
safety:
  destructive_keywords: []
""".strip(),
        encoding="utf-8",
    )

    skill_root = tmp_path / "skills" / "pauli"
    skill_root.mkdir(parents=True)

    result = route_task(
        "review this repo",
        strict=None,
        config_path=router_cfg,
        worker_registry_path=worker_cfg,
        skill_root=skill_root,
    )

    assert result["blocked"] is False
    assert result["missing_skills"] == ["missing-policy-skill"]
    assert result["skipped_skills"] == ["missing-policy-skill"]
