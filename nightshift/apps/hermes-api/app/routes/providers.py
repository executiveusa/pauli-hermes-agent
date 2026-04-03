"""Provider profile routes — CRUD."""
from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.db import get_db
from app.models import ProviderProfile

router = APIRouter(prefix="/providers", tags=["providers"])

@router.get("")
async def list_providers(db: AsyncSession = Depends(get_db)):
    result = await db.execute(select(ProviderProfile).order_by(ProviderProfile.id))
    return [{"id": p.id, "provider": p.provider, "model": p.model,
             "purpose": p.purpose, "is_active": p.is_active} for p in result.scalars()]

@router.get("/{profile_id}")
async def get_provider(profile_id: str, db: AsyncSession = Depends(get_db)):
    p = await db.get(ProviderProfile, profile_id)
    if not p: raise HTTPException(404, "Provider not found")
    return p
