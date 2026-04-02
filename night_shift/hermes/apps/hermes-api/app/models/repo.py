from datetime import datetime

from sqlalchemy import Boolean, DateTime, Float, ForeignKey, Integer, JSON, String, Text
from sqlalchemy.orm import Mapped, mapped_column

from app.db import Base


class Repo(Base):
    __tablename__ = "repos"

    id: Mapped[int] = mapped_column(Integer, primary_key=True)
    repo_full_name: Mapped[str] = mapped_column(String(255), unique=True, index=True)
    repo_url: Mapped[str] = mapped_column(String(500))
    default_branch: Mapped[str] = mapped_column(String(120), default="main")
    provider: Mapped[str] = mapped_column(String(50), default="github")
    is_active: Mapped[bool] = mapped_column(Boolean, default=True)
    language_summary: Mapped[str | None] = mapped_column(Text)
    framework_summary: Mapped[str | None] = mapped_column(Text)
    package_manager: Mapped[str | None] = mapped_column(String(80))
    has_docker: Mapped[bool] = mapped_column(Boolean, default=False)
    has_ci: Mapped[bool] = mapped_column(Boolean, default=False)
    has_tests: Mapped[bool] = mapped_column(Boolean, default=False)
    has_frontend: Mapped[bool] = mapped_column(Boolean, default=False)
    has_backend: Mapped[bool] = mapped_column(Boolean, default=False)
    deployability_score: Mapped[float] = mapped_column(Float, default=0)
    risk_score: Mapped[float] = mapped_column(Float, default=0)
    last_scanned_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True))
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), default=datetime.utcnow)
    updated_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), default=datetime.utcnow)


class RepoProfile(Base):
    __tablename__ = "repo_profiles"

    id: Mapped[int] = mapped_column(Integer, primary_key=True)
    repo_id: Mapped[int] = mapped_column(ForeignKey("repos.id"), index=True)
    summary: Mapped[str | None] = mapped_column(Text)
    architecture_notes: Mapped[str | None] = mapped_column(Text)
    commands_detected_json: Mapped[dict | None] = mapped_column(JSON)
    risks_json: Mapped[list | None] = mapped_column(JSON)
    recommendations_json: Mapped[list | None] = mapped_column(JSON)
    raw_profile_json: Mapped[dict | None] = mapped_column(JSON)
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), default=datetime.utcnow)
