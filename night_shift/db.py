from __future__ import annotations

import sqlite3
from pathlib import Path

SCHEMA = """
CREATE TABLE IF NOT EXISTS repos (id TEXT PRIMARY KEY, repo_url TEXT, branch TEXT, created_at TEXT);
CREATE TABLE IF NOT EXISTS prds (id TEXT PRIMARY KEY, repo_id TEXT, title TEXT, objective TEXT, status TEXT, created_at TEXT);
CREATE TABLE IF NOT EXISTS runs (id TEXT PRIMARY KEY, kind TEXT, status TEXT, details TEXT, created_at TEXT);
CREATE TABLE IF NOT EXISTS approvals (id TEXT PRIMARY KEY, run_id TEXT, action TEXT, reason TEXT, status TEXT, created_at TEXT);
CREATE TABLE IF NOT EXISTS coding_sessions (id TEXT PRIMARY KEY, run_id TEXT, repo_id TEXT, status TEXT, created_at TEXT);
CREATE TABLE IF NOT EXISTS browser_runs (id TEXT PRIMARY KEY, run_id TEXT, target_url TEXT, status TEXT, created_at TEXT);
CREATE TABLE IF NOT EXISTS provider_profiles (name TEXT PRIMARY KEY, base_url TEXT, model TEXT, api_key_env TEXT, max_cost_per_1k REAL, budget_cap_usd REAL);
CREATE TABLE IF NOT EXISTS subagents (id TEXT PRIMARY KEY, name TEXT, mission TEXT, scope TEXT, status TEXT, created_at TEXT);
CREATE TABLE IF NOT EXISTS appwrite_projects (id TEXT PRIMARY KEY, name TEXT, region TEXT, status TEXT, created_at TEXT);
"""


def _resolve_sqlite_path(database_url: str) -> Path:
    if database_url.startswith("sqlite:///"):
        return Path(database_url.removeprefix("sqlite:///"))
    return Path("night_shift.db")


def connect(database_url: str) -> sqlite3.Connection:
    path = _resolve_sqlite_path(database_url)
    path.parent.mkdir(parents=True, exist_ok=True)
    conn = sqlite3.connect(path)
    conn.row_factory = sqlite3.Row
    conn.executescript(SCHEMA)
    return conn
