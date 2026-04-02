from fastapi import APIRouter
from app.services.coding_sessions import create_coding_session
router = APIRouter(prefix="/coding-sessions", tags=["coding-sessions"])
@router.post("")
def create(payload: dict): return create_coding_session(payload)
