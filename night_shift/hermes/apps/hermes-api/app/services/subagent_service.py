def create_subagent(payload: dict) -> dict:
    return {"subagent_id": payload.get("subagent_id"), "status": "active", "policy_enforced": True}
