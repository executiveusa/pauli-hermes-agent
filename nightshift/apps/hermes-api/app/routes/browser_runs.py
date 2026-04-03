"""Browser run routes — create, list, pause/resume/cancel, artifacts."""
import json
from uuid import UUID
from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession
import redis.asyncio as aioredis

from app.db import get_db
from app.config import settings
from app.models import BrowserRun, BrowserArtifact
from app.schemas import BrowserRunCreate

router = APIRouter(prefix="/browser-runs", tags=["browser"])

@router.get("")
async def list_runs(limit: int = 50, db: AsyncSession = Depends(get_db)):
    result = await db.execute(select(BrowserRun).order_by(BrowserRun.created_at.desc()).limit(limit))
    return [{"id": str(b.id), "start_url": b.start_url, "state": b.state,
             "artifact_count": b.artifact_count, "error": b.error,
             "created_at": b.created_at.isoformat() if b.created_at else None} for b in result.scalars()]

@router.post("")
async def create_run(body: BrowserRunCreate, db: AsyncSession = Depends(get_db)):
    run = BrowserRun(start_url=body.start_url, steps=body.steps)
    db.add(run)
    await db.commit()
    await db.refresh(run)
    r = aioredis.from_url(settings.redis_url)
    await r.lpush("queue:browser_runs", json.dumps({
        "run_id": str(run.id), "start_url": body.start_url, "steps": body.steps,
    }))
    await r.close()
    return {"id": str(run.id), "state": "queued"}

@router.get("/{run_id}")
async def get_run(run_id: UUID, db: AsyncSession = Depends(get_db)):
    run = await db.get(BrowserRun, run_id)
    if not run: raise HTTPException(404, "Run not found")
    return run

@router.get("/{run_id}/artifacts")
async def get_artifacts(run_id: UUID, db: AsyncSession = Depends(get_db)):
    result = await db.execute(select(BrowserArtifact).where(BrowserArtifact.browser_run_id == run_id))
    return [{"id": str(a.id), "kind": a.kind, "label": a.label,
             "created_at": a.created_at.isoformat() if a.created_at else None} for a in result.scalars()]

async def _control(run_id: UUID, action: str):
    r = aioredis.from_url(settings.redis_url)
    await r.publish("browser:control", json.dumps({"run_id": str(run_id), "action": action}))
    await r.close()
    return {"status": action}

@router.post("/{run_id}/pause")
async def pause(run_id: UUID): return await _control(run_id, "pause")

@router.post("/{run_id}/resume")
async def resume(run_id: UUID): return await _control(run_id, "resume")

@router.post("/{run_id}/cancel")
async def cancel(run_id: UUID): return await _control(run_id, "cancel")
