from datetime import datetime

from sqlalchemy import DateTime, Integer, JSON, String
from sqlalchemy.orm import Mapped, mapped_column

from app.db import Base


class AppwriteProject(Base):
    __tablename__ = "appwrite_projects"

    id: Mapped[int] = mapped_column(Integer, primary_key=True)
    tenant_id: Mapped[str] = mapped_column(String(120), index=True)
    project_name: Mapped[str] = mapped_column(String(200))
    appwrite_project_id: Mapped[str | None] = mapped_column(String(120), index=True)
    endpoint: Mapped[str | None] = mapped_column(String(500))
    features_json: Mapped[dict | None] = mapped_column(JSON)
    environment: Mapped[str] = mapped_column(String(50), default="staging")
    status: Mapped[str] = mapped_column(String(50), default="provisioning")
    metadata_json: Mapped[dict | None] = mapped_column(JSON)
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), default=datetime.utcnow)
    updated_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), default=datetime.utcnow)
