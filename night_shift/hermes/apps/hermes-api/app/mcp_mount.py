from fastapi import APIRouter
mcp_router = APIRouter(prefix="/mcp", tags=["mcp"])
@mcp_router.get("")
def describe_mcp():
    return {"tools": ["catalog_repos","generate_prd_batch","list_prd_batch","approve_prd","enqueue_repo_work","get_run_status","list_open_prs","create_browser_run","get_browser_run_status","pause_browser_run","resume_browser_run","cancel_browser_run","provision_appwrite_project","list_appwrite_projects","create_subagent","list_subagents","stop_subagent"]}
