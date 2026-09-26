import os
import json
from pathlib import Path
from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import FileResponse, JSONResponse

def create_obsidian_bridge_app(vault_path: str = None) -> FastAPI:
    """
    Creates a FastAPI app that bridges the local headless Obsidian Vault to the Hermes Web UI.
    Serves the JSON Canvas map and the Markdown memories.
    """
    app = FastAPI(title="Hermes Obsidian Bridge")
    
    app.add_middleware(
        CORSMiddleware,
        allow_origins=["*"], # Allow the hermes-web-frontend to access
        allow_credentials=True,
        allow_methods=["*"],
        allow_headers=["*"],
    )
    
    resolved_vault_path = vault_path or os.environ.get("OBSIDIAN_VAULT_PATH", os.path.expanduser("~/Obsidian_Hermes_Vault"))
    vault_dir = Path(resolved_vault_path)
    
    @app.get("/api/vault/canvas")
    async def get_agent_canvas():
        """Returns the primary Agent_Brain_Map.canvas for the Web UI."""
        canvas_path = vault_dir / "Agent_Brain_Map.canvas"
        if not canvas_path.exists():
            return JSONResponse(status_code=404, content={"error": "Canvas map not generated yet."})
            
        with open(canvas_path, "r", encoding="utf-8") as f:
            data = json.load(f)
        return data

    @app.get("/api/vault/memories")
    async def get_memories():
        """Lists all memories in the vault."""
        if not vault_dir.exists():
            return []
            
        memories = []
        for file_path in vault_dir.rglob("*.md"):
            memories.append({"id": file_path.stem, "path": str(file_path.relative_to(vault_dir))})
            
        return memories

    @app.get("/api/vault/memory/{memory_id}")
    async def get_memory(memory_id: str):
        """Returns a specific memory file."""
        file_path = vault_dir / f"{memory_id}.md"
        # Search recursively if not in root
        if not file_path.exists():
            matches = list(vault_dir.rglob(f"{memory_id}.md"))
            if matches:
                file_path = matches[0]
            else:
                raise HTTPException(status_code=404, detail="Memory not found")
                
        return FileResponse(file_path)

    return app

if __name__ == "__main__":
    import uvicorn
    # Standalone test server
    app = create_obsidian_bridge_app()
    print("Starting Obsidian Bridge Server on http://0.0.0.0:8001")
    uvicorn.run(app, host="0.0.0.0", port=8001)
