"""Continuous deployment loop for repositories stored in Postgres."""

from __future__ import annotations

import logging
import os
import time
from typing import Dict, Any

import psycopg2

from deploy_tool import deploy_repo
from repo_analyzer import analyze_repo

logger = logging.getLogger(__name__)


def _require_env(name: str) -> str:
    value = os.getenv(name, "").strip()
    if not value:
        raise RuntimeError(f"Missing required environment variable: {name}")
    return value


def _set_status(cur, repo_id: int, status: str) -> None:
    cur.execute("UPDATE repos SET status = %s WHERE id = %s", (status, repo_id))


def run_loop(sleep_seconds: int = 60) -> None:
    """Poll undeployed repos and deploy viable candidates."""

    logging.basicConfig(level=logging.INFO)
    database_url = _require_env("DATABASE_URL")

    logger.info("Starting deploy loop (interval=%ss)", sleep_seconds)
    while True:
        with psycopg2.connect(database_url) as conn:
            with conn.cursor() as cur:
                cur.execute("SELECT id, name, github_url FROM repos WHERE status = 'undeployed' LIMIT 10")
                rows = cur.fetchall()

                if not rows:
                    logger.info("No undeployed repos found")
                for repo_id, name, repo_url in rows:
                    logger.info("Evaluating repo: %s (%s)", name, repo_url)
                    try:
                        analysis: Dict[str, Any] = analyze_repo(repo_url)
                        score = int(analysis.get("score", 0))
                        stack = analysis.get("stack", "unknown")
                        if score > 60 and analysis.get("deployable"):
                            logger.info("Decision: deploy (score=%s, stack=%s)", score, stack)
                            _set_status(cur, repo_id, "deploying")
                            deployment = deploy_repo(repo_url, stack)
                            final_status = "deployed" if deployment.get("status") in {"queued", "deploying", "deployed"} else "failed"
                            _set_status(cur, repo_id, final_status)
                            logger.info("Result: %s (deployment_id=%s)", final_status, deployment.get("deployment_id"))
                        else:
                            logger.info("Decision: skip (score=%s)", score)
                            _set_status(cur, repo_id, "skipped")
                    except Exception:
                        logger.exception("Failed while processing repo %s", repo_url)
                        _set_status(cur, repo_id, "failed")
            conn.commit()

        time.sleep(sleep_seconds)


if __name__ == "__main__":
    run_loop()
