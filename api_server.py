"""
Hermes Agent API Server - Voice agent endpoint for web UI.
Runs on port 8642. Provider priority:
  1. Synthia Gateway  (OpenAI/Groq/etc. — fast, real AI)
  2. Mercury          (Inception Labs diffusion model)
  3. NVIDIA NIM       (free moonshotai/kimi-k2-thinking)
"""

import os
from fastapi import FastAPI, HTTPException, Request, Depends
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
from datetime import datetime
import httpx

app = FastAPI(title="Hermes Agent API", version="2.0.0")

app.add_middleware(
    CORSMiddleware,
    allow_origins=[
        "http://localhost:3000",
        "http://localhost:8642",
        "https://pauli-hermes-agent.vercel.app",
    ],
    allow_origin_regex=r"^https://.*\.vercel\.app$",
    allow_credentials=True,
    allow_methods=["GET", "POST", "OPTIONS"],
    allow_headers=["Content-Type", "Authorization", "X-Hermes-Key"],
)


def verify_api_key(request: Request) -> None:
    """Verify the X-Hermes-Key header matches the configured API key."""
    api_key = os.getenv("HERMES_API_KEY")
    if not api_key:
        raise HTTPException(
            status_code=500,
            detail="HERMES_API_KEY not configured on server",
        )

    provided_key = request.headers.get("X-Hermes-Key", "")
    if provided_key != api_key:
        raise HTTPException(
            status_code=401,
            detail="Unauthorized: Invalid or missing X-Hermes-Key header",
        )

HERMES_SYSTEM_PROMPT = (
    "You are Hermes, a personal AI agent. You help remember contacts, "
    "recall relationships, take notes, and execute actions on behalf of the user. "
    "Be concise — your responses will be read aloud via text-to-speech. "
    "Keep replies under 3 sentences unless the user asks for more detail."
)


def verify_api_key(request: Request) -> None:
    """Verify the X-Hermes-Key header matches the configured API key."""
    api_key = os.getenv("HERMES_API_KEY")
    if not api_key:
        raise HTTPException(
            status_code=500,
            detail="HERMES_API_KEY not configured on server",
        )

    provided_key = request.headers.get("X-Hermes-Key", "")
    if provided_key != api_key:
        raise HTTPException(
            status_code=401,
            detail="Unauthorized: Invalid or missing X-Hermes-Key header",
        )

