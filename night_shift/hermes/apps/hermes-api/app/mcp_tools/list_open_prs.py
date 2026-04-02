def handler(payload: dict) -> dict:
    return {"tool":"list_open_prs","payload":payload,"status":"ok"}
