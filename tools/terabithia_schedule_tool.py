#!/usr/bin/env python3
"""Canonical scheduling client for Hermes/Cosmos -> Terabithia.

Hermes remains the natural-language interpreter/orchestrator. Terabithia owns
schedule identity, policy, persistence, audit, and canonical mission dispatch.
The existing native ``cronjob`` tool remains available for backwards
compatibility, but owner-facing cloud schedules should prefer this tool.
"""

from __future__ import annotations

import json
import os
import re
from datetime import timedelta
from typing import Any, Dict, List, Optional

import requests

from cron.jobs import parse_schedule
from hermes_time import now as hermes_now
from tools.registry import registry, tool_error


def _configured() -> bool:
    return bool(os.getenv("TERABITHIA_URL") and os.getenv("TERABITHIA_API_KEY"))


def _base_url() -> str:
    raw = (os.getenv("TERABITHIA_URL") or "").strip().rstrip("/")
    if not raw:
        raise ValueError("TERABITHIA_URL is not configured")
    return raw


def _headers() -> Dict[str, str]:
    token = (os.getenv("TERABITHIA_API_KEY") or "").strip()
    if not token:
        raise ValueError("TERABITHIA_API_KEY is not configured")
    return {
        "Authorization": f"Bearer {token}",
        "Accept": "application/json",
        "Content-Type": "application/json",
    }


def _request(method: str, path: str, body: Optional[Dict[str, Any]] = None) -> Any:
    timeout = float(os.getenv("TERABITHIA_TIMEOUT_SECONDS", "20"))
    response = requests.request(
        method,
        f"{_base_url()}{path}",
        headers=_headers(),
        json=body,
        timeout=timeout,
    )
    try:
        payload = response.json()
    except Exception:
        payload = {"message": response.text[:500]}
    if not response.ok:
        message = payload.get("message") or payload.get("error") or response.reason
        raise RuntimeError(f"Terabithia returned HTTP {response.status_code}: {message}")
    return payload


def _next_daily(hour: int, minute: int):
    current = hermes_now()
    candidate = current.replace(hour=hour, minute=minute, second=0, microsecond=0)
    if candidate <= current:
        candidate += timedelta(days=1)
    return candidate


def _next_weekly(hour: int, minute: int, cron_weekday: int):
    current = hermes_now()
    # cron: Sunday=0/7, Monday=1 ... Saturday=6
    cron_weekday = 0 if cron_weekday == 7 else cron_weekday
    py_weekday = (cron_weekday - 1) % 7
    days = (py_weekday - current.weekday()) % 7
    candidate = (current + timedelta(days=days)).replace(
        hour=hour, minute=minute, second=0, microsecond=0
    )
    if candidate <= current:
        candidate += timedelta(days=7)
    return candidate


def _canonical_recurrence(schedule: str) -> Dict[str, Any]:
    """Translate Hermes schedule syntax to Terabithia's canonical recurrence.

    Terabithia currently stores one-shot and fixed-interval recurrences. Common
    cron forms are losslessly reduced to fixed intervals with an explicit
    start_at. Complex calendar expressions fail closed rather than silently
    changing semantics.
    """
    parsed = parse_schedule(schedule)
    kind = parsed.get("kind")
    if kind == "once":
        return {"kind": "once", "run_at": parsed["run_at"]}
    if kind == "interval":
        return {"kind": "interval", "every_seconds": int(parsed["minutes"]) * 60}
    if kind != "cron":
        raise ValueError(f"Unsupported schedule kind: {kind}")

    expr = str(parsed.get("expr") or "").strip()
    fields = expr.split()
    if len(fields) != 5:
        raise ValueError("Canonical cloud scheduling currently supports five-field cron expressions only")
    minute, hour, day, month, weekday = fields

    # */N * * * * -> every N minutes
    step = re.fullmatch(r"\*/(\d+)", minute)
    if step and hour == day == month == weekday == "*":
        every = int(step.group(1))
        if every <= 0:
            raise ValueError("Cron interval must be positive")
        return {"kind": "interval", "every_seconds": every * 60}

    # M * * * * -> hourly at minute M
    if minute.isdigit() and hour == day == month == weekday == "*":
        m = int(minute)
        if not 0 <= m <= 59:
            raise ValueError("Cron minute out of range")
        current = hermes_now()
        start = current.replace(minute=m, second=0, microsecond=0)
        if start <= current:
            start += timedelta(hours=1)
        return {"kind": "interval", "every_seconds": 3600, "start_at": start.isoformat()}

    # M H * * * -> daily
    if minute.isdigit() and hour.isdigit() and day == month == weekday == "*":
        m, h = int(minute), int(hour)
        if not (0 <= m <= 59 and 0 <= h <= 23):
            raise ValueError("Cron hour/minute out of range")
        return {
            "kind": "interval",
            "every_seconds": 86400,
            "start_at": _next_daily(h, m).isoformat(),
        }

    # M H * * D -> weekly on one weekday
    if minute.isdigit() and hour.isdigit() and day == month == "*" and weekday.isdigit():
        m, h, w = int(minute), int(hour), int(weekday)
        if not (0 <= m <= 59 and 0 <= h <= 23 and 0 <= w <= 7):
            raise ValueError("Cron field out of range")
        return {
            "kind": "interval",
            "every_seconds": 604800,
            "start_at": _next_weekly(h, m, w).isoformat(),
        }

    raise ValueError(
        "This calendar cron expression cannot yet be represented losslessly by "
        "Terabithia's canonical scheduler. Use a one-shot, every-N interval, "
        "hourly, daily, or single-weekday weekly schedule."
    )


