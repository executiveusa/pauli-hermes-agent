from pydantic import BaseModel
from typing import Any
class ChatRequest(BaseModel):
    provider_profile_id: str
    messages: list[dict[str, Any]]
    temperature: float | None = 0.2
    max_tokens: int | None = 2048
    tools: list[dict[str, Any]] | None = None
    metadata: dict[str, Any] | None = None
