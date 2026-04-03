"""Agent MAXX — Worker Coder: Coding session runner."""

import asyncio
import json
import logging
import os
import subprocess
import uuid
from datetime import datetime, timezone
from enum import Enum
from pathlib import Path
from typing import Optional

import httpx
import redis.asyncio as aioredis

logger = logging.getLogger(__name__)

WORKSPACES_ROOT = Path(os.getenv("WORKSPACES_ROOT", "/data/workspaces"))
ARTIFACTS_ROOT = Path(os.getenv("ARTIFACTS_ROOT", "/data/artifacts"))
PROVIDER_ROUTER_URL = os.getenv("PROVIDER_ROUTER_URL", "http://provider-router:8080")
REDIS_URL = os.getenv("REDIS_URL", "redis://redis:6379/0")
HERMES_API_URL = os.getenv("HERMES_API_URL", "http://hermes-api:8000")


class SessionState(str, Enum):
    queued = "queued"
    starting = "starting"
    cloning = "cloning"
    analyzing = "analyzing"
    coding = "coding"
    testing = "testing"
    committing = "committing"
    completed = "completed"
    failed = "failed"
    cancelled = "cancelled"


class CodingSessionRunner:
    """Runs a single coding session: clone -> analyze -> code -> test -> commit."""

    def __init__(self, session_id: str, job_data: dict):
        self.session_id = session_id
        self.job = job_data
        self.workspace: Optional[Path] = None
        self.state = SessionState.queued
        self.steps: list[dict] = []

    async def run(self):
        try:
            self.state = SessionState.starting
            await self._report("starting")

            self.state = SessionState.cloning
            await self._report("cloning")
            self.workspace = await self._clone_repo()
            self._log_step("clone", "success", f"Cloned to {self.workspace}")

            self.state = SessionState.analyzing
            await self._report("analyzing")
            profile = await self._analyze_repo()
            self._log_step("analyze", "success", json.dumps(profile, indent=2))

            self.state = SessionState.coding
            await self._report("coding")
            result = await self._execute_task(profile)
            self._log_step("code", "success", result.get("summary", ""))

            self.state = SessionState.testing
            await self._report("testing")
            test_result = await self._run_tests(profile)
            self._log_step("test", "success" if test_result["passed"] else "failed", test_result.get("output", ""))

            self.state = SessionState.committing
            await self._report("committing")
            commit_info = await self._commit_and_push()
            self._log_step("commit", "success", json.dumps(commit_info))

            self.state = SessionState.completed
            await self._report("completed")

        except Exception as e:
            self.state = SessionState.failed
            self._log_step("error", "failed", str(e))
            await self._report("failed", error=str(e))
            logger.exception(f"Session {self.session_id} failed: {e}")

    async def _clone_repo(self) -> Path:
        repo_url = self.job["repo_url"]
        branch = self.job.get("base_branch", "main")
        ws = WORKSPACES_ROOT / f"session_{self.session_id}"
        ws.mkdir(parents=True, exist_ok=True)
        proc = subprocess.run(
            ["git", "clone", "--depth", "10", "--branch", branch, repo_url, str(ws / "repo")],
            capture_output=True, text=True, timeout=120,
            env={**os.environ, "GIT_TERMINAL_PROMPT": "0"},
        )
        if proc.returncode != 0:
            raise RuntimeError(f"Clone failed: {proc.stderr[:500]}")
        return ws / "repo"

    async def _analyze_repo(self) -> dict:
        repo_dir = self.workspace
        profile = {"language": None, "framework": None, "has_tests": False, "test_command": None}
        files = [f.name for f in repo_dir.iterdir()] if repo_dir.exists() else []
        if "package.json" in files:
            profile["language"] = "javascript"
            try:
                pkg = json.loads((repo_dir / "package.json").read_text())
                scripts = pkg.get("scripts", {})
                if "test" in scripts:
                    profile["has_tests"] = True
                    profile["test_command"] = "npm test"
                deps = {**pkg.get("dependencies", {}), **pkg.get("devDependencies", {})}
                if "next" in deps: profile["framework"] = "next.js"
                elif "react" in deps: profile["framework"] = "react"
            except Exception: pass
        elif "pyproject.toml" in files or "requirements.txt" in files:
            profile["language"] = "python"
            if (repo_dir / "tests").is_dir():
                profile["has_tests"] = True
                profile["test_command"] = "python -m pytest tests/ -q"
        elif "Cargo.toml" in files:
            profile["language"] = "rust"
            profile["has_tests"] = True
            profile["test_command"] = "cargo test"
        return profile

    async def _execute_task(self, profile: dict) -> dict:
        prompt = self.job.get("prompt", "")
        context = f"Repo: {self.job['repo_url']}\nLanguage: {profile.get('language')}\nTask: {prompt}\n"
        async with httpx.AsyncClient(timeout=300.0) as client:
            resp = await client.post(f"{PROVIDER_ROUTER_URL}/v1/llm/chat", json={
                "provider_profile_id": self.job.get("provider_profile", "balanced"),
                "messages": [
                    {"role": "system", "content": "You are a senior software engineer. Execute the coding task."},
                    {"role": "user", "content": context},
                ],
                "max_tokens": 8192,
            })
        if resp.status_code != 200:
            return {"summary": f"Provider error: {resp.status_code}"}
        return {"summary": resp.json().get("response_text", "")[:500]}

    async def _run_tests(self, profile: dict) -> dict:
        if not profile.get("has_tests") or not profile.get("test_command"):
            return {"passed": True, "output": "No tests — skipped"}
        proc = subprocess.run(profile["test_command"].split(), cwd=str(self.workspace),
                              capture_output=True, text=True, timeout=300)
        return {"passed": proc.returncode == 0, "output": (proc.stdout + proc.stderr)[:2000]}

    async def _commit_and_push(self) -> dict:
        branch = f"maxx/session-{self.session_id[:8]}"
        for cmd in [["git", "checkout", "-b", branch], ["git", "add", "-A"],
                    ["git", "commit", "-m", f"[Agent MAXX] {self.job.get('prompt', '')[:80]}"]]:
            subprocess.run(cmd, cwd=str(self.workspace), capture_output=True, text=True)
        return {"branch": branch}

    def _log_step(self, name: str, status: str, output: str):
        self.steps.append({"step_name": name, "status": status, "output": output[:2000],
                           "timestamp": datetime.now(timezone.utc).isoformat()})

    async def _report(self, status: str, error: str = None):
        try:
            async with httpx.AsyncClient(timeout=10.0) as client:
                await client.patch(f"{HERMES_API_URL}/api/coding-sessions/{self.session_id}",
                                   json={"status": status, "error": error})
        except Exception: pass


async def consume_coding_jobs():
    r = aioredis.from_url(REDIS_URL)
    logger.info("🐕 Worker-Coder listening on queue:coding_sessions")
    while True:
        try:
            result = await r.brpop("queue:coding_sessions", timeout=30)
            if result is None: continue
            _, raw = result
            job = json.loads(raw)
            session_id = job.get("session_id", str(uuid.uuid4()))
            logger.info(f"Processing coding session {session_id}")
            runner = CodingSessionRunner(session_id, job)
            await runner.run()
        except Exception as e:
            logger.exception(f"Job consumer error: {e}")
            await asyncio.sleep(5)


if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO)
    asyncio.run(consume_coding_jobs())
