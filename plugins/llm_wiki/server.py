import os
from fastapi import FastAPI
from fastapi.responses import HTMLResponse
from fastapi.staticfiles import StaticFiles
from fastapi.middleware.cors import CORSMiddleware
import uvicorn
from dotenv import load_dotenv
from supabase import create_client

# Load environment variables (from the hermes root .env if it exists)
root_dir = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
load_dotenv(os.path.join(root_dir, ".env"))

app = FastAPI(title="LLM Wiki - Second Brain Graph")

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Connect to Supabase
url = os.environ.get("SUPABASE_URL")
key = os.environ.get("SUPABASE_SERVICE_KEY") or os.environ.get("SUPABASE_KEY")
if not url or not key:
    print("Warning: SUPABASE_URL and SUPABASE_KEY/SUPABASE_SERVICE_KEY must be set in .env")

supabase = None
if url and key:
    try:
        supabase = create_client(url, key)
    except Exception as e:
        print(f"Failed to connect to Supabase: {e}")

@app.get("/api/memories")
def get_memories():
    if not supabase:
        return {"error": "Supabase not connected"}
    res = supabase.table("memories").select("id, title, summary, category").limit(100).execute()
    return res.data

@app.get("/api/graph")
def get_graph():
    if not supabase:
        return {"nodes": [], "edges": []}
    
    # Fetch nodes
    mem_res = supabase.table("memories").select("id, title, category").limit(100).execute()
    nodes = [{"id": str(m["id"]), "label": m.get("title", "Untitled"), "group": m.get("category", "Uncategorized")} for m in mem_res.data]
    
    # Fetch edges
    edges = []
    try:
        link_res = supabase.table("memory_links").select("source_id, target_id").limit(500).execute()
        for link in link_res.data:
            edges.append({
                "from": str(link["source_id"]),
                "to": str(link["target_id"])
            })
    except Exception:
        pass # Optional table
        
    return {"nodes": nodes, "edges": edges}

ui_dir = os.path.join(os.path.dirname(os.path.abspath(__file__)), "ui")
os.makedirs(ui_dir, exist_ok=True)

# Mount UI static files
app.mount("/", StaticFiles(directory=ui_dir, html=True), name="ui")

if __name__ == "__main__":
    uvicorn.run("server:app", host="127.0.0.1", port=8085, reload=True)
