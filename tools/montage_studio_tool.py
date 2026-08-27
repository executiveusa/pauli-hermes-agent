#!/usr/bin/env python3
"""Hermes operator boundary for the MONTAGE/YAPPY-CLIPZ Studio API.

Hermes may inspect Montage health/capabilities and dispatch named Studio actions.
It never rewrites StudioProject state locally and never exposes bearer credentials.
"""

from __future__ import annotations

import json
import os
import urllib.error
import urllib.parse
import urllib.request
from typing import Any


def _base_url() -> str:
    return os.environ.get("MONTAGE_API_URL", "http://127.0.0.1:8000").rstrip("/")


def _headers(*, json_body: bool = False) -> dict[str, str]:
    headers = {"Accept": "application/json"}
    if json_body:
        headers["Content-Type"] = "application/json"
    tenant = os.environ.get("MONTAGE_TENANT", "").strip()
    if tenant:
        headers["X-Yappy-Tenant"] = tenant
    token = os.environ.get("MONTAGE_API_TOKEN", "").strip()
    if token:
        headers["Authorization"] = f"Bearer {token}"
    return headers


def _request(path: str, *, method: str = "GET", payload: dict[str, Any] | None = None) -> dict[str, Any]:
    data = json.dumps(payload).encode("utf-8") if payload is not None else None
    request = urllib.request.Request(
        f"{_base_url()}{path}",
        data=data,
        headers=_headers(json_body=payload is not None),
        method=method,
    )
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            raw = response.read().decode("utf-8")
            return json.loads(raw) if raw else {"ok": True}
    except urllib.error.HTTPError as exc:
        body = exc.read().decode("utf-8", errors="replace")
        try:
            detail = json.loads(body)
        except json.JSONDecodeError:
            detail = {"detail": body[:1000]}
        return {"ok": False, "status": exc.code, "error": "montage_http_error", "detail": detail}
    except urllib.error.URLError as exc:
        return {"ok": False, "error": "montage_unreachable", "detail": str(exc.reason)}


def montage_studio_tool(
    operation: str,
    action_id: str | None = None,
    action_input: dict[str, Any] | None = None,
    approved: bool = False,
) -> str:
    """Inspect or operate Montage through its stable HTTP contract."""
    operation = str(operation or "").strip().lower()
    if operation == "health":
        return json.dumps(_request("/healthz"), ensure_ascii=False)
    if operation == "capabilities":
        return json.dumps(_request("/api/v1/capabilities"), ensure_ascii=False)
    if operation == "describe":
        if not action_id:
            return json.dumps({"ok": False, "error": "action_id_required"})
        encoded = urllib.parse.quote(action_id, safe="")
        return json.dumps(_request(f"/api/v1/capabilities/{encoded}"), ensure_ascii=False)
    if operation == "run":
        if not action_id:
            return json.dumps({"ok": False, "error": "action_id_required"})
        encoded = urllib.parse.quote(action_id, safe="/")
        payload = {"input": action_input or {}, "approved": bool(approved)}
        return json.dumps(_request(f"/api/v1/actions/{encoded}", method="POST", payload=payload), ensure_ascii=False)
    return json.dumps({"ok": False, "error": "unsupported_operation", "supported": ["health", "capabilities", "describe", "run"]})


def check_montage_requirements() -> bool:
    """Network availability is runtime-dependent; stdlib client always imports."""
    return True


MONTAGE_STUDIO_SCHEMA = {
    "name": "montage_studio",
    "description": (
        "Operate the owner-controlled MONTAGE video studio. Use health/capabilities before dispatching work. "
        "Use describe to inspect an action contract. Use run only for a named Montage action; keep project, "
        "timeline, footage, approvals, and render truth inside Montage. Set approved=true only when the user "
        "has approved an action that Montage marks as consequential or paid."
    ),
    "parameters": {
        "type": "object",
        "properties": {
            "operation": {"type": "string", "enum": ["health", "capabilities", "describe", "run"]},
            "action_id": {"type": "string", "description": "Montage action id for describe/run."},
            "action_input": {"type": "object", "description": "Structured action input defined by Montage."},
            "approved": {"type": "boolean", "default": False, "description": "Explicit approval flag forwarded to Montage."},
        },
        "required": ["operation"],
    },
}


from tools.registry import registry

registry.register(
    name="montage_studio",
    toolset="montage",
    schema=MONTAGE_STUDIO_SCHEMA,
    handler=lambda args, **kw: montage_studio_tool(
        operation=args.get("operation", ""),
        action_id=args.get("action_id"),
        action_input=args.get("action_input"),
        approved=bool(args.get("approved", False)),
    ),
    check_fn=check_montage_requirements,
    emoji="🎬",
)
