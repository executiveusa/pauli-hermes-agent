"""GitHub repository ingestion into Postgres.

Pulls repositories for the authenticated GitHub user and upserts metadata into a
`repos` table.
"""

from __future__ import annotations

import logging
import os
from datetime import datetime
from typing import Any, Dict, List

import psycopg2
import requests

logger = logging.getLogger(__name__)

GITHUB_API_BASE = "https://api.github.com"


def _require_env(name: str) -> str:
    value = os.getenv(name, "").strip()
    if not value:
        raise RuntimeError(f"Missing required environment variable: {name}")
    return value


def _parse_timestamp(value: str | None) -> datetime | None:
    if not value:
        return None
    return datetime.fromisoformat(value.replace("Z", "+00:00"))


def _fetch_all_repos(token: str) -> List[Dict[str, Any]]:
    repos: List[Dict[str, Any]] = []
    page = 1
    headers = {
        "Authorization": f"Bearer {token}",
        "Accept": "application/vnd.github+json",
        "X-GitHub-Api-Version": "2022-11-28",
    }

    while True:
        response = requests.get(
            f"{GITHUB_API_BASE}/user/repos",
            headers=headers,
            params={"per_page": 100, "page": page, "sort": "updated", "direction": "desc"},
            timeout=30,
        )
        response.raise_for_status()
        batch = response.json()
        if not batch:
            break
        repos.extend(batch)
        logger.info("Fetched %s repos from page %s", len(batch), page)
        if len(batch) < 100:
            break
        page += 1

    return repos


def _ensure_table(conn) -> None:
    ddl = """
    CREATE TABLE IF NOT EXISTS repos (
        id SERIAL PRIMARY KEY,
        name TEXT,
        github_url TEXT UNIQUE,
        description TEXT,
        language TEXT,
        default_branch TEXT,
        updated_at TIMESTAMP,
        status TEXT DEFAULT 'undeployed',
        created_at TIMESTAMP DEFAULT NOW()
    );
    """
    with conn.cursor() as cur:
        cur.execute(ddl)
    conn.commit()


def ingest_repos() -> int:
    """Pull all user repositories from GitHub and upsert into Postgres.

    Returns:
        Number of repositories processed.
    """

    github_token = _require_env("GITHUB_TOKEN")
    database_url = _require_env("DATABASE_URL")

    logging.basicConfig(level=logging.INFO)
    logger.info("Starting repository ingestion")

    repos = _fetch_all_repos(github_token)
    if not repos:
        logger.info("No repositories returned by GitHub API")
        return 0

    upsert_sql = """
    INSERT INTO repos (name, github_url, description, language, default_branch, updated_at)
    VALUES (%s, %s, %s, %s, %s, %s)
    ON CONFLICT (github_url)
    DO UPDATE SET
        name = EXCLUDED.name,
        description = EXCLUDED.description,
        language = EXCLUDED.language,
        default_branch = EXCLUDED.default_branch,
        updated_at = EXCLUDED.updated_at;
    """

    with psycopg2.connect(database_url) as conn:
        _ensure_table(conn)
        with conn.cursor() as cur:
            for repo in repos:
                cur.execute(
                    upsert_sql,
                    (
                        repo.get("name"),
                        repo.get("html_url"),
                        repo.get("description"),
                        repo.get("language"),
                        repo.get("default_branch"),
                        _parse_timestamp(repo.get("updated_at")),
                    ),
                )
        conn.commit()

    logger.info("Ingestion complete. Processed %s repos", len(repos))
    return len(repos)


if __name__ == "__main__":
    ingest_repos()
