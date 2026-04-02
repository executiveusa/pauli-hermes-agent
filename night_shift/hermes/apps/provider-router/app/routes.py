from fastapi import APIRouter
from app.schemas import ChatRequest
from app.providers import openai_compatible, gemini, generic_http
router = APIRouter()
@router.post("/v1/llm/chat")
async def chat(req: ChatRequest):
    pid = req.provider_profile_id.lower()
    out = await (gemini.chat(req.model_dump()) if "gemini" in pid else generic_http.chat(req.model_dump()) if "http" in pid else openai_compatible.chat(req.model_dump()))
    out["provider_profile_id"] = req.provider_profile_id
    return out
