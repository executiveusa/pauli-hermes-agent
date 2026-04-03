from __future__ import annotations

import os
import httpx
from .models import ProviderProfile


DEFAULT_PROFILES: dict[str, ProviderProfile] = {
    "venice_primary": ProviderProfile(
        name="venice_primary",
        base_url=os.getenv("VENICE_BASE_URL", "https://api.venice.ai/v1"),
        model=os.getenv("VENICE_MODEL", "venice-uncensored"),
        api_key_env="VENICE_API_KEY",
        max_cost_per_1k=0.006,
        budget_cap_usd=25,
    ),
    "gemini_flash_primary": ProviderProfile(
        name="gemini_flash_primary",
        base_url=os.getenv("GEMINI_BASE_URL", "https://generativelanguage.googleapis.com/v1beta/openai"),
        model=os.getenv("GEMINI_MODEL", "gemini-2.5-flash"),
        api_key_env="GEMINI_API_KEY",
        max_cost_per_1k=0.001,
        budget_cap_usd=20,
    ),
    "openai_premium_planning": ProviderProfile(
        name="openai_premium_planning",
        base_url=os.getenv("OPENAI_BASE_URL", "https://api.openai.com/v1"),
        model=os.getenv("OPENAI_MODEL", "gpt-5"),
        api_key_env="OPENAI_API_KEY",
        max_cost_per_1k=0.015,
        budget_cap_usd=30,
    ),
    "model_fallback": ProviderProfile(
        name="model_fallback",
        base_url=os.getenv("OPENAI_BASE_URL", "https://api.openai.com/v1"),
        model=os.getenv("OPENAI_MODEL_FALLBACK", "gpt-4.1-mini"),
        api_key_env="OPENAI_API_KEY",
        max_cost_per_1k=0.002,
        budget_cap_usd=10,
    ),
}


class BudgetExceededError(RuntimeError):
    pass


async def chat_completion(profile_name: str, payload: dict, spent_usd: float = 0.0) -> dict:
    profile = DEFAULT_PROFILES.get(profile_name, DEFAULT_PROFILES["model_fallback"])
    if spent_usd > profile.budget_cap_usd:
        raise BudgetExceededError(f"Budget exceeded for {profile.name}")

    api_key = os.getenv(profile.api_key_env)
    if not api_key:
        raise RuntimeError(f"Missing key: {profile.api_key_env}")

    body = {
        "model": profile.model,
        "messages": payload["messages"],
        "temperature": payload.get("temperature", 0.2),
        "max_tokens": payload.get("max_tokens", 1024),
    }
    if payload.get("tools"):
        body["tools"] = payload["tools"]

    async with httpx.AsyncClient(timeout=90) as client:
        resp = await client.post(
            f"{profile.base_url.rstrip('/')}/chat/completions",
            headers={"Authorization": f"Bearer {api_key}"},
            json=body,
        )
        resp.raise_for_status()
        return resp.json()
