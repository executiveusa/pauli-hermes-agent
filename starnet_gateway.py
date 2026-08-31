"""Pauli's Place / STARNET gateway mounted by the Hermes API service.

Public requests terminate at the existing HTTPS gateway. This router never exposes
STARNET itself: it talks only to the loopback sidecar on 127.0.0.1.
"""

from __future__ import annotations

import json
import os
import secrets
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

import httpx
from fastapi import APIRouter, Depends, HTTPException, Request
from pydantic import BaseModel

router = APIRouter()

STARNET_BASE = os.getenv("STARNET_LOCAL_URL", "http://127.0.0.1:8787").rstrip("/")
STARNET_ENV_FILE = Path(os.getenv("STARNET_ENV_FILE", "/etc/pauli-starnet.env"))
TERABITHIA_ENV_FILE = Path(os.getenv("TERABITHIA_ENV_FILE", "/opt/pauli-effect/terabithia/.env"))
TASK_DIR = Path(os.getenv("STARNET_GATEWAY_STATE_DIR", "/var/lib/pauli-starnet-gateway/tasks"))


def _read_env_value(path: Path, key: str) -> str:
    try:
        for raw in path.read_text(encoding="utf-8").splitlines():
            line = raw.strip()
            if not line or line.startswith("#") or "=" not in line:
                continue
            name, value = line.split("=", 1)
            if name.strip() == key:
                return value.strip().strip('"').strip("'")
    except OSError:
        pass
    return ""


def _gateway_token() -> str:
    # Reuse the already-established server-to-server control-plane token when
    # available; never return or log it.
    return (
        os.getenv("STARNET_GATEWAY_TOKEN", "").strip()
        or os.getenv("TERABITHIA_API_KEY", "").strip()
        or _read_env_value(TERABITHIA_ENV_FILE, "TERABITHIA_API_KEY")
        or os.getenv("HERMES_API_KEY", "").strip()
    )


def _starnet_key() -> str:
    return (
        os.getenv("STARNET_API_KEY", "").strip()
        or _read_env_value(STARNET_ENV_FILE, "STARNET_API_KEY")
    )


def verify_gateway_bearer(request: Request) -> None:
    expected = _gateway_token()
    if len(expected) < 16:
        raise HTTPException(status_code=503, detail="STARNET gateway bearer is not configured")
    auth = request.headers.get("Authorization", "")
    provided = auth[7:].strip() if auth.lower().startswith("bearer ") else ""
    if not provided or not secrets.compare_digest(provided, expected):
        raise HTTPException(status_code=401, detail="Unauthorized")


def _headers() -> dict[str, str]:
    key = _starnet_key()
    if len(key) < 16:
        raise HTTPException(status_code=503, detail="STARNET local API key is not configured")
    return {
        "Authorization": f"Bearer {key}",
        "Content-Type": "application/json",
        # STARNET intentionally enforces its loopback Host pin.
        "Host": "127.0.0.1:8787",
    }


def _task_path(task_id: str) -> Path:
    safe = "".join(c for c in task_id if c.isalnum() or c in "-_")[:80]
    if not safe:
        raise HTTPException(status_code=400, detail="Invalid task id")
    return TASK_DIR / f"{safe}.json"


def _save_task(record: dict[str, Any]) -> None:
    TASK_DIR.mkdir(parents=True, exist_ok=True)
    path = _task_path(str(record["id"]))
    tmp = path.with_suffix(".tmp")
    tmp.write_text(json.dumps(record, ensure_ascii=False), encoding="utf-8")
    os.replace(tmp, path)


def _load_task(task_id: str) -> dict[str, Any]:
    path = _task_path(task_id)
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        raise HTTPException(status_code=404, detail="Task not found")
    except (OSError, json.JSONDecodeError):
        raise HTTPException(status_code=500, detail="Task receipt unreadable")


class HeisenbergTask(BaseModel):
    task: str
    context: dict[str, Any] = {}


class ApprovalDecision(BaseModel):
    decision: str


async def _starnet_json(method: str, path: str, *, body: dict[str, Any] | None = None, auth: bool = True, timeout: float = 60.0) -> dict[str, Any]:
    headers = _headers() if auth else {"Host": "127.0.0.1:8787"}
    try:
        async with httpx.AsyncClient(timeout=timeout) as client:
            resp = await client.request(method, f"{STARNET_BASE}{path}", headers=headers, json=body)
    except httpx.RequestError as exc:
        raise HTTPException(status_code=503, detail=f"STARNET unreachable: {exc.__class__.__name__}")
    try:
        data = resp.json()
    except ValueError:
        data = {"raw": resp.text[:1000]}
    if resp.status_code >= 400:
        detail = data.get("error") or data.get("detail") or data.get("message") or f"STARNET returned {resp.status_code}"
        if isinstance(detail, dict):
            detail = detail.get("message") or str(detail)
        raise HTTPException(status_code=502 if resp.status_code >= 500 else resp.status_code, detail=str(detail))
    return data


