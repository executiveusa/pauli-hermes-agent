"""Agent MAXX — hermes-api: FastAPI control plane."""

import logging
from contextlib import asynccontextmanager

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

from app.config import settings
from app.db import init_db

logger = logging.getLogger(__name__)


@asynccontextmanager
async def lifespan(app: FastAPI):
    logger.info(f"🐕 {settings.app_name} API starting…")
    await init_db()
    missing = settings.missing_secrets()
    if missing:
        critical = [s for s in missing if not s["deferrable"]]
        if critical:
            logger.warning(f"⚠️  {len(critical)} critical secrets missing: {[s['key'] for s in critical]}")
        logger.info(f"ℹ️  {len(missing)} total secrets not configured (use /api/dashboard/missing-secrets)")
    yield
    logger.info("Shutting down…")


app = FastAPI(
    title="Agent MAXX API",
    description="Night Shift control plane",
    version="1.0.0",
    lifespan=lifespan,
)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_methods=["*"],
    allow_headers=["*"],
)

# ── Mount Routes ──
from app.routes import health, dashboard, repos, prds, runs, approvals
from app.routes import coding_sessions, browser_runs, providers, subagents, appwrite

app.include_router(health.router)
app.include_router(dashboard.router, prefix="/api")
app.include_router(repos.router, prefix="/api")
app.include_router(prds.router, prefix="/api")
app.include_router(runs.router, prefix="/api")
app.include_router(approvals.router, prefix="/api")
app.include_router(coding_sessions.router, prefix="/api")
app.include_router(browser_runs.router, prefix="/api")
app.include_router(providers.router, prefix="/api")
app.include_router(subagents.router, prefix="/api")
app.include_router(appwrite.router, prefix="/api")

# ── MCP Mount ──
try:
    from fastapi_mcp import FastApiMCP
    mcp = FastApiMCP(app, name="Agent MAXX MCP", description="Night Shift tool surface")
    mcp.mount()
    logger.info("MCP mounted at /mcp")
except ImportError:
    logger.info("fastapi-mcp not installed — MCP endpoint disabled")
