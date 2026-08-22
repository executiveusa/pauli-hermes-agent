from __future__ import annotations

import hashlib
import json
import os
import shutil
import sqlite3
import subprocess
import sys
import uuid
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from hermes_constants import get_hermes_home


STAGES = (
    "discover", "validate", "research", "package", "script", "qa",
    "produce", "approval", "publish", "distribute", "measure", "learn",
)


class FacelessYouTubeRuntime:
    """Small deterministic state/provider layer underneath the agentic Channel OS.

    Reasoning stays in Hermes. This class owns durable jobs, receipts, idempotency,
    provider readiness checks and deterministic external-provider calls.
    """

    def __init__(self, home: Path | None = None, repo_root: Path | None = None):
        self.home = (home or get_hermes_home()).expanduser().resolve()
        self.root = self.home / "faceless-youtube"
        self.artifacts = self.root / "artifacts"
        self.receipts = self.root / "receipts"
        self.db_path = self.root / "runtime.db"
        self.repo_root = (repo_root or Path.cwd()).resolve()
        for directory in (self.root, self.artifacts, self.receipts):
            directory.mkdir(parents=True, exist_ok=True)
            try:
                os.chmod(directory, 0o700)
            except OSError:
                pass
        self._init_db()

    def _connect(self) -> sqlite3.Connection:
        conn = sqlite3.connect(self.db_path)
        conn.row_factory = sqlite3.Row
        return conn

    def _init_db(self) -> None:
        with self._connect() as conn:
            conn.executescript(
                """
                CREATE TABLE IF NOT EXISTS jobs (
                    id TEXT PRIMARY KEY,
                    idempotency_key TEXT UNIQUE NOT NULL,
                    stage TEXT NOT NULL,
                    status TEXT NOT NULL,
                    payload_json TEXT NOT NULL,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL,
                    error TEXT
                );
                CREATE TABLE IF NOT EXISTS receipts (
                    id TEXT PRIMARY KEY,
                    job_id TEXT NOT NULL,
                    kind TEXT NOT NULL,
                    body_json TEXT NOT NULL,
                    created_at TEXT NOT NULL
                );
                """
            )
        try:
            os.chmod(self.db_path, 0o600)
        except OSError:
            pass

    @staticmethod
    def _now() -> str:
        return datetime.now(timezone.utc).isoformat()

    @staticmethod
    def _stable_key(payload: dict[str, Any]) -> str:
        encoded = json.dumps(payload, sort_keys=True, separators=(",", ":")).encode("utf-8")
        return hashlib.sha256(encoded).hexdigest()

    def create_job(self, payload: dict[str, Any], *, force: bool = False) -> dict[str, Any]:
        idem = self._stable_key(payload)
        with self._connect() as conn:
            if not force:
                row = conn.execute("SELECT * FROM jobs WHERE idempotency_key = ?", (idem,)).fetchone()
                if row:
                    return self._row(row)
            job_id = f"yt_{uuid.uuid4().hex[:12]}"
            now = self._now()
            if force:
                idem = f"{idem}:{uuid.uuid4().hex[:8]}"
            conn.execute(
                "INSERT INTO jobs VALUES (?, ?, ?, ?, ?, ?, ?, NULL)",
                (job_id, idem, "discover", "queued", json.dumps(payload), now, now),
            )
            row = conn.execute("SELECT * FROM jobs WHERE id = ?", (job_id,)).fetchone()
        return self._row(row)

    def update_job(self, job_id: str, *, stage: str | None = None, status: str | None = None,
                   error: str | None = None) -> dict[str, Any]:
        if stage is not None and stage not in STAGES:
            raise ValueError(f"unknown stage: {stage}")
        row = self.get_job(job_id)
        if not row:
            raise KeyError(job_id)
        new_stage = stage or row["stage"]
        new_status = status or row["status"]
        with self._connect() as conn:
            conn.execute(
                "UPDATE jobs SET stage=?, status=?, updated_at=?, error=? WHERE id=?",
                (new_stage, new_status, self._now(), error, job_id),
            )
            dbrow = conn.execute("SELECT * FROM jobs WHERE id=?", (job_id,)).fetchone()
        return self._row(dbrow)

    def get_job(self, job_id: str) -> dict[str, Any] | None:
        with self._connect() as conn:
            row = conn.execute("SELECT * FROM jobs WHERE id = ?", (job_id,)).fetchone()
        return self._row(row) if row else None

    def list_jobs(self, limit: int = 20) -> list[dict[str, Any]]:
        with self._connect() as conn:
            rows = conn.execute(
                "SELECT * FROM jobs ORDER BY updated_at DESC LIMIT ?", (max(1, min(limit, 200)),)
            ).fetchall()
        return [self._row(row) for row in rows]

    @staticmethod
    def _row(row: sqlite3.Row) -> dict[str, Any]:
        result = dict(row)
        result["payload"] = json.loads(result.pop("payload_json"))
        return result

    def write_receipt(self, job_id: str, kind: str, body: dict[str, Any]) -> Path:
        receipt_id = f"rcpt_{uuid.uuid4().hex[:12]}"
        created = self._now()
        envelope = {"receipt_id": receipt_id, "job_id": job_id, "kind": kind,
                    "created_at": created, "body": body}
        path = self.receipts / f"{receipt_id}.json"
        path.write_text(json.dumps(envelope, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        try:
            os.chmod(path, 0o600)
        except OSError:
            pass
        with self._connect() as conn:
            conn.execute(
                "INSERT INTO receipts VALUES (?, ?, ?, ?, ?)",
                (receipt_id, job_id, kind, json.dumps(body), created),
            )
        return path

    def doctor(self) -> dict[str, Any]:
        runner = self.repo_root / "integrations" / "hyperagent" / "runner.cjs"
        node = shutil.which("node")
        provider_dir = self.repo_root / "integrations" / "hyperagent" / "node_modules" / "@hyperbrowser" / "agent"
        google_ready = bool(os.getenv("HERMES_YOUTUBE_CLIENT_SECRET_FILE"))
        publish_enabled = os.getenv("HERMES_YOUTUBE_PUBLISH_ENABLED", "0") == "1"
        return {
            "ok": True,
            "runtime_db": str(self.db_path),
            "hyperagent": {
                "runner_exists": runner.exists(),
                "node": node,
                "package_installed": provider_dir.exists(),
                "browser_provider": os.getenv("HYPERAGENT_BROWSER_PROVIDER", "local/default"),
                "ready": bool(runner.exists() and node and provider_dir.exists()),
            },
            "sollo": {
                "ready_for_browser_attempt": bool(runner.exists() and node and provider_dir.exists()),
                "auth": "browser-session-dependent",
            },
            "youtube": {
                "oauth_configured": google_ready,
                "publish_enabled": publish_enabled,
                "publish_guard": "explicit approval + env enable required",
            },
        }

    def hyperagent(self, request: dict[str, Any], timeout: int = 300) -> dict[str, Any]:
        runner = self.repo_root / "integrations" / "hyperagent" / "runner.cjs"
        if not runner.exists():
            raise RuntimeError(f"HyperAgent runner missing: {runner}")
        proc = subprocess.run(
            ["node", str(runner)],
            input=json.dumps(request),
            text=True,
            capture_output=True,
            timeout=timeout,
            cwd=str(runner.parent),
            check=False,
        )
        output = (proc.stdout or "").strip().splitlines()
        if not output:
            raise RuntimeError((proc.stderr or "HyperAgent returned no output").strip())
        try:
            result = json.loads(output[-1])
        except json.JSONDecodeError as exc:
            raise RuntimeError("HyperAgent returned invalid JSON") from exc
        if proc.returncode != 0 or not result.get("ok"):
            raise RuntimeError(result.get("error") or proc.stderr or "HyperAgent failed")
        return result

    def sollo_script(self, brief: dict[str, Any]) -> dict[str, Any]:
        """Drive Sollo through HyperAgent. Never submits publish/account mutations."""
        safe_brief = json.dumps(brief, ensure_ascii=False)
        task = (
            "Open https://sollo.ai and use the already-authorized browser session. "
            "Navigate to the YouTube script/content writer. Create a DRAFT only using this brief: "
            f"{safe_brief}. Do not publish, purchase, upgrade, connect accounts, or change settings. "
            "If login, CAPTCHA, MFA, payment, or an account mutation is required, stop and return AUTH_REQUIRED. "
            "Return the generated script and visible generation metadata as structured text."
        )
        return self.hyperagent({"action": "task", "task": task})


def main() -> int:
    from .cli import main as cli_main
    return cli_main()


if __name__ == "__main__":
    sys.exit(main())