HERMES_SYSTEM_PROMPT = (
    "You are Hermes, a personal AI agent. You help remember contacts, "
    "recall relationships, take notes, and execute actions on behalf of the user. "
    "Be concise — your responses will be read aloud via text-to-speech. "
    "Keep replies under 3 sentences unless the user asks for more detail."
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
    provider: str
    timestamp: str


@app.get("/health")
async def health():
    return {"status": "ok", "service": "hermes-agent-api", "version": "2.0.0"}


@app.post("/api/chat")
async def chat(request: ChatRequest, _: None = Depends(verify_api_key)) -> ChatResponse:
    message = request.message.strip()

    if not message:
        raise HTTPException(status_code=400, detail="Message cannot be empty")

    if len(message) > 4000:
        raise HTTPException(status_code=400, detail="Message too long (max 4000 characters)")

    try:
        # 1 — Synthia Gateway (OpenAI-compatible, fastest real AI)
        result = await call_synthia_gateway(message)
        if result:
            return ChatResponse(response=result, provider="synthia", timestamp=datetime.now().isoformat())

        # 2 — Mercury Inception Labs
        if request.providers.mercury:
            result = await call_mercury_api(message)
            if result:
                return ChatResponse(response=f"💎 {result}", provider="mercury", timestamp=datetime.now().isoformat())

        # 3 — NVIDIA NIM proxy
        if request.providers.nvidia:
            result = await call_nim_proxy(message)
            if result:
                return ChatResponse(response=f"🚀 {result}", provider="nvidia-nim", timestamp=datetime.now().isoformat())

        raise HTTPException(status_code=503, detail="No AI provider available. Check API keys in ~/.hermes/.env")

    except HTTPException:
        raise
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Agent error: {str(e)}")


async def call_synthia_gateway(message: str) -> str | None:
    """
    Call Synthia Gateway (OpenAI-compatible BYOK proxy).
    Routes to OpenAI, Groq, Anthropic, etc. based on SYNTHIA_MODEL.
    Set SYNTHIA_GATEWAY_URL and SYNTHIA_GATEWAY_KEY in ~/.hermes/.env.
    """
    gateway_url = os.getenv("SYNTHIA_GATEWAY_URL", "http://localhost:3000")
    gateway_key = os.getenv("SYNTHIA_GATEWAY_KEY") or os.getenv("OPENAI_API_KEY")
    if not gateway_key:
        return None

    model = os.getenv("SYNTHIA_MODEL", "gpt-4o-mini")

    headers = {
        "Authorization": f"Bearer {gateway_key}",
        "Content-Type": "application/json",
    }

    try:
        async with httpx.AsyncClient(timeout=15.0) as client:
            resp = await client.post(
                f"{gateway_url}/v1/chat/completions",
                headers=headers,
                json={
                    "model": model,
                    "messages": [
                        {"role": "system", "content": HERMES_SYSTEM_PROMPT},
                        {"role": "user", "content": message},
                    ],
                    "max_tokens": 300,
                    "temperature": 0.7,
                },
            )
            if resp.status_code == 200:
                data = resp.json()
                return data["choices"][0]["message"]["content"].strip()
            print(f"Synthia Gateway error {resp.status_code}: {resp.text[:200]}")
    except Exception as e:
        print(f"Synthia Gateway unreachable: {e}")
    return None


async def call_mercury_api(message: str) -> str | None:
    """Call Mercury Inception Labs diffusion model."""
    token = os.getenv("MERCURY_API_KEY")
    if not token:
        return None

    try:
        async with httpx.AsyncClient(timeout=20.0) as client:
            resp = await client.post(
                "https://api.inceptionlabs.ai/v1/chat/completions",
                headers={"Authorization": f"Bearer {token}"},
                json={
                    "model": "mercury-coder-small",
                    "messages": [
                        {"role": "system", "content": HERMES_SYSTEM_PROMPT},
                        {"role": "user", "content": message},
                    ],
                    "max_tokens": 300,
                },
            )
            if resp.status_code == 200:
                data = resp.json()
                return data["choices"][0]["message"]["content"].strip()
            print(f"Mercury error {resp.status_code}: {resp.text[:200]}")
    except Exception as e:
        print(f"Mercury API error: {e}")
    return None


async def call_nim_proxy(message: str) -> str | None:
    """Call NVIDIA NIM proxy (moonshotai/kimi-k2-thinking, free tier)."""
    nim_url = os.getenv("NIM_PROXY_URL", "http://localhost:8082")
    nim_key = os.getenv("NVIDIA_NIM_API_KEY", "dummy")

    try:
        async with httpx.AsyncClient(timeout=30.0) as client:
            resp = await client.post(
                f"{nim_url}/v1/messages",
                headers={
                    "Authorization": f"Bearer {nim_key}",
                    "anthropic-version": "2023-06-01",
                    "Content-Type": "application/json",
                },
                json={
                    "model": "claude-3-5-sonnet-20241022",
                    "max_tokens": 300,
                    "system": HERMES_SYSTEM_PROMPT,
                    "messages": [{"role": "user", "content": message}],
                },
            )
            if resp.status_code == 200:
                data = resp.json()
                content = data.get("content", [])
                if content:
                    return content[0].get("text", "").strip()
            print(f"NIM proxy error {resp.status_code}: {resp.text[:200]}")
    except Exception as e:
        print(f"NIM proxy error: {e}")
    return None


@app.get("/api/status")
async def status(_: None = Depends(verify_api_key)):
    gateway_url = os.getenv("SYNTHIA_GATEWAY_URL", "http://localhost:3000")
    has_synthia = bool(os.getenv("SYNTHIA_GATEWAY_KEY") or os.getenv("OPENAI_API_KEY"))
    has_mercury = bool(os.getenv("MERCURY_API_KEY"))
    has_nim = bool(os.getenv("NVIDIA_NIM_API_KEY"))
    return {
        "agent": "Hermes",
        "version": "2.0.0",
        "status": "active",
        "providers": {
            "synthia_gateway": {"url": gateway_url, "ready": has_synthia},
            "mercury": {"ready": has_mercury},
            "nvidia_nim": {"ready": has_nim},
        },
        "features": ["voice_control", "memory_recall", "action_execution", "synthia_gateway"],
    }


@app.get("/api/hostinger/vps")
async def hostinger_vps_list(_: None = Depends(verify_api_key)):
    try:
        api_key = os.getenv("HOSTINGER_API_KEY")
        if not api_key:
            raise HTTPException(status_code=400, detail="HOSTINGER_API_KEY not configured")
        async with httpx.AsyncClient(timeout=30.0) as client:
            resp = await client.get(
                "https://api.hostinger.com/v1/vps",
                headers={"Authorization": f"Bearer {api_key}"},
            )
            if resp.status_code == 200:
                return resp.json()
            raise HTTPException(status_code=resp.status_code, detail="Hostinger API error")
    except HTTPException:
        raise
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"VPS list error: {str(e)}")


@app.get("/api/hostinger/domains")
async def hostinger_domains_list(_: None = Depends(verify_api_key)):
    try:
        api_key = os.getenv("HOSTINGER_API_KEY")
        if not api_key:
            raise HTTPException(status_code=400, detail="HOSTINGER_API_KEY not configured")
        async with httpx.AsyncClient(timeout=30.0) as client:
            resp = await client.get(
                "https://api.hostinger.com/v1/domains",
                headers={"Authorization": f"Bearer {api_key}"},
            )
            if resp.status_code == 200:
                return resp.json()
            raise HTTPException(status_code=resp.status_code, detail="Hostinger API error")
    except HTTPException:
        raise
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Domains error: {str(e)}")


@app.get("/api/hostinger/account")
async def hostinger_account_info(_: None = Depends(verify_api_key)):
    try:
        api_key = os.getenv("HOSTINGER_API_KEY")
        if not api_key:
            raise HTTPException(status_code=400, detail="HOSTINGER_API_KEY not configured")
        async with httpx.AsyncClient(timeout=30.0) as client:
            resp = await client.get(
                "https://api.hostinger.com/v1/account",
                headers={"Authorization": f"Bearer {api_key}"},
            )
            if resp.status_code == 200:
                return resp.json()
            raise HTTPException(status_code=resp.status_code, detail="Hostinger API error")
    except HTTPException:
        raise
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Account error: {str(e)}")


if __name__ == "__main__":
    import uvicorn
    port = int(os.getenv("API_SERVER_PORT", 8642))
    host = os.getenv("API_SERVER_HOST", "0.0.0.0")
    print(f"🚀 Hermes Agent API v2 starting on {host}:{port}")
    print(f"   Synthia Gateway: {os.getenv('SYNTHIA_GATEWAY_URL', 'http://localhost:3000')}")
    uvicorn.run(app, host=host, port=port, log_level="info")
