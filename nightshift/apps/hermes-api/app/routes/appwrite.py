"""Appwrite project routes — provision, list."""
from uuid import UUID
from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.db import get_db
from app.models import AppwriteProject

router = APIRouter(prefix="/appwrite-projects", tags=["appwrite"])

@router.get("")
async def list_projects(db: AsyncSession = Depends(get_db)):
    result = await db.execute(select(AppwriteProject).order_by(AppwriteProject.created_at.desc()))
    return [{"id": str(p.id), "name": p.name, "status": p.status, "endpoint": p.endpoint,
             "project_id": p.project_id, "created_at": p.created_at.isoformat() if p.created_at else None}
            for p in result.scalars()]

@router.post("/provision")
async def provision(body: dict, db: AsyncSession = Depends(get_db)):
    project = AppwriteProject(name=body.get("name", "unnamed"))
    db.add(project)
    await db.commit()
    await db.refresh(project)
    return {"id": str(project.id), "status": "provisioning"}

@router.get("/{project_id}")
async def get_project(project_id: UUID, db: AsyncSession = Depends(get_db)):
    p = await db.get(AppwriteProject, project_id)
    if not p: raise HTTPException(404, "Project not found")
    return p