@router.get("/health")
async def health() -> dict[str, Any]:
    """Public liveness only; no secrets or city data."""
    data = await _starnet_json("GET", "/health", auth=False, timeout=5.0)
    return {"status": data.get("status", "ok"), "service": "pauli-starnet-gateway", "starnet": data.get("platform", "starnet-agent")}


@router.get("/v1/city/status")
async def city_status(_: None = Depends(verify_gateway_bearer)) -> dict[str, Any]:
    health_data = await _starnet_json("GET", "/health", auth=False, timeout=5.0)
    models = await _starnet_json("GET", "/v1/models", timeout=10.0)
    citizens = []
    for item in models.get("data", []) if isinstance(models.get("data"), list) else []:
        if not isinstance(item, dict):
            continue
        model_id = str(item.get("id", ""))
        if model_id and model_id != "starnet-agent":
            citizens.append({"id": model_id, "name": model_id, "status": "available"})
    return {
        "city": {"name": "Pauli's Place", "status": "online"},
        "districts": [],
        "citizens": citizens,
        "missions": [],
        "approvals": [],
        "experiments": [],
        "revenue": {"verified": None},
        "costs": {"total": None},
        "health": {"status": health_data.get("status", "unknown"), "version": health_data.get("version")},
        "generatedAt": datetime.now(timezone.utc).isoformat(),
    }


@router.post("/v1/heisenberg/tasks")
async def create_heisenberg_task(payload: HeisenbergTask, _: None = Depends(verify_gateway_bearer)) -> dict[str, Any]:
    task = payload.task.strip()
    if not task:
        raise HTTPException(status_code=400, detail="Task cannot be empty")
    if len(task) > 12000:
        raise HTTPException(status_code=400, detail="Task too long")

    task_id = f"heis-{secrets.token_hex(12)}"
    created_at = datetime.now(timezone.utc).isoformat()
    context = payload.context if isinstance(payload.context, dict) else {}
    system = (
        "You are Heisenberg, the owner-facing First Mate and city manager for Pauli's Place. "
        "Operate through the real STARNET runtime. Be concise, factual, and evidence-first. "
        "Never claim an external action completed without evidence. Preserve owner approval for consequential actions."
    )
    if context:
        system += "\nDispatch context: " + json.dumps(context, ensure_ascii=False)[:6000]

    record: dict[str, Any] = {
        "id": task_id,
        "task_id": task_id,
        "mission_id": task_id,
        "status": "running",
        "task": task,
        "createdAt": created_at,
        "updatedAt": created_at,
        "logs": ["accepted by Pauli/STARNET gateway"],
    }
    _save_task(record)

    try:
        result = await _starnet_json(
            "POST",
            "/v1/chat/completions",
            body={
                "model": "agent",
                "stream": False,
                "messages": [
                    {"role": "system", "content": system},
                    {"role": "user", "content": task},
                ],
            },
            timeout=90.0,
        )
        choices = result.get("choices") if isinstance(result, dict) else None
        text = ""
        if isinstance(choices, list) and choices:
            message = choices[0].get("message", {}) if isinstance(choices[0], dict) else {}
            text = str(message.get("content", "")).strip() if isinstance(message, dict) else ""
        record.update({
            "status": "completed",
            "result": text,
            "response": text,
            "usage": result.get("usage") if isinstance(result, dict) else None,
            "updatedAt": datetime.now(timezone.utc).isoformat(),
            "logs": record["logs"] + ["completed by STARNET agent runtime"],
            "receipt": {"source": "starnet-v1", "task_id": task_id, "completed": True},
        })
        _save_task(record)
        return record
    except HTTPException as exc:
        record.update({
            "status": "failed",
            "error": str(exc.detail),
            "updatedAt": datetime.now(timezone.utc).isoformat(),
            "logs": record["logs"] + ["STARNET dispatch failed"],
            "receipt": {"source": "starnet-v1", "task_id": task_id, "completed": False},
        })
        _save_task(record)
        raise


@router.get("/v1/heisenberg/tasks/{task_id}")
async def read_heisenberg_task(task_id: str, _: None = Depends(verify_gateway_bearer)) -> dict[str, Any]:
    return _load_task(task_id)


@router.post("/v1/approvals/{approval_id}/decision")
async def decide_approval(approval_id: str, payload: ApprovalDecision, _: None = Depends(verify_gateway_bearer)) -> dict[str, Any]:
    decision = payload.decision.strip().lower()
    if decision not in {"approve", "reject"}:
        raise HTTPException(status_code=400, detail="Decision must be approve or reject")
    # STARNET's native consent system must remain the authority. Until a stable
    # headless approval seam is verified, fail closed rather than manufacture an approval.
    raise HTTPException(status_code=501, detail="Remote STARNET approval decisions are not enabled; native consent remains authoritative")
