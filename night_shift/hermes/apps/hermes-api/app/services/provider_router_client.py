import os, httpx
async def chat(payload: dict) -> dict:
    base = os.getenv("PROVIDER_ROUTER_URL", "http://provider-router:8080")
    async with httpx.AsyncClient(timeout=60) as client:
        r = await client.post(f"{base}/v1/llm/chat", json=payload); r.raise_for_status(); return r.json()
