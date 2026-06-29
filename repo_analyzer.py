"""Heuristic GitHub repository analyzer."""

from __future__ import annotations

import logging
import os
from datetime import datetime, timedelta, timezone
from typing import Dict, Any, List, Set, Tuple
from urllib.parse import urlparse

import requests

logger = logging.getLogger(__name__)
GITHUB_API_BASE = "https://api.github.com"


def _headers() -> dict:
    headers = {
        "Accept": "application/vnd.github+json",
        "X-GitHub-Api-Version": "2022-11-28",
    }
    token = os.getenv("GITHUB_TOKEN", "").strip()
    if token:
        headers["Authorization"] = f"Bearer {token}"
    return headers


def _owner_repo_from_url(repo_url: str) -> Tuple[str, str]:
    parsed = urlparse(repo_url)
    parts = [p for p in parsed.path.split("/") if p]
    if len(parts) < 2:
        raise ValueError(f"Invalid GitHub repository URL: {repo_url}")
    return parts[0], parts[1].removesuffix(".git")


def _github_get(path: str, params: dict | None = None) -> Any:
    response = requests.get(f"{GITHUB_API_BASE}{path}", headers=_headers(), params=params, timeout=30)
    response.raise_for_status()
    return response.json()


def _fetch_file_tree(owner: str, repo: str, branch: str) -> Set[str]:
    tree = _github_get(f"/repos/{owner}/{repo}/git/trees/{branch}", params={"recursive": 1})
    return {item.get("path", "") for item in tree.get("tree", [])}


def _readme_exists(owner: str, repo: str) -> bool:
    resp = requests.get(
        f"{GITHUB_API_BASE}/repos/{owner}/{repo}/readme",
        headers=_headers(),
        timeout=30,
    )
    return resp.status_code == 200


def _is_recent_commit(owner: str, repo: str, days: int = 180) -> bool:
    commits = _github_get(f"/repos/{owner}/{repo}/commits", params={"per_page": 1})
    if not commits:
        return False
    date_str = commits[0].get("commit", {}).get("committer", {}).get("date")
    if not date_str:
        return False
    commit_dt = datetime.fromisoformat(date_str.replace("Z", "+00:00"))
    return commit_dt >= datetime.now(timezone.utc) - timedelta(days=days)


def _classify_stack(paths: Set[str]) -> str:
    has_package = "package.json" in paths
    has_next = "next.config.js" in paths or "next.config.mjs" in paths
    has_requirements = "requirements.txt" in paths or "pyproject.toml" in paths
    has_index_html = "index.html" in paths or "public/index.html" in paths

    if has_next:
        return "nextjs"
    if has_package:
        return "node"
    if has_requirements:
        return "python"
    if has_index_html:
        return "static"
    return "unknown"


def analyze_repo(repo_url: str) -> Dict[str, Any]:
    """Analyze a repository and classify deployability using heuristics."""

    logging.basicConfig(level=logging.INFO)
    owner, repo = _owner_repo_from_url(repo_url)
    logger.info("Analyzing repo %s/%s", owner, repo)

    metadata = _github_get(f"/repos/{owner}/{repo}")
    default_branch = metadata.get("default_branch", "main")
    paths = _fetch_file_tree(owner, repo, default_branch)

    has_readme = _readme_exists(owner, repo)
    has_dockerfile = "Dockerfile" in paths
    has_package = "package.json" in paths
    has_requirements = "requirements.txt" in paths or "pyproject.toml" in paths
    has_index_html = "index.html" in paths or "public/index.html" in paths
    has_entrypoint = has_dockerfile or has_package or has_requirements or has_index_html

    stack = _classify_stack(paths)
    recent = _is_recent_commit(owner, repo)

    score = 0
    if has_readme:
        score += 10
    if recent:
        score += 10
    if has_entrypoint:
        score += 30

    config_hits = sum([has_dockerfile, has_package, has_requirements, has_index_html])
    if config_hits >= 2:
        score += 20

    deployable = stack != "unknown" and has_entrypoint
    if deployable:
        score += 30

    score = max(0, min(100, score))

    result: Dict[str, Any] = {
        "stack": stack,
        "score": score,
        "deployable": deployable,
        "signals": {
            "readme": has_readme,
            "recent_commit": recent,
            "has_dockerfile": has_dockerfile,
            "has_package_json": has_package,
            "has_requirements_txt": has_requirements,
            "has_index_html": has_index_html,
        },
    }
    logger.info("Analysis result for %s: %s", repo_url, result)
    return result


if __name__ == "__main__":
    import sys

    if len(sys.argv) < 2:
        raise SystemExit("Usage: python repo_analyzer.py <repo_url>")
    print(analyze_repo(sys.argv[1]))
