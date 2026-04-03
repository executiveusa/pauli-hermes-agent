"""Agent MAXX — API key verification middleware."""

import os

from fastapi import HTTPException, Security
from fastapi.security import APIKeyHeader

api_key_header = APIKeyHeader(name="X-API-Key", auto_error=False)


async def verify_api_key(key: str = Security(api_key_header)):
    expected = os.getenv("API_KEY", "")
    if not expected:
        return  # Dev mode — no key required
    if key != expected:
        raise HTTPException(status_code=401, detail="Invalid API key")
