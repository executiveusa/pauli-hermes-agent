from __future__ import annotations

from datetime import datetime
from pydantic import BaseModel, Field


class HealthResponse(BaseModel):
    status: str
    services: dict[str, str]
    timestamp: datetime = Field(default_factory=datetime.utcnow)


class RepoScanRequest(BaseModel):
    repo_url: str
    branch: str = "main"


class PRDRequest(BaseModel):
    repo_id: str
    title: str
    objective: str


class ChatRequest(BaseModel):
    profile: str = "model_fallback"
    messages: list[dict]
    temperature: float = 0.2
    max_tokens: int = 1024
    tools: list[dict] | None = None


class ApprovalRequest(BaseModel):
    run_id: str
    action: str
    reason: str


class SubagentRequest(BaseModel):
    name: str
    mission: str
    scope: list[str]


class AppwriteProjectRequest(BaseModel):
    name: str
    region: str = "us-east"


class ProviderProfile(BaseModel):
    name: str
    base_url: str
    model: str
    api_key_env: str
    max_cost_per_1k: float = 0.0
    budget_cap_usd: float = 20.0
