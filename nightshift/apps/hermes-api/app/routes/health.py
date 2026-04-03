"""Health check routes."""
from fastapi import APIRouter
from app.db import check_db_connection

router = APIRouter()

@router.get("/healthz")
async def healthz():
    return {"status": "ok", "service": "hermes-api"}

@router.get("/readyz")
async def readyz():
    db_ok = await check_db_connection()
    return {"ready": db_ok, "database": "connected" if db_ok else "disconnected"}

@router.get("/version")
async def version():
    return {"version": "1.0.0", "codename": "Night Shift"}
