from __future__ import annotations

import json
import uuid
from datetime import datetime

from fastapi import FastAPI, HTTPException

from .config import NightShiftConfig
from .db import connect
from .models import (
    AppwriteProjectRequest,
    ApprovalRequest,
    ChatRequest,
    HealthResponse,
    PRDRequest,
    RepoScanRequest,
    SubagentRequest,
)
from .policy import classify_action, requires_approval
from .router import BudgetExceededError, DEFAULT_PROFILES, chat_completion


def _new_id(prefix: str) -> str:
    return f"{prefix}_{uuid.uuid4().hex[:10]}"


def create_app() -> FastAPI:
    cfg = NightShiftConfig()
    app = FastAPI(title="Hermes Night Shift", version="0.1.0")

    @app.get("/health", response_model=HealthResponse)
    async def health() -> HealthResponse:
        services = {
            "db": "ok",
            "redis": "configured" if cfg.redis_url else "missing",
            "temporal": "configured" if cfg.temporal_host else "missing",
            "paperclip": "configured" if cfg.paperclip_url else "missing",
            "composio": "configured" if cfg.composio_url else "missing",
            "appwrite": "configured" if cfg.appwrite_endpoint else "missing",
        }
        return HealthResponse(status="ok", services=services)

    @app.get("/setup/missing-secrets")
    async def missing_secrets() -> dict:
        return {"missing": cfg.missing_secrets}

    @app.get("/dashboard")
    async def dashboard() -> dict:
        # Lightweight cockpit payload for UI clients.
        return {
            "status": "running",
            "modules": {
                "repos": "/repos",
                "prds": "/prds",
                "runs": "/runs",
                "approvals": "/approvals",
                "coding_sessions": "/coding-sessions",
                "browser_runs": "/browser-runs",
                "provider_profiles": "/provider-profiles",
                "subagents": "/subagents",
                "appwrite_projects": "/appwrite-projects",
            },
            "i18n": {
                "es_419": {
                    "missing_secrets_title": "Faltan secretos para encender Hermes Night Shift.",
                    "approval_needed": "Esta acción requiere aprobación humana.",
                }
            },
        }

    @app.post("/repos/scan")
    async def scan_repo(req: RepoScanRequest) -> dict:
        repo_id = _new_id("repo")
        conn = connect(cfg.database_url)
        conn.execute(
            "INSERT INTO repos (id, repo_url, branch, created_at) VALUES (?, ?, ?, ?)",
            (repo_id, req.repo_url, req.branch, datetime.utcnow().isoformat()),
        )
        conn.commit()
        return {"repo_id": repo_id, "status": "cataloged"}

    @app.post("/prds")
    async def create_prd(req: PRDRequest) -> dict:
        prd_id = _new_id("prd")
        conn = connect(cfg.database_url)
        conn.execute(
            "INSERT INTO prds (id, repo_id, title, objective, status, created_at) VALUES (?, ?, ?, ?, ?, ?)",
            (prd_id, req.repo_id, req.title, req.objective, "pending_approval", datetime.utcnow().isoformat()),
        )
        conn.commit()
        return {"prd_id": prd_id, "status": "pending_approval"}

    @app.post("/v1/llm/chat")
    async def llm_chat(req: ChatRequest) -> dict:
        try:
            data = await chat_completion(req.profile, req.model_dump())
        except BudgetExceededError as e:
            raise HTTPException(status_code=402, detail=str(e)) from e
        except Exception as e:  # provider errors
            raise HTTPException(status_code=502, detail=str(e)) from e
        return data

    @app.get("/provider-profiles")
    async def provider_profiles() -> dict:
        return {"profiles": [p.model_dump() for p in DEFAULT_PROFILES.values()]}

    @app.post("/approvals")
    async def request_approval(req: ApprovalRequest) -> dict:
        classification = classify_action(req.action)
        approval_id = _new_id("approval")
        conn = connect(cfg.database_url)
        status = "required" if requires_approval(req.action) else "auto_approved"
        conn.execute(
            "INSERT INTO approvals (id, run_id, action, reason, status, created_at) VALUES (?, ?, ?, ?, ?, ?)",
            (approval_id, req.run_id, req.action, req.reason, status, datetime.utcnow().isoformat()),
        )
        conn.commit()
        return {
            "approval_id": approval_id,
            "status": status,
            "classification": classification,
            "paperclip_gate": classification in {"sensitive_write", "irreversible_action"},
        }

    @app.post("/subagents")
    async def spawn_subagent(req: SubagentRequest) -> dict:
        sid = _new_id("subagent")
        conn = connect(cfg.database_url)
        conn.execute(
            "INSERT INTO subagents (id, name, mission, scope, status, created_at) VALUES (?, ?, ?, ?, ?, ?)",
            (sid, req.name, req.mission, json.dumps(req.scope), "running", datetime.utcnow().isoformat()),
        )
        conn.commit()
        return {"subagent_id": sid, "status": "running"}

    @app.post("/subagents/{subagent_id}/kill")
    async def kill_subagent(subagent_id: str) -> dict:
        conn = connect(cfg.database_url)
        row = conn.execute("SELECT id FROM subagents WHERE id = ?", (subagent_id,)).fetchone()
        if not row:
            raise HTTPException(status_code=404, detail="Sub-agent not found")
        conn.execute("UPDATE subagents SET status = ? WHERE id = ?", ("killed", subagent_id))
        conn.commit()
        return {"subagent_id": subagent_id, "status": "killed"}

    @app.post("/appwrite-projects")
    async def create_appwrite_project(req: AppwriteProjectRequest) -> dict:
        project_id = _new_id("appw")
        conn = connect(cfg.database_url)
        conn.execute(
            "INSERT INTO appwrite_projects (id, name, region, status, created_at) VALUES (?, ?, ?, ?, ?)",
            (project_id, req.name, req.region, "provision_requested", datetime.utcnow().isoformat()),
        )
        conn.commit()
        return {
            "project_id": project_id,
            "status": "provision_requested",
            "next": "Connect APPWRITE_ENDPOINT and APPWRITE_API_KEY to enable live provisioning.",
        }

    @app.get("/mcp")
    async def mcp_probe() -> dict:
        return {
            "name": "hermes-night-shift-mcp",
            "status": "ready",
            "hint": "Use existing Hermes MCP tooling from tools/mcp_tool.py for server registrations.",
        }

    return app


app = create_app()
