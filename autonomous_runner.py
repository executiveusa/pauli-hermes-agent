"""End-to-end autonomous deployment runner."""

from __future__ import annotations

import json
import logging
import os
import time
from typing import Any, Dict, List

import psycopg2

from repo_ingestion import ingest_repos
from run_agent import AIAgent

logger = logging.getLogger(__name__)


def _require_env(name: str) -> str:
    value = os.getenv(name, "").strip()
    if not value:
        raise RuntimeError(f"Missing required environment variable: {name}")
    return value


def _parse_agent_decision(content: str) -> Dict[str, Any]:
    try:
        return json.loads(content)
    except json.JSONDecodeError:
        return {"deploy": False, "reason": f"Non-JSON decision from agent: {content[:200]}"}


def run_autonomous_loop(sleep_seconds: int = 60, max_retries: int = 3) -> None:
    logging.basicConfig(level=logging.INFO)
    database_url = _require_env("DATABASE_URL")

    agent = AIAgent(enabled_toolsets=["deploy_apps"], quiet_mode=True, task_mode="deploy_apps")

    while True:
        try:
            ingested = ingest_repos()
            logger.info("Ingest cycle complete: %s repos processed", ingested)

            with psycopg2.connect(database_url) as conn:
                with conn.cursor() as cur:
                    cur.execute("SELECT id, name, github_url FROM repos WHERE status = 'undeployed' LIMIT 10")
                    rows: List[tuple] = cur.fetchall()

                    for repo_id, name, repo_url in rows:
                        logger.info("Autonomous review: %s (%s)", name, repo_url)
                        prompt = (
                            "Return JSON only with keys deploy (bool), stack (string), and reason (string). "
                            f"Assess repository: {repo_url}. Use tools to analyze and optionally deploy."
                        )

                        attempts = 0
                        decision: Dict[str, Any] = {"deploy": False, "stack": "unknown", "reason": "no decision"}
                        while attempts < max_retries:
                            attempts += 1
                            try:
                                response = agent.chat(prompt)
                                decision = _parse_agent_decision(response)
                                break
                            except Exception:
                                logger.exception("Agent decision failed for %s (attempt %s/%s)", repo_url, attempts, max_retries)
                                if attempts >= max_retries:
                                    decision = {"deploy": False, "reason": "agent failure"}

                        if decision.get("deploy"):
                            status = "deploying"
                        else:
                            status = "skipped"

                        cur.execute("UPDATE repos SET status = %s WHERE id = %s", (status, repo_id))
                        logger.info("Decision for %s: %s (%s)", name, status, decision.get("reason", ""))

                conn.commit()

        except Exception:
            logger.exception("Autonomous cycle failed")

        time.sleep(sleep_seconds)


if __name__ == "__main__":
    run_autonomous_loop()
