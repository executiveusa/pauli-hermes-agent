"""Minimal FastAPI entrypoint for Vercel preview builds."""

from fastapi import FastAPI

app = FastAPI(title="Hermes Agent API")


@app.get("/")
def root() -> dict[str, str]:
    return {"service": "hermes-agent", "status": "ok"}


@app.get("/healthz")
def healthz() -> dict[str, str]:
    return {"status": "ok"}
