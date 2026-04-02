from fastapi import APIRouter
router = APIRouter(prefix="/prds", tags=["prds"])
@router.get("")
def list_prds(): return {"items": []}
