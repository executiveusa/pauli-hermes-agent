from pathlib import Path

from agent.pauli_skill_router import (
    build_redacted_env_status,
    resolve_skill_identifier,
    route_skills_for_task,
)


def test_router_loads_github_skills_only_for_repo_task():
    result = route_skills_for_task("Scan this repo and open a PR for the CI fix")
    assert "github" in result["matched_routes"]
    assert "design" not in result["matched_routes"]
    assert "github-auth" in result["selected_skills"]
    assert "codebase-inspection" in result["selected_skills"]


def test_router_rejects_over_budget_skill_load():
    result = route_skills_for_task(
        "Repo scan with CI debugging, memory search, and Vercel deployment failed"
    )
    assert len(result["selected_skills"]) <= result["max_skills_loaded"]
    assert result["skipped_skills"]


def test_redacted_secret_status_never_returns_values():
    status = build_redacted_env_status(
        ["OPENROUTER_API_KEY", "MISSING_KEY"],
        env={"OPENROUTER_API_KEY": "super-secret-value"},
    )
    assert status == {
        "OPENROUTER_API_KEY": "present",
        "MISSING_KEY": "missing",
    }
    assert "super-secret-value" not in str(status)


def test_video_task_defaults_to_non_paid_generation():
    result = route_skills_for_task("Create a video montage and render plan")
    assert "video" in result["matched_routes"]
    assert result["paid_generation_default"] is False


def test_production_deploy_requires_approval():
    result = route_skills_for_task("Deploy this to production on Coolify")
    assert result["approval_required"] is True


def test_design_task_loads_design_skills_only():
    result = route_skills_for_task("Refresh the dashboard UI and improve UX clarity")
    assert "design" in result["matched_routes"]
    assert "pauli-open-design" in result["selected_skills"]
    assert "github-auth" not in result["selected_skills"]


def test_memory_task_uses_search_only_mode():
    result = route_skills_for_task("Search my files and remember the project notes")
    assert "memory" in result["matched_routes"]
    assert result["retrieval_mode"] == "search_only"


def test_custom_skill_identifier_resolves_to_repo_path():
    resolved = resolve_skill_identifier("pauli-open-design")
    assert resolved.endswith(str(Path("skills") / "pauli" / "open-design"))
