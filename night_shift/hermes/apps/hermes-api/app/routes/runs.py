from fastapi import APIRouter
router = APIRouter(prefix="/runs", tags=["runs"])
@router.get("")
def list_runs(): return {"items": []}
