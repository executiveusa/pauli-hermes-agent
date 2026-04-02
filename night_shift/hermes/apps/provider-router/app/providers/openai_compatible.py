async def chat(payload: dict) -> dict:
    return {"provider_type": "openai_compatible", "model": "configured", "response_text": "stub", "tool_calls": [], "token_usage_json": {}, "raw_metadata_json": {"adapter": "openai_compatible"}}
