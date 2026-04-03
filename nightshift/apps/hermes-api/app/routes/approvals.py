"""Approval routes — list pending, decide."""
import json
from uuid import UUID
from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession
from datetime import datetime, timezone
import redis.asyncio as aioredis

from app.db import get_db
from app.config import settings
from app.models import Approval

router = APIRouter(prefix="/approvals", tags=["approvals"])

@router.get("/pending")
async def list_pending(db: AsyncSession = Depends(get_db)):
    result = await db.execute(select(Approval).where(Approval.decision.is_(None)).order_by(Approval.created_at.desc()))
    return [{"id": str(a.id), "run_id": str(a.run_id) if a.run_id else None,
             "risk_level": a.risk_level, "proposed_command": a.proposed_command,
             "reason": a.reason, "created_at": a.created_at.isoformat() if a.created_at else None}
            for a in result.scalars()]

@router.post("/{approval_id}/decide")
async def decide(approval_id: UUID, body: dict, db: AsyncSession = Depends(get_db)):
    approval = await db.get(Approval, approval_id)
    if not approval: raise HTTPException(404, "Approval not found")
    if approval.decision: raise HTTPException(400, "Already decided")
    decision = body.get("decision", "")
    if decision not in ("approve", "reject", "modify"):
        raise HTTPException(400, "Invalid decision")
    approval.decision = decision
    approval.decided_at = datetime.now(timezone.utc)
    approval.decided_by = "operator"
    await db.commit()
    # Publish decision event
    r = aioredis.from_url(settings.redis_url)
    await r.publish("events:approvals", json.dumps({
        "approval_id": str(approval_id), "run_id": str(approval.run_id), "decision": decision,
    }))
    await r.close()
    return {"status": decision}
