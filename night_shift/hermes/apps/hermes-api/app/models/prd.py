from datetime import datetime

from sqlalchemy import DateTime, ForeignKey, Integer, JSON, String, Text
from sqlalchemy.orm import Mapped, mapped_column

from app.db import Base


class PRDBatch(Base):
    __tablename__ = "prd_batches"

    id: Mapped[int] = mapped_column(Integer, primary_key=True)
    label: Mapped[str] = mapped_column(String(200), index=True)
    selection_query: Mapped[str | None] = mapped_column(Text)
    status: Mapped[str] = mapped_column(String(50), default="pending")
    batch_size: Mapped[int] = mapped_column(Integer)
    created_by: Mapped[str | None] = mapped_column(String(120))
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), default=datetime.utcnow)
    updated_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), default=datetime.utcnow)


class PRD(Base):
    __tablename__ = "prds"

    id: Mapped[int] = mapped_column(Integer, primary_key=True)
    repo_id: Mapped[int] = mapped_column(ForeignKey("repos.id"), index=True)
    batch_id: Mapped[int] = mapped_column(ForeignKey("prd_batches.id"), index=True)
    title: Mapped[str] = mapped_column(String(300))
    problem_statement: Mapped[str | None] = mapped_column(Text)
    current_state: Mapped[str | None] = mapped_column(Text)
    target_state: Mapped[str | None] = mapped_column(Text)
    acceptance_criteria_json: Mapped[list | None] = mapped_column(JSON)
    rollout_plan: Mapped[str | None] = mapped_column(Text)
    rollback_plan: Mapped[str | None] = mapped_column(Text)
    status: Mapped[str] = mapped_column(String(50), default="draft")
    risk_level: Mapped[str | None] = mapped_column(String(50))
    artifact_path: Mapped[str | None] = mapped_column(String(500))
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), default=datetime.utcnow)
    updated_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), default=datetime.utcnow)
