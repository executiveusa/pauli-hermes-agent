from uuid import uuid4
def provision_project(payload: dict) -> dict:
    return {"project_ref": f"awp_{uuid4().hex[:10]}", "status": "provisioning", "payload": payload}
