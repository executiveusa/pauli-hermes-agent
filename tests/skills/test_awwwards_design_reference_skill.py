"""Static contracts for the optional Awwwards MCP design-reference adapter."""
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
UPSTREAM_PIN = "0560b94e646db37edcdba5249beeabf8c58ba6c7"


def test_awwwards_skill_preserves_design_factory_authority_and_source_rights():
    skill = (ROOT / ".hermes/skills/awwwards-design-reference/SKILL.md").read_text()
    assert "name: awwwards-design-reference" in skill
    assert "pauli-design-factory" in skill
    assert UPSTREAM_PIN in skill
    assert "non-commercial" in skill
    assert "reviewing Awwwards" in skill or "Commercial use requires" in skill
    assert "search_sites" in skill
    assert "create_design_brief" in skill
    assert "verify_build" in skill
    assert "not a new design factory" in skill


def test_awwwards_router_loads_only_for_reference_intent():
    routing = (ROOT / "config/pauli_skill_router.yaml").read_text()
    assert "design_reference:" in routing
    design_reference = routing.split("  design_reference:", 1)[1].split("\n  design:", 1)[0]
    assert "awwwards-design-reference" in design_reference
    assert "Awwwards" in design_reference
    regular_design = routing.split("\n  design:", 1)[1].split("\n  course:", 1)[0]
    assert "awwwards-design-reference" not in regular_design


def test_awwwards_mcp_bootstrap_is_opt_in_and_version_pinned():
    bootstrap = (ROOT / ".hermes/mcp/bootstrap.sh").read_text()
    assert 'if [[ "${ENABLE_AWWWARDS_MCP:-0}" == "1" ]]; then' in bootstrap
    assert "awwwards-mcp@1.20.0" in bootstrap
    assert "node -e" in bootstrap
    assert "Playwright" in bootstrap
    assert "--mcp-stdio" in bootstrap


def test_awwwards_route_resolves_skill_in_isolated_home(tmp_path, monkeypatch):
    from agent.pauli_skill_router import resolve_skill_identifier, route_skills_for_task

    monkeypatch.setenv("HERMES_HOME", str(tmp_path / "isolated-hermes"))
    resolved = Path(resolve_skill_identifier("awwwards-design-reference"))
    assert resolved.is_dir()
    assert (resolved / "SKILL.md").exists()
    result = route_skills_for_task("Find Awwwards design references")
    assert "design_reference" in result["matched_routes"]
    assert str(resolved) in result["resolved_skill_identifiers"]
