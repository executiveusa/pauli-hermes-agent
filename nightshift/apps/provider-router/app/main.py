"""Agent MAXX — Provider Router: Model-agnostic LLM proxy."""

import json
import logging
import os
from typing import Optional

import httpx
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel

logger = logging.getLogger(__name__)

app = FastAPI(title="Agent MAXX Provider Router", version="1.0.0")


# ── Provider Adapters ────────────────────────────────────────


class LLMRequest(BaseModel):
    provider_profile_id: str = "balanced"
    messages: list[dict]
    temperature: float = 0.7
    max_tokens: int = 4096
    tools: Optional[list[dict]] = None


class OpenAICompatibleProvider:
    def __init__(self, base_url: str, api_key: str, model: str, is_anthropic: bool = False):
        self.base_url = base_url
        self.api_key = api_key
        self.model = model
        self.is_anthropic = is_anthropic

    async def chat(self, req: LLMRequest) -> dict:
        if self.is_anthropic:
            return await self._anthropic_chat(req)
        headers = {"Authorization": f"Bearer {self.api_key}", "Content-Type": "application/json"}
        body = {"model": self.model, "messages": req.messages,
                "temperature": req.temperature, "max_tokens": req.max_tokens}
        if req.tools:
            body["tools"] = req.tools
        async with httpx.AsyncClient(timeout=120.0) as client:
            resp = await client.post(f"{self.base_url}/chat/completions", headers=headers, json=body)
        if resp.status_code != 200:
            raise HTTPException(502, f"Provider error: {resp.status_code} {resp.text[:200]}")
        data = resp.json()
        return {"response_text": data["choices"][0]["message"].get("content", ""),
                "usage": data.get("usage", {}), "model": self.model}

    async def _anthropic_chat(self, req: LLMRequest) -> dict:
        headers = {"x-api-key": self.api_key, "content-type": "application/json", "anthropic-version": "2023-06-01"}
        system_msg = ""
        messages = []
        for m in req.messages:
            if m["role"] == "system":
                system_msg = m["content"]
            else:
                messages.append(m)
        body = {"model": self.model, "messages": messages, "max_tokens": req.max_tokens, "temperature": req.temperature}
        if system_msg:
            body["system"] = system_msg
        async with httpx.AsyncClient(timeout=120.0) as client:
            resp = await client.post(f"{self.base_url}/messages", headers=headers, json=body)
        if resp.status_code != 200:
            raise HTTPException(502, f"Anthropic error: {resp.status_code} {resp.text[:200]}")
        data = resp.json()
        text = "".join(b.get("text", "") for b in data.get("content", []) if b.get("type") == "text")
        return {"response_text": text, "usage": data.get("usage", {}), "model": self.model}


class GeminiProvider:
    def __init__(self, api_key: str, model: str = "gemini-2.5-flash"):
        self.api_key = api_key
        self.model = model

    async def chat(self, req: LLMRequest) -> dict:
        url = f"https://generativelanguage.googleapis.com/v1beta/models/{self.model}:generateContent?key={self.api_key}"
        parts = [{"text": m["content"]} for m in req.messages]
        body = {"contents": [{"parts": parts}], "generationConfig": {"temperature": req.temperature, "maxOutputTokens": req.max_tokens}}
        async with httpx.AsyncClient(timeout=120.0) as client:
            resp = await client.post(url, json=body)
        if resp.status_code != 200:
            raise HTTPException(502, f"Gemini error: {resp.status_code}")
        data = resp.json()
        text = data.get("candidates", [{}])[0].get("content", {}).get("parts", [{}])[0].get("text", "")
        return {"response_text": text, "usage": data.get("usageMetadata", {}), "model": self.model}


# ── Profile Resolution ───────────────────────────────────────


def build_profiles() -> dict:
    profiles = {}
    anthropic_key = os.getenv("ANTHROPIC_API_KEY", "")
    openai_key = os.getenv("OPENAI_API_KEY", "")
    google_key = os.getenv("GOOGLE_API_KEY", "")
    openrouter_key = os.getenv("OPENROUTER_API_KEY", "")

    if anthropic_key:
        profiles["balanced"] = OpenAICompatibleProvider("https://api.anthropic.com/v1", anthropic_key, os.getenv("DEFAULT_MODEL_BALANCED", "claude-sonnet-4-20250514"), is_anthropic=True)
        profiles["premium_reasoning"] = OpenAICompatibleProvider("https://api.anthropic.com/v1", anthropic_key, os.getenv("DEFAULT_MODEL_PREMIUM", "claude-opus-4-20250514"), is_anthropic=True)
        profiles["low_cost_coding"] = OpenAICompatibleProvider("https://api.anthropic.com/v1", anthropic_key, os.getenv("DEFAULT_MODEL_CODING", "claude-sonnet-4-20250514"), is_anthropic=True)
    if google_key:
        profiles["research_browser"] = GeminiProvider(google_key, os.getenv("DEFAULT_MODEL_RESEARCH", "gemini-2.5-flash"))
        profiles["gemini_flash"] = GeminiProvider(google_key, "gemini-2.5-flash")
    if openai_key:
        profiles["openai_gpt4"] = OpenAICompatibleProvider("https://api.openai.com/v1", openai_key, "gpt-4o")
    if openrouter_key:
        profiles["openrouter_balanced"] = OpenAICompatibleProvider("https://openrouter.ai/api/v1", openrouter_key, "anthropic/claude-sonnet-4-20250514")

    return profiles


PROFILES = build_profiles()


# ── Routes ───────────────────────────────────────────────────


@app.get("/health")
async def health():
    return {"status": "ok", "profiles": list(PROFILES.keys())}


@app.post("/v1/llm/chat")
async def llm_chat(req: LLMRequest):
    provider = PROFILES.get(req.provider_profile_id)
    if not provider:
        available = list(PROFILES.keys())
        if not available:
            raise HTTPException(503, "No provider profiles configured — add API keys to .env")
        # Fallback to first available
        provider = PROFILES[available[0]]
        logger.warning(f"Profile '{req.provider_profile_id}' not found, falling back to '{available[0]}'")
    return await provider.chat(req)
