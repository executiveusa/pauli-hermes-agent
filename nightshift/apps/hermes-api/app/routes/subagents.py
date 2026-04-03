"""Sub-agent routes — create (scoped), list, stop."""
from uuid import UUID
from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.db import get_db
from app.models import SubAgent
from app.schemas import SubAgentCreate

router = APIRouter(prefix="/subagents", tags=["subagents"])

@router.get("")
async def list_agents(db: AsyncSession = Depends(get_db)):
    result = await db.execute(select(SubAgent).order_by(SubAgent.created_at.desc()).limit(50))
    return [{"id": str(a.id), "label": a.label, "repo_scope": a.repo_scope, "tool_scope": a.tool_scope,
             "time_budget_minutes": a.time_budget_minutes, "status": a.status, "error": a.error,
             "created_at": a.created_at.isoformat() if a.created_at else None} for a in result.scalars()]

@router.post("")
async def create_agent(body: SubAgentCreate, db: AsyncSession = Depends(get_db)):
    if not body.repo_scope:
        raise HTTPException(400, "repo_scope is required (at least one repo)")
    if not body.tool_scope:
        raise HTTPException(400, "tool_scope is required (at least one tool)")
    agent = SubAgent(label=body.label, repo_scope=body.repo_scope, tool_scope=body.tool_scope,
                      time_budget_minutes=body.time_budget_minutes)
    db.add(agent)
    await db.commit()
    await db.refresh(agent)
    return {"id": str(agent.id), "status": "pending"}

@router.get("/{agent_id}")
async def get_agent(agent_id: UUID, db: AsyncSession = Depends(get_db)):
    a = await db.get(SubAgent, agent_id)
    if not a: raise HTTPException(404, "Agent not found")
    return a

@router.post("/{agent_id}/stop")
async def stop_agent(agent_id: UUID, db: AsyncSession = Depends(get_db)):
    a = await db.get(SubAgent, agent_id)
    if not a: raise HTTPException(404, "Agent not found")
    a.status = "stopped"
    await db.commit()
    return {"status": "stopped"}
