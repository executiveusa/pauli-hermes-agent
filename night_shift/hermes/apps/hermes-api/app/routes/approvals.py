from fastapi import APIRouter
router = APIRouter(prefix="/approvals", tags=["approvals"])
@router.get("")
def list_approvals(): return {"items": []}
