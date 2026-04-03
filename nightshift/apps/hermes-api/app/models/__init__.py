"""Agent MAXX — SQLAlchemy ORM models (14 tables)."""

import enum
import uuid
from datetime import datetime, timezone

from sqlalchemy import Boolean, Column, DateTime, Enum, ForeignKey, Integer, String, Text, JSON
from sqlalchemy.dialects.postgresql import UUID, ARRAY
from sqlalchemy.orm import relationship

from app.db import Base


def utcnow():
    return datetime.now(timezone.utc)


def new_uuid():
    return uuid.uuid4()


class RunStatus(str, enum.Enum):
    queued = "queued"
    starting = "starting"
    running = "running"
    paused = "paused"
    completed = "completed"
    failed = "failed"
    cancelled = "cancelled"
    timeout = "timeout"


class ActionRisk(str, enum.Enum):
    none = "none"
    low = "low"
    medium = "medium"
    high = "high"
    critical = "critical"


class Repo(Base):
    __tablename__ = "repos"
    id = Column(UUID(as_uuid=True), primary_key=True, default=new_uuid)
    repo_url = Column(Text, nullable=False, unique=True)
    name = Column(Text)
    source = Column(Text, default="github")
    last_scanned_at = Column(DateTime(timezone=True))
    created_at = Column(DateTime(timezone=True), default=utcnow)
    profile = relationship("RepoProfile", uselist=False, back_populates="repo")


class RepoProfile(Base):
    __tablename__ = "repo_profiles"
    id = Column(UUID(as_uuid=True), primary_key=True, default=new_uuid)
    repo_id = Column(UUID(as_uuid=True), ForeignKey("repos.id", ondelete="CASCADE"), nullable=False)
    language = Column(Text)
    framework = Column(Text)
    package_manager = Column(Text)
    has_tests = Column(Boolean, default=False)
    has_lint = Column(Boolean, default=False)
    has_build = Column(Boolean, default=False)
    has_docker = Column(Boolean, default=False)
    has_ci = Column(Boolean, default=False)
    test_command = Column(Text)
    lint_command = Column(Text)
    build_command = Column(Text)
    readme_summary = Column(Text)
    stars = Column(Integer, default=0)
    extra = Column(JSON, default=dict)
    created_at = Column(DateTime(timezone=True), default=utcnow)
    repo = relationship("Repo", back_populates="profile")


class PRDBatch(Base):
    __tablename__ = "prd_batches"
    id = Column(UUID(as_uuid=True), primary_key=True, default=new_uuid)
    status = Column(Text, default="pending")
    strategy = Column(Text)
    prd_count = Column(Integer, default=0)
    created_at = Column(DateTime(timezone=True), default=utcnow)
    completed_at = Column(DateTime(timezone=True))
    prds = relationship("PRD", back_populates="batch")


class PRD(Base):
    __tablename__ = "prds"
    id = Column(UUID(as_uuid=True), primary_key=True, default=new_uuid)
    batch_id = Column(UUID(as_uuid=True), ForeignKey("prd_batches.id", ondelete="SET NULL"))
    repo_id = Column(UUID(as_uuid=True), ForeignKey("repos.id", ondelete="SET NULL"))
    title = Column(Text, nullable=False)
    body = Column(Text, nullable=False)
    priority = Column(Text, default="medium")
    status = Column(Text, default="draft")
    approved_by = Column(Text)
    created_at = Column(DateTime(timezone=True), default=utcnow)
    batch = relationship("PRDBatch", back_populates="prds")


class Run(Base):
    __tablename__ = "runs"
    id = Column(UUID(as_uuid=True), primary_key=True, default=new_uuid)
    run_type = Column(Text)
    status = Column(Text, default="queued")
    model = Column(Text)
    started_at = Column(DateTime(timezone=True), default=utcnow)
    finished_at = Column(DateTime(timezone=True))
    iteration_count = Column(Integer, default=0)
    token_usage = Column(JSON, default=dict)
    error = Column(Text)
    metadata_ = Column("metadata", JSON, default=dict)
    steps = relationship("RunStep", back_populates="run")


class RunStep(Base):
    __tablename__ = "run_steps"
    id = Column(UUID(as_uuid=True), primary_key=True, default=new_uuid)
    run_id = Column(UUID(as_uuid=True), ForeignKey("runs.id", ondelete="CASCADE"), nullable=False)
    step_index = Column(Integer, nullable=False)
    tool_name = Column(Text)
    tool_input = Column(JSON, default=dict)
    tool_output = Column(Text)
    token_usage = Column(JSON, default=dict)
    duration_ms = Column(Integer)
    created_at = Column(DateTime(timezone=True), default=utcnow)
    run = relationship("Run", back_populates="steps")


