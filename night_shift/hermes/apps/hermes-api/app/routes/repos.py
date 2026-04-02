from fastapi import APIRouter
router = APIRouter(prefix="/repos", tags=["repos"])
@router.get("")
def list_repos(): return {"items": []}