def _channel_context() -> Dict[str, str]:
    try:
        from gateway.session_context import get_session_env

        platform = (get_session_env("HERMES_SESSION_PLATFORM") or "").lower()
        chat_id = get_session_env("HERMES_SESSION_CHAT_ID") or ""
        thread_id = get_session_env("HERMES_SESSION_THREAD_ID") or ""
        if platform == "telegram":
            conversation = f"telegram:{chat_id}" + (f":{thread_id}" if thread_id else "")
            return {
                "type": "telegram",
                "conversation_id": conversation,
                "reply_target": conversation,
            }
    except Exception:
        pass
    return {"type": "internal"}


def terabithia_schedule(
    action: str,
    schedule_id: Optional[str] = None,
    prompt: Optional[str] = None,
    schedule: Optional[str] = None,
    name: Optional[str] = None,
    skills: Optional[List[str]] = None,
    desired_outcome: Optional[str] = None,
) -> str:
    """Manage canonical cloud schedules through Terabithia."""
    try:
        normalized = (action or "").strip().lower()
        if normalized == "list":
            return json.dumps(_request("GET", "/api/v1/schedules"), indent=2)

        if normalized == "create":
            if not prompt or not prompt.strip():
                return tool_error("prompt is required for create", success=False)
            if not schedule or not schedule.strip():
                return tool_error("schedule is required for create", success=False)
            refs = [f"skill://{item.strip()}" for item in (skills or []) if item and item.strip()]
            body = {
                "template": {
                    "intent": prompt.strip(),
                    "desired_outcome": (desired_outcome or prompt).strip(),
                    "context_refs": refs + ([f"schedule-name://{name.strip()}"] if name and name.strip() else []),
                    "constraints": ["owner_facing_schedule", "hermes_orchestrated"],
                    "channel": _channel_context(),
                },
                "recurrence": _canonical_recurrence(schedule),
            }
            result = _request("POST", "/api/v1/schedules", body)
            return json.dumps({"success": True, "schedule": result}, indent=2)

        if not schedule_id:
            return tool_error(f"schedule_id is required for action '{normalized}'", success=False)
        encoded = requests.utils.quote(str(schedule_id), safe="")

        if normalized == "get":
            return json.dumps(_request("GET", f"/api/v1/schedules/{encoded}"), indent=2)
        if normalized in {"pause", "resume", "cancel", "run"}:
            return json.dumps(
                _request("POST", f"/api/v1/schedules/{encoded}/{normalized}"),
                indent=2,
            )
        if normalized == "update":
            body: Dict[str, Any] = {}
            if prompt is not None or desired_outcome is not None or skills is not None:
                template: Dict[str, Any] = {}
                if prompt is not None:
                    template["intent"] = prompt.strip()
                if desired_outcome is not None:
                    template["desired_outcome"] = desired_outcome.strip()
                if skills is not None:
                    template["context_refs"] = [
                        f"skill://{item.strip()}" for item in skills if item and item.strip()
                    ]
                body["template"] = template
            if schedule is not None:
                body["recurrence"] = _canonical_recurrence(schedule)
            if not body:
                return tool_error("No updates provided", success=False)
            return json.dumps(
                _request("PATCH", f"/api/v1/schedules/{encoded}", body),
                indent=2,
            )

        return tool_error(f"Unknown Terabithia schedule action '{action}'", success=False)
    except Exception as exc:
        # Never include auth material in errors. requests does not include our
        # Authorization header in the messages above, and we return only the
        # sanitized exception text.
        return tool_error(str(exc), success=False)


TERABITHIA_SCHEDULE_SCHEMA = {
    "name": "terabithia_schedule",
    "description": (
        "Manage owner-facing persistent cloud schedules through the canonical "
        "Terabithia control plane. Prefer this over local cronjob for work that "
        "must remain visible/auditable across Command Center and Telegram. "
        "Supports one-shot/duration, every-N intervals, and common hourly/daily/weekly cron forms."
    ),
    "parameters": {
        "type": "object",
        "properties": {
            "action": {
                "type": "string",
                "enum": ["create", "list", "get", "update", "pause", "resume", "cancel", "run"],
            },
            "schedule_id": {"type": "string"},
            "prompt": {"type": "string"},
            "schedule": {
                "type": "string",
                "description": "Examples: 30m, every 2h, 0 9 * * *, 2026-09-01T09:00:00",
            },
            "name": {"type": "string"},
            "skills": {"type": "array", "items": {"type": "string"}},
            "desired_outcome": {"type": "string"},
        },
        "required": ["action"],
    },
}

registry.register(
    name="terabithia_schedule",
    toolset="cronjob",
    schema=TERABITHIA_SCHEDULE_SCHEMA,
    handler=lambda args, **kw: terabithia_schedule(
        action=args.get("action", ""),
        schedule_id=args.get("schedule_id"),
        prompt=args.get("prompt"),
        schedule=args.get("schedule"),
        name=args.get("name"),
        skills=args.get("skills"),
        desired_outcome=args.get("desired_outcome"),
    ),
    check_fn=_configured,
    requires_env=["TERABITHIA_URL", "TERABITHIA_API_KEY"],
    emoji="🛰️",
)
