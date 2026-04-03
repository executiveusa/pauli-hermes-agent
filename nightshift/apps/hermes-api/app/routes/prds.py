"""PRD batch routes — generate, list, approve/reject."""
import json
from uuid import UUID
from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession
import redis.asyncio as aioredis

from app.db import get_db
from app.config import settings
from app.models import PRDBatch, PRD

router = APIRouter(tags=["prds"])

@router.get("/prd-batches")
async def list_batches(db: AsyncSession = Depends(get_db)):
    result = await db.execute(select(PRDBatch).order_by(PRDBatch.created_at.desc()).limit(50))
    return [{"id": str(b.id), "status": b.status, "strategy": b.strategy, "prd_count": b.prd_count,
             "created_at": b.created_at.isoformat() if b.created_at else None} for b in result.scalars()]

@router.post("/prd-batches/generate")
async def generate_batch(body: dict, db: AsyncSession = Depends(get_db)):
    batch = PRDBatch(strategy=body.get("strategy", "auto"))
    db.add(batch)
    await db.commit()
    await db.refresh(batch)
    r = aioredis.from_url(settings.redis_url)
    await r.lpush("queue:prd_generation", json.dumps({"batch_id": str(batch.id), "strategy": batch.strategy}))
    await r.close()
    return {"batch_id": str(batch.id), "status": "queued"}

@router.get("/prds")
async def list_prds(batch_id: UUID = None, limit: int = 50, db: AsyncSession = Depends(get_db)):
    q = select(PRD).order_by(PRD.created_at.desc()).limit(limit)
    if batch_id:
        q = q.where(PRD.batch_id == batch_id)
    result = await db.execute(q)
    return [{"id": str(p.id), "title": p.title, "priority": p.priority, "status": p.status,
             "created_at": p.created_at.isoformat() if p.created_at else None} for p in result.scalars()]

@router.post("/prds/{prd_id}/approve")
async def approve_prd(prd_id: UUID, db: AsyncSession = Depends(get_db)):
    prd = await db.get(PRD, prd_id)
    if not prd: raise HTTPException(404, "PRD not found")
    prd.status = "approved"
    await db.commit()
    return {"status": "approved"}

@router.post("/prds/{prd_id}/reject")
async def reject_prd(prd_id: UUID, db: AsyncSession = Depends(get_db)):
    prd = await db.get(PRD, prd_id)
    if not prd: raise HTTPException(404, "PRD not found")
    prd.status = "rejected"
    await db.commit()
    return {"status": "rejected"}
