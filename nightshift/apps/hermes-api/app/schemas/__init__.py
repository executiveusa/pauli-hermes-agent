"""Agent MAXX — Pydantic v2 Request/Response Schemas."""

from datetime import datetime
from typing import Optional
from uuid import UUID

from pydantic import BaseModel, Field


# ── Repos ──
class RepoOut(BaseModel):
    id: UUID
    repo_url: str
    name: Optional[str] = None
    source: str = "github"
    last_scanned_at: Optional[datetime] = None
    created_at: datetime
    profile: Optional[dict] = None
    class Config: from_attributes = True

# ── PRDs ──
class PRDBatchOut(BaseModel):
    id: UUID; status: str; strategy: Optional[str] = None; prd_count: int = 0
    created_at: datetime; completed_at: Optional[datetime] = None
    class Config: from_attributes = True

class PRDOut(BaseModel):
    id: UUID; batch_id: Optional[UUID] = None; repo_id: Optional[UUID] = None
    title: str; body: str; priority: str = "medium"; status: str = "draft"
    approved_by: Optional[str] = None; created_at: datetime
    class Config: from_attributes = True

class PRDBatchGenerate(BaseModel):
    strategy: str = "auto"
    repo_ids: Optional[list[UUID]] = None

class PRDDecision(BaseModel):
    decision: str  # "approve" | "reject"

# ── Runs ──
class RunOut(BaseModel):
    id: UUID; run_type: Optional[str] = None; status: str; model: Optional[str] = None
    started_at: Optional[datetime] = None; finished_at: Optional[datetime] = None
    iteration_count: int = 0; error: Optional[str] = None
    class Config: from_attributes = True

class RunStepOut(BaseModel):
    id: UUID; run_id: UUID; step_index: int; tool_name: Optional[str] = None
    tool_output: Optional[str] = None; duration_ms: Optional[int] = None; created_at: datetime
    class Config: from_attributes = True

# ── Approvals ──
class ApprovalOut(BaseModel):
    id: UUID; run_id: Optional[UUID] = None; risk_level: str; proposed_command: Optional[str] = None
    reason: Optional[str] = None; decision: Optional[str] = None; decided_by: Optional[str] = None
    created_at: datetime
    class Config: from_attributes = True

class ApprovalDecide(BaseModel):
    decision: str  # "approve" | "reject" | "modify"

# ── Coding Sessions ──
class CodingSessionCreate(BaseModel):
    repo_url: str
    prompt: Optional[str] = None
    task_type: str = "update"
    provider_profile: str = "balanced"

class CodingSessionOut(BaseModel):
    id: UUID; repo_url: str; prompt: Optional[str] = None; task_type: str
    status: str; branch: Optional[str] = None; error: Optional[str] = None
    created_at: datetime; finished_at: Optional[datetime] = None
    class Config: from_attributes = True

# ── Browser Runs ──
class BrowserRunCreate(BaseModel):
    start_url: str
    steps: list[dict] = Field(default_factory=list)

class BrowserRunOut(BaseModel):
    id: UUID; start_url: Optional[str] = None; state: str; artifact_count: int = 0
    error: Optional[str] = None; created_at: datetime; finished_at: Optional[datetime] = None
    class Config: from_attributes = True

class BrowserArtifactOut(BaseModel):
    id: UUID; browser_run_id: UUID; kind: str; label: Optional[str] = None
    content: Optional[str] = None; file_path: Optional[str] = None; created_at: datetime
    class Config: from_attributes = True

# ── Providers ──
class ProviderProfileOut(BaseModel):
    id: str; provider: str; model: str; purpose: Optional[str] = None
    is_active: bool = True
    class Config: from_attributes = True

# ── Sub-Agents ──
class SubAgentCreate(BaseModel):
    label: Optional[str] = None
    repo_scope: list[str]
    tool_scope: list[str]
    time_budget_minutes: int = Field(default=30, ge=1, le=1440)

class SubAgentOut(BaseModel):
    id: UUID; label: Optional[str] = None; repo_scope: list[str]; tool_scope: list[str]
    time_budget_minutes: int; status: str; error: Optional[str] = None; created_at: datetime
    class Config: from_attributes = True

# ── Appwrite ──
class AppwriteProjectOut(BaseModel):
    id: UUID; name: str; status: str; endpoint: Optional[str] = None
    project_id: Optional[str] = None; created_at: datetime
    class Config: from_attributes = True

# ── Dashboard ──
class DashboardOverview(BaseModel):
    repos_total: int = 0
    runs_active: int = 0
    approvals_pending: int = 0
    coding_sessions_active: int = 0
    browser_runs_active: int = 0
    subagents_active: int = 0
    missing_secrets: list[dict] = Field(default_factory=list)

class MissingSecretOut(BaseModel):
    key: str; description: str; why: str; deferrable: bool
