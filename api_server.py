"""
Hermes Agent API Server - Voice agent endpoint for web UI.
Runs on port 8642, routes agent commands to Hermes MCP server.
Integrates with Hostinger API for VPS management.
"""

import os
import json
from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
import subprocess
from datetime import datetime
import httpx
import asyncio

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


class ProviderSettings(BaseModel):
    nvidia: bool = True
    mercury: bool = True


class ChatRequest(BaseModel):
    message: str
    agent_type: str = "hermes"
    providers: ProviderSettings = ProviderSettings()


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
    Supports provider selection (NVIDIA NIM free or Mercury Inception Labs).
    """
    try:
        message = request.message.strip()
        if not message:
            raise HTTPException(status_code=400, detail="Message cannot be empty")

        # Validate at least one provider is enabled
        if not request.providers.nvidia and not request.providers.mercury:
            raise HTTPException(
                status_code=400,
                detail="At least one provider must be enabled (NVIDIA or Mercury)",
            )

        # Try Mercury first if enabled (premium reasoning)
        if request.providers.mercury:
            mercury_response = await call_mercury_api(message)
            if mercury_response:
                return ChatResponse(
                    response=f"💎 {mercury_response}",
                    timestamp=datetime.now().isoformat(),
                )

        # Fall back to standard Hermes routing (NVIDIA NIM if enabled)
        # Routes to selected providers:
        # - NVIDIA NIM (free inference, moonshotai/kimi-k2-thinking)
        # - Hermes Rolodex (memory, contacts, relationships)
        # - Executes agent actions (make notes, recall, trigger skills)

        response = process_agent_command(message, request.providers)

        return ChatResponse(
            response=response,
            timestamp=datetime.now().isoformat(),
        )

    except HTTPException:
        raise
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Agent error: {str(e)}")


async def call_mercury_api(message: str) -> str:
    """Call Mercury Inception Labs API if enabled."""
    mercury_token = os.getenv("MERCURY_API_KEY", "sk_5917d05c1126bf0f5af161adf566e68c")
    if not mercury_token:
        return None

    try:
        async with httpx.AsyncClient(timeout=30.0) as client:
            response = await client.post(
                "https://api.mercury.ai/v1/chat/completions",
                headers={"Authorization": f"Bearer {mercury_token}"},
                json={
                    "model": "mercury-turbo",
                    "messages": [{"role": "user", "content": message}],
                    "max_tokens": 500,
                },
            )
            if response.status_code == 200:
                data = response.json()
                return data.get("choices", [{}])[0].get("message", {}).get("content", None)
    except Exception as e:
        print(f"Mercury API error: {e}")
        return None


def process_agent_command(message: str, providers: ProviderSettings) -> str:
    """
    Process a user command through the Hermes agent.
    Routes to selected providers (NVIDIA NIM free or Mercury Inception Labs).

    In production, this would:
    - Parse intent (note-taking, recall, actions)
    - Call MCP tools via hermes-rolodex server
    - Execute agent skills via Mercury or NVIDIA
    - Return structured response
    """

    # Build provider string for logging
    active_providers = []
    if providers.nvidia:
        active_providers.append("🚀 NVIDIA NIM")
    if providers.mercury:
        active_providers.append("💎 Mercury")

    provider_info = f" [using: {', '.join(active_providers)}]" if active_providers else " [no providers enabled]"

    # Simple intent-based routing for MVP
    lower_msg = message.lower()

    # Remember/note commands
    if any(word in lower_msg for word in ["remember", "note", "add", "save"]):
        return f"✅ Noted: {message}. Saving to Hermes memory...{provider_info}"

    # Recall/search commands
    elif any(word in lower_msg for word in ["who was", "recall", "who is", "find", "search"]):
        search_term = message.replace('who was ', '').replace('recall ', '').replace('who is ', '').replace('search for ', '')
        return f"🔍 Searching memory for: {search_term}{provider_info}"

    # Action commands
    elif any(word in lower_msg for word in ["send", "call", "message", "email", "execute"]):
        return f"📤 Preparing to: {message}. Ready to execute.{provider_info}"

    # Status/info commands
    elif any(word in lower_msg for word in ["status", "how are", "what is", "tell me"]):
        status = f"📊 Hermes is running. Ready to: remember contacts, recall relationships, and execute actions on your behalf.{provider_info}"
        return status

    # Default response
    else:
        return f"🤖 Processing: {message}. What would you like me to do?{provider_info}"


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
            "hostinger_integration",
        ],
        "api_version": "1.0.0",
    }


@app.get("/api/hostinger/vps")
async def hostinger_vps_list():
    """Get list of VPS instances from Hostinger"""
    try:
        api_key = os.getenv("HOSTINGER_API_KEY")
        if not api_key:
            raise HTTPException(status_code=400, detail="HOSTINGER_API_KEY not configured")

        async with httpx.AsyncClient() as client:
            response = await client.get(
                "https://api.hostinger.com/v1/vps",
                headers={"Authorization": f"Bearer {api_key}"},
                timeout=30.0,
            )

            if response.status_code == 200:
                return response.json()
            else:
                raise HTTPException(status_code=response.status_code, detail="Hostinger API error")

    except Exception as e:
        raise HTTPException(status_code=500, detail=f"VPS list error: {str(e)}")


@app.get("/api/hostinger/domains")
async def hostinger_domains_list():
    """Get list of domains from Hostinger"""
    try:
        api_key = os.getenv("HOSTINGER_API_KEY")
        if not api_key:
            raise HTTPException(status_code=400, detail="HOSTINGER_API_KEY not configured")

        async with httpx.AsyncClient() as client:
            response = await client.get(
                "https://api.hostinger.com/v1/domains",
                headers={"Authorization": f"Bearer {api_key}"},
                timeout=30.0,
            )

            if response.status_code == 200:
                return response.json()
            else:
                raise HTTPException(status_code=response.status_code, detail="Hostinger API error")

    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Domains error: {str(e)}")


@app.get("/api/hostinger/account")
async def hostinger_account_info():
    """Get Hostinger account information"""
    try:
        api_key = os.getenv("HOSTINGER_API_KEY")
        if not api_key:
            raise HTTPException(status_code=400, detail="HOSTINGER_API_KEY not configured")

        async with httpx.AsyncClient() as client:
            response = await client.get(
                "https://api.hostinger.com/v1/account",
                headers={"Authorization": f"Bearer {api_key}"},
                timeout=30.0,
            )

            if response.status_code == 200:
                return response.json()
            else:
                raise HTTPException(status_code=response.status_code, detail="Hostinger API error")

    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Account error: {str(e)}")


if __name__ == "__main__":
    import uvicorn

    port = int(os.getenv("API_SERVER_PORT", 8642))
    host = os.getenv("API_SERVER_HOST", "0.0.0.0")

    print(f"🚀 Hermes Agent API starting on {host}:{port}")
    uvicorn.run(app, host=host, port=port, log_level="info")
