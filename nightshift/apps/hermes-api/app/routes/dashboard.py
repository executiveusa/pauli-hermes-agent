"""Dashboard bootstrap route — overview stats + missing secrets."""
from fastapi import APIRouter, Depends
from sqlalchemy import func, select
from sqlalchemy.ext.asyncio import AsyncSession

from app.db import get_db
from app.config import settings
from app.models import Repo, Run, Approval, CodingSession, BrowserRun, SubAgent

router = APIRouter(prefix="/dashboard", tags=["dashboard"])

@router.get("/overview")
async def overview(db: AsyncSession = Depends(get_db)):
    repos = (await db.execute(select(func.count(Repo.id)))).scalar() or 0
    runs = (await db.execute(select(func.count(Run.id)).where(Run.status.in_(["running", "starting"])))).scalar() or 0
    approvals = (await db.execute(select(func.count(Approval.id)).where(Approval.decision.is_(None)))).scalar() or 0
    coding = (await db.execute(select(func.count(CodingSession.id)).where(CodingSession.status.in_(["running", "coding", "testing"])))).scalar() or 0
    browser = (await db.execute(select(func.count(BrowserRun.id)).where(BrowserRun.state.in_(["running", "interacting", "navigating"])))).scalar() or 0
    agents = (await db.execute(select(func.count(SubAgent.id)).where(SubAgent.status == "running"))).scalar() or 0

    return {
        "repos_total": repos,
        "runs_active": runs,
        "approvals_pending": approvals,
        "coding_sessions_active": coding,
        "browser_runs_active": browser,
        "subagents_active": agents,
        "missing_secrets": settings.missing_secrets(),
    }

@router.get("/missing-secrets")
async def missing_secrets():
    return settings.missing_secrets()
