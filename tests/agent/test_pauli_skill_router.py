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
