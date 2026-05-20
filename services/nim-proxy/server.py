"""
Claude Code Proxy - Entry Point

Minimal entry point that imports the app from the api module.
Run with: uv run uvicorn server:app --host 0.0.0.0 --port 8082
"""

from api.app import app, create_app

__all__ = ["app", "create_app"]

if __name__ == "__main__":
    import uvicorn
    from config.settings import get_settings

    settings = get_settings()
    uvicorn.run(app, host=settings.host, port=settings.port, log_level="debug")
