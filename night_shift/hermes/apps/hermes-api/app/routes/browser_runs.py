from fastapi import APIRouter
from app.services.browser_runs import create_browser_run
router = APIRouter(prefix="/browser-runs", tags=["browser-runs"])
@router.post("")
def create(payload: dict): return create_browser_run(payload)
