"""Coding session routes — create, list, cancel."""
import json
from uuid import UUID
from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession
import redis.asyncio as aioredis

from app.db import get_db
from app.config import settings
from app.models import CodingSession
from app.schemas import CodingSessionCreate

router = APIRouter(prefix="/coding-sessions", tags=["coding"])

@router.get("")
async def list_sessions(limit: int = 50, db: AsyncSession = Depends(get_db)):
    result = await db.execute(select(CodingSession).order_by(CodingSession.created_at.desc()).limit(limit))
    return [{"id": str(s.id), "repo_url": s.repo_url, "prompt": s.prompt, "task_type": s.task_type,
             "status": s.status, "branch": s.branch, "error": s.error,
             "created_at": s.created_at.isoformat() if s.created_at else None} for s in result.scalars()]

@router.post("")
async def create_session(body: CodingSessionCreate, db: AsyncSession = Depends(get_db)):
    session = CodingSession(repo_url=body.repo_url, prompt=body.prompt,
                             task_type=body.task_type, provider_profile=body.provider_profile)
    db.add(session)
    await db.commit()
    await db.refresh(session)
    r = aioredis.from_url(settings.redis_url)
    await r.lpush("queue:coding_sessions", json.dumps({
        "session_id": str(session.id), "repo_url": body.repo_url,
        "prompt": body.prompt, "task_type": body.task_type, "provider_profile": body.provider_profile,
    }))
    await r.close()
    return {"id": str(session.id), "status": "queued"}

@router.get("/{session_id}")
async def get_session(session_id: UUID, db: AsyncSession = Depends(get_db)):
    s = await db.get(CodingSession, session_id)
    if not s: raise HTTPException(404, "Session not found")
    return s

@router.post("/{session_id}/cancel")
async def cancel_session(session_id: UUID, db: AsyncSession = Depends(get_db)):
    s = await db.get(CodingSession, session_id)
    if not s: raise HTTPException(404, "Session not found")
    s.status = "cancelled"
    await db.commit()
    return {"status": "cancelled"}
