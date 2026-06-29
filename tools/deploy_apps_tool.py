"""Hermes tools for autonomous repository deployment workflows."""

from __future__ import annotations

import json
import os

from tools.registry import registry

from deploy_tool import deploy_repo
from repo_analyzer import analyze_repo
from repo_ingestion import ingest_repos


TOOLSET_NAME = "deploy_apps"


def _check_requirements() -> bool:
    # Allow partial operation; each tool validates its own env at runtime.
    return bool(os.getenv("DATABASE_URL") and os.getenv("GITHUB_TOKEN"))


def _ingest_repos_tool() -> str:
    count = ingest_repos()
    return json.dumps({"success": True, "ingested": count})


def _analyze_repo_tool(repo_url: str) -> str:
    return json.dumps(analyze_repo(repo_url))


def _deploy_repo_tool(repo_url: str, stack: str) -> str:
    return json.dumps(deploy_repo(repo_url, stack))


registry.register(
    name="ingest_repos",
    toolset=TOOLSET_NAME,
    schema={
        "name": "ingest_repos",
        "description": "Pull repositories from GitHub and upsert them into the Postgres repos table.",
        "parameters": {"type": "object", "properties": {}, "required": []},
    },
    handler=lambda args, **kwargs: _ingest_repos_tool(),
    check_fn=_check_requirements,
    requires_env=["GITHUB_TOKEN", "DATABASE_URL"],
)

registry.register(
    name="analyze_repo",
    toolset=TOOLSET_NAME,
    schema={
        "name": "analyze_repo",
        "description": "Analyze a GitHub repository and classify stack, score, and deployability.",
        "parameters": {
            "type": "object",
            "properties": {
                "repo_url": {"type": "string", "description": "GitHub repository URL."},
            },
            "required": ["repo_url"],
        },
    },
    handler=lambda args, **kwargs: _analyze_repo_tool(args.get("repo_url", "")),
    check_fn=_check_requirements,
    requires_env=["GITHUB_TOKEN"],
)

registry.register(
    name="deploy_repo",
    toolset=TOOLSET_NAME,
    schema={
        "name": "deploy_repo",
        "description": "Trigger a deployment in Coolify for a repository and stack.",
        "parameters": {
            "type": "object",
            "properties": {
                "repo_url": {"type": "string", "description": "GitHub repository URL."},
                "stack": {
                    "type": "string",
                    "enum": ["nextjs", "node", "python", "static"],
                    "description": "Detected stack for deployment.",
                },
            },
            "required": ["repo_url", "stack"],
        },
    },
    handler=lambda args, **kwargs: _deploy_repo_tool(args.get("repo_url", ""), args.get("stack", "")),
    check_fn=lambda: bool(os.getenv("COOLIFY_API_KEY") and os.getenv("COOLIFY_BASE_URL")),
    requires_env=["COOLIFY_API_KEY", "COOLIFY_BASE_URL"],
)
