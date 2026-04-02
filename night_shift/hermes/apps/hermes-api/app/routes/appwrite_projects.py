from fastapi import APIRouter
router = APIRouter(prefix="/appwrite-projects", tags=["appwrite_projects"])
@router.get("")
def list_appwrite_projects(): return {"items": []}
