"""Repo routes — list, get, scan trigger."""
import json
from uuid import UUID
from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession
import redis.asyncio as aioredis

from app.db import get_db
from app.config import settings
from app.models import Repo, RepoProfile

router = APIRouter(prefix="/repos", tags=["repos"])

@router.get("")
async def list_repos(limit: int = 100, db: AsyncSession = Depends(get_db)):
    result = await db.execute(select(Repo).order_by(Repo.created_at.desc()).limit(limit))
    repos = result.scalars().all()
    out = []
    for r in repos:
        d = {"id": str(r.id), "repo_url": r.repo_url, "name": r.name, "source": r.source,
             "last_scanned_at": r.last_scanned_at.isoformat() if r.last_scanned_at else None,
             "created_at": r.created_at.isoformat() if r.created_at else None, "profile": None}
        if r.profile:
            d["profile"] = {"language": r.profile.language, "framework": r.profile.framework,
                            "stars": r.profile.stars, "has_tests": r.profile.has_tests}
        out.append(d)
    return out

@router.get("/count")
async def count_repos(db: AsyncSession = Depends(get_db)):
    from sqlalchemy import func
    count = (await db.execute(select(func.count(Repo.id)))).scalar()
    return {"count": count}

@router.get("/{repo_id}")
async def get_repo(repo_id: UUID, db: AsyncSession = Depends(get_db)):
    repo = await db.get(Repo, repo_id)
    if not repo:
        raise HTTPException(404, "Repo not found")
    return repo

@router.post("/scan-trigger")
async def trigger_scan(db: AsyncSession = Depends(get_db)):
    r = aioredis.from_url(settings.redis_url)
    await r.lpush("queue:repo_scan", json.dumps({"source": "api_trigger"}))
    await r.close()
    return {"status": "scan_queued"}
