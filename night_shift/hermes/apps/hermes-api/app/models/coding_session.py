from datetime import datetime

from sqlalchemy import DateTime, ForeignKey, Integer, JSON, String, Text
from sqlalchemy.orm import Mapped, mapped_column

from app.db import Base


class CodingSession(Base):
    __tablename__ = "coding_sessions"

    id: Mapped[int] = mapped_column(Integer, primary_key=True)
    session_id: Mapped[str] = mapped_column(String(120), unique=True, index=True)
    repo_id: Mapped[int | None] = mapped_column(ForeignKey("repos.id"), index=True)
    run_id_fk: Mapped[int | None] = mapped_column(ForeignKey("runs.id"), index=True)
    provider_profile_id: Mapped[str] = mapped_column(String(120))
    workspace_path: Mapped[str] = mapped_column(String(500))
    task_type: Mapped[str] = mapped_column(String(80))
    status: Mapped[str] = mapped_column(String(50), default="queued")
    prompt_text: Mapped[str] = mapped_column(Text)
    time_budget_minutes: Mapped[int] = mapped_column(Integer, default=60)
    started_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True))
    ended_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True))
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), default=datetime.utcnow)
    updated_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), default=datetime.utcnow)


class CodingSessionStep(Base):
    __tablename__ = "coding_session_steps"

    id: Mapped[int] = mapped_column(Integer, primary_key=True)
    session_id_fk: Mapped[int] = mapped_column(ForeignKey("coding_sessions.id"), index=True)
    step_name: Mapped[str] = mapped_column(String(120))
    status: Mapped[str] = mapped_column(String(50))
    summary: Mapped[str | None] = mapped_column(Text)
    artifact_path: Mapped[str | None] = mapped_column(String(500))
    metadata_json: Mapped[dict | None] = mapped_column(JSON)
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), default=datetime.utcnow)
