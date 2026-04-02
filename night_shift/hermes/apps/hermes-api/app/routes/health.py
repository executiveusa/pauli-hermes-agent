from fastapi import APIRouter
router = APIRouter(tags=["health"])
@router.get("/healthz")
def healthz(): return {"ok": True}
@router.get("/readyz")
def readyz(): return {"ready": True}
@router.get("/version")
def version(): return {"service":"hermes-api","version":"0.1.0"}