class Approval(Base):
    __tablename__ = "approvals"
    id = Column(UUID(as_uuid=True), primary_key=True, default=new_uuid)
    run_id = Column(UUID(as_uuid=True), ForeignKey("runs.id", ondelete="CASCADE"))
    risk_level = Column(Text, default="medium")
    proposed_command = Column(Text)
    reason = Column(Text)
    decision = Column(Text)
    decided_by = Column(Text)
    decided_at = Column(DateTime(timezone=True))
    created_at = Column(DateTime(timezone=True), default=utcnow)


class CodingSession(Base):
    __tablename__ = "coding_sessions"
    id = Column(UUID(as_uuid=True), primary_key=True, default=new_uuid)
    repo_url = Column(Text, nullable=False)
    prompt = Column(Text)
    task_type = Column(Text, default="update")
    status = Column(Text, default="queued")
    branch = Column(Text)
    provider_profile = Column(Text, default="balanced")
    error = Column(Text)
    created_at = Column(DateTime(timezone=True), default=utcnow)
    finished_at = Column(DateTime(timezone=True))


class CodingSessionStep(Base):
    __tablename__ = "coding_session_steps"
    id = Column(UUID(as_uuid=True), primary_key=True, default=new_uuid)
    session_id = Column(UUID(as_uuid=True), ForeignKey("coding_sessions.id", ondelete="CASCADE"), nullable=False)
    step_name = Column(Text)
    status = Column(Text)
    output = Column(Text)
    created_at = Column(DateTime(timezone=True), default=utcnow)


class BrowserRun(Base):
    __tablename__ = "browser_runs"
    id = Column(UUID(as_uuid=True), primary_key=True, default=new_uuid)
    start_url = Column(Text)
    state = Column(Text, default="queued")
    steps = Column(JSON, default=list)
    artifact_count = Column(Integer, default=0)
    error = Column(Text)
    created_at = Column(DateTime(timezone=True), default=utcnow)
    finished_at = Column(DateTime(timezone=True))


class BrowserArtifact(Base):
    __tablename__ = "browser_artifacts"
    id = Column(UUID(as_uuid=True), primary_key=True, default=new_uuid)
    browser_run_id = Column(UUID(as_uuid=True), ForeignKey("browser_runs.id", ondelete="CASCADE"), nullable=False)
    kind = Column(Text, nullable=False)
    label = Column(Text)
    content = Column(Text)
    file_path = Column(Text)
    created_at = Column(DateTime(timezone=True), default=utcnow)


class ProviderProfile(Base):
    __tablename__ = "provider_profiles"
    id = Column(Text, primary_key=True)
    provider = Column(Text, nullable=False)
    model = Column(Text, nullable=False)
    purpose = Column(Text)
    is_active = Column(Boolean, default=True)
    config = Column(JSON, default=dict)
    created_at = Column(DateTime(timezone=True), default=utcnow)


class SubAgent(Base):
    __tablename__ = "sub_agents"
    id = Column(UUID(as_uuid=True), primary_key=True, default=new_uuid)
    label = Column(Text)
    repo_scope = Column(ARRAY(Text), nullable=False)
    tool_scope = Column(ARRAY(Text), nullable=False)
    time_budget_minutes = Column(Integer, default=30)
    status = Column(Text, default="pending")
    started_at = Column(DateTime(timezone=True))
    finished_at = Column(DateTime(timezone=True))
    error = Column(Text)
    created_at = Column(DateTime(timezone=True), default=utcnow)


class AppwriteProject(Base):
    __tablename__ = "appwrite_projects"
    id = Column(UUID(as_uuid=True), primary_key=True, default=new_uuid)
    name = Column(Text, nullable=False)
    status = Column(Text, default="provisioning")
    endpoint = Column(Text)
    project_id = Column(Text)
    api_key = Column(Text)
    created_at = Column(DateTime(timezone=True), default=utcnow)


class Artifact(Base):
    __tablename__ = "artifacts"
    id = Column(UUID(as_uuid=True), primary_key=True, default=new_uuid)
    parent_type = Column(Text, nullable=False)
    parent_id = Column(UUID(as_uuid=True), nullable=False)
    kind = Column(Text, nullable=False)
    label = Column(Text)
    content = Column(Text)
    file_path = Column(Text)
    created_at = Column(DateTime(timezone=True), default=utcnow)
