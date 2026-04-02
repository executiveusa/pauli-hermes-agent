from fastapi import APIRouter
router = APIRouter(prefix="/subagents", tags=["subagents"])
@router.get("")
def list_subagents(): return {"items": []}
