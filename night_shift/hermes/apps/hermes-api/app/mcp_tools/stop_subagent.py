def handler(payload: dict) -> dict:
    return {"tool":"stop_subagent","payload":payload,"status":"ok"}
