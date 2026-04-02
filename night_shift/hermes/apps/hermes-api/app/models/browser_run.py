from datetime import datetime

from sqlalchemy import DateTime, ForeignKey, Integer, JSON, String, Text
from sqlalchemy.orm import Mapped, mapped_column

from app.db import Base


class BrowserRun(Base):
    __tablename__ = "browser_runs"

    id: Mapped[int] = mapped_column(Integer, primary_key=True)
    browser_run_id: Mapped[str] = mapped_column(String(120), unique=True, index=True)
    run_id_fk: Mapped[int | None] = mapped_column(ForeignKey("runs.id"), index=True)
    workflow_id: Mapped[str] = mapped_column(String(120))
    start_url: Mapped[str] = mapped_column(String(500))
    allowed_domains_json: Mapped[list] = mapped_column(JSON)
    goal: Mapped[str] = mapped_column(Text)
    status: Mapped[str] = mapped_column(String(50), default="queued")
    auth_profile_ref: Mapped[str | None] = mapped_column(String(120))
    started_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True))
    ended_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True))
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), default=datetime.utcnow)
    updated_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), default=datetime.utcnow)


class BrowserArtifact(Base):
    __tablename__ = "browser_artifacts"

    id: Mapped[int] = mapped_column(Integer, primary_key=True)
    browser_run_id_fk: Mapped[int] = mapped_column(ForeignKey("browser_runs.id"), index=True)
    artifact_type: Mapped[str] = mapped_column(String(80))
    path: Mapped[str] = mapped_column(String(500))
    metadata_json: Mapped[dict | None] = mapped_column(JSON)
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), default=datetime.utcnow)
