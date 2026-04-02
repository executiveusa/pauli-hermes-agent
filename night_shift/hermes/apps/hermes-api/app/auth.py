from fastapi import Header, HTTPException
def require_operator(x_operator_token: str | None = Header(default=None)):
    if not x_operator_token: raise HTTPException(status_code=401, detail="missing operator token")
    return {"operator": "authorized"}
