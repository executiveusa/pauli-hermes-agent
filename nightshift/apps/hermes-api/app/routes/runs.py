"""Run routes — list, get, steps."""
from uuid import UUID
from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.db import get_db
from app.models import Run, RunStep

router = APIRouter(prefix="/runs", tags=["runs"])

@router.get("")
async def list_runs(status: str = None, limit: int = 50, db: AsyncSession = Depends(get_db)):
    q = select(Run).order_by(Run.started_at.desc()).limit(limit)
    if status:
        q = q.where(Run.status == status)
    result = await db.execute(q)
    return [{"id": str(r.id), "run_type": r.run_type, "status": r.status, "model": r.model,
             "iteration_count": r.iteration_count,
             "started_at": r.started_at.isoformat() if r.started_at else None} for r in result.scalars()]

@router.get("/{run_id}")
async def get_run(run_id: UUID, db: AsyncSession = Depends(get_db)):
    run = await db.get(Run, run_id)
    if not run: raise HTTPException(404, "Run not found")
    return run

@router.get("/{run_id}/steps")
async def get_steps(run_id: UUID, db: AsyncSession = Depends(get_db)):
    result = await db.execute(select(RunStep).where(RunStep.run_id == run_id).order_by(RunStep.step_index))
    return [{"id": str(s.id), "step_index": s.step_index, "tool_name": s.tool_name,
             "tool_output": (s.tool_output or "")[:500], "duration_ms": s.duration_ms,
             "created_at": s.created_at.isoformat() if s.created_at else None} for s in result.scalars()]
