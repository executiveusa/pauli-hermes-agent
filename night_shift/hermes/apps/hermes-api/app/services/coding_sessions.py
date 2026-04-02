from uuid import uuid4
def create_coding_session(payload: dict) -> dict:
    return {"session_id": f"cs_{uuid4().hex[:12]}", "status": "queued", "payload": payload}
