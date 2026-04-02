from fastapi import FastAPI
from app.routes import router
app=FastAPI(title="provider-router",version="0.1.0")
app.include_router(router)
@app.get("/healthz")
def healthz(): return {"ok": True}
@app.get("/readyz")
def readyz(): return {"ready": True}
@app.get("/version")
def version(): return {"service": "provider-router", "version": "0.1.0"}
