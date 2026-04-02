from fastapi import FastAPI
from app.db import Base, engine
from app.mcp_mount import mcp_router
from app.models import repo, prd, run, approval, coding_session, browser_run, appwrite_project, subagent
from app.routes import health, repos, prds, runs, approvals, coding_sessions, browser_runs, providers, appwrite_projects, subagents
Base.metadata.create_all(bind=engine)
app = FastAPI(title="hermes-api", version="0.1.0")
app.include_router(health.router)
for r in [repos.router, prds.router, runs.router, approvals.router, coding_sessions.router, browser_runs.router, providers.router, appwrite_projects.router, subagents.router]: app.include_router(r, prefix="/api")
app.include_router(mcp_router)
@app.get("/deployment-manifest.json")
def manifest():
    return {"service_name":"hermes-api","deployment_id":"local-dev","git_sha":"dev","environment":"development","base_url":"","api_base_url":"/api","mcp_url":"/mcp","openapi_url":"/openapi.json","health_url":"/healthz","ready_url":"/readyz","auth_mode":"header_token","version":"0.1.0","release_timestamp":"TBD"}
