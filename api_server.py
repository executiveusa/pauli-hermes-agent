"""
Hermes Agent API Server - Voice agent endpoint for web UI.
Runs on port 8642, routes agent commands to Hermes MCP server.
"""

import os
import json
from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
import subprocess
from datetime import datetime

app = FastAPI(title="Hermes Agent API", version="1.0.0")

# CORS middleware for web UI access
app.add_middleware(
    CORSMiddleware,
    allow_origins=[
        "http://localhost:3000",
        "http://localhost:8642",
        "https://pauli-hermes-agent.vercel.app",
        "https://*.vercel.app",
    ],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


class ChatRequest(BaseModel):
    message: str
    agent_type: str = "hermes"


class ChatResponse(BaseModel):
    response: str
    timestamp: str


@app.get("/health")
async def health():
    """Health check endpoint."""
    return {"status": "ok", "service": "hermes-agent-api"}


@app.post("/api/chat")
async def chat(request: ChatRequest) -> ChatResponse:
    """
    Chat with the Hermes agent.
    Routes voice transcripts to the agent for processing.
    """
    try:
        message = request.message.strip()
        if not message:
            raise HTTPException(status_code=400, detail="Empty message")

        # Call Hermes CLI or MCP to process the command
        # For now, return a simple response. In production, this calls:
        # - Hermes MCP server (hermes-rolodex, jcodemunch-mcp)
        # - Executes agent actions (make notes, recall contacts, trigger skills)

        response = process_agent_command(message)

        return ChatResponse(
            response=response,
            timestamp=datetime.now().isoformat(),
        )

    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


def process_agent_command(message: str) -> str:
    """
    Process a user command through the Hermes agent.
    In production, this would:
    - Parse intent (note-taking, recall, actions)
    - Call MCP tools via hermes-rolodex server
    - Execute agent skills
    - Return structured response
    """

    # Simple intent-based routing for MVP
    lower_msg = message.lower()

    # Remember/note commands
    if any(word in lower_msg for word in ["remember", "note", "add", "save", "remember"]):
        return f"✅ Noted: {message}. Saving to Hermes memory..."

    # Recall/search commands
    elif any(word in lower_msg for word in ["who was", "recall", "remember", "who is", "find"]):
        return f"🔍 Searching memory for: {message.replace('who was ', '').replace('recall ', '').replace('who is ', '')}"

    # Action commands
    elif any(word in lower_msg for word in ["send", "call", "message", "email"]):
        return f"📤 Preparing to: {message}. Ready to execute."

    # Status/info commands
    elif any(word in lower_msg for word in ["status", "how are", "what is", "tell me"]):
        return f"📊 Hermes is running. Ready to: remember contacts, recall relationships, and execute actions on your behalf."

    # Default response
    else:
        return f"🤖 Processing: {message}. What would you like me to do?"


@app.get("/api/status")
async def status():
    """Get agent status."""
    return {
        "agent": "Hermes",
        "status": "active",
        "features": [
            "voice_control",
            "memory_recall",
            "action_execution",
            "relationship_strength",
        ],
        "api_version": "1.0.0",
    }


if __name__ == "__main__":
    import uvicorn

    port = int(os.getenv("API_SERVER_PORT", 8642))
    host = os.getenv("API_SERVER_HOST", "0.0.0.0")

    print(f"🚀 Hermes Agent API starting on {host}:{port}")
    uvicorn.run(app, host=host, port=port, log_level="info")
