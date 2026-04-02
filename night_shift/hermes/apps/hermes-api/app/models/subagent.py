from datetime import datetime

from sqlalchemy import Boolean, DateTime, Float, Integer, JSON, String
from sqlalchemy.orm import Mapped, mapped_column

from app.db import Base


class ProviderProfile(Base):
    __tablename__ = "provider_profiles"

    id: Mapped[int] = mapped_column(Integer, primary_key=True)
    provider_profile_id: Mapped[str] = mapped_column(String(120), unique=True, index=True)
    provider_type: Mapped[str] = mapped_column(String(80))
    model: Mapped[str] = mapped_column(String(120))
    api_base_url: Mapped[str | None] = mapped_column(String(500))
    secret_ref: Mapped[str] = mapped_column(String(120))
    max_tokens: Mapped[int | None] = mapped_column(Integer)
    temperature: Mapped[float | None] = mapped_column(Float)
    purpose: Mapped[str | None] = mapped_column(String(120))
    fallback_profile: Mapped[str | None] = mapped_column(String(120))
    is_active: Mapped[bool] = mapped_column(Boolean, default=True)
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), default=datetime.utcnow)
    updated_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), default=datetime.utcnow)


class Subagent(Base):
    __tablename__ = "subagents"

    id: Mapped[int] = mapped_column(Integer, primary_key=True)
    subagent_id: Mapped[str] = mapped_column(String(120), unique=True, index=True)
    purpose: Mapped[str] = mapped_column(String(500))
    repo_scope_json: Mapped[list] = mapped_column(JSON)
    tool_scope_json: Mapped[list] = mapped_column(JSON)
    provider_profile_id: Mapped[str] = mapped_column(String(120))
    time_budget_minutes: Mapped[int] = mapped_column(Integer)
    memory_scope: Mapped[str | None] = mapped_column(String(120))
    approval_policy: Mapped[str] = mapped_column(String(120))
    status: Mapped[str] = mapped_column(String(50), default="active")
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), default=datetime.utcnow)
    updated_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), default=datetime.utcnow)
