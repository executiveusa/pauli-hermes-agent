from uuid import uuid4
def create_browser_run(payload: dict) -> dict:
    return {"browser_run_id": f"br_{uuid4().hex[:12]}", "status": "queued", "payload": payload}
