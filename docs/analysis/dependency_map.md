# Dependency Map

## Package Managers
- Python: `pyproject.toml`, `requirements.txt`, `uv.lock`
- Node: `package.json`, `package-lock.json`, `ui-tui/package.json`
- Nix: `flake.nix`, `flake.lock`

## Core Runtime Layers
- Kernel/agent loop: `run_agent.py`, `model_tools.py`, `toolsets.py`
- Tooling: `tools/` registry + handlers
- CLI: `cli.py`, `hermes_cli/`
- Messaging Gateway: `gateway/`
- TUI: `ui-tui/` + `tui_gateway/`
- Web Dashboard/API: `hermes_cli/web_server.py`, `web/`

## Key External Integrations Detected
- GitHub API usage in skills publishing and auth/model flows (`hermes_cli/skills_hub.py`, `hermes_cli/auth.py`, `hermes_cli/models.py`)
- Twilio SMS platform adapter (`gateway/platforms/sms.py`)
- Webhook ingestion (`gateway/platforms/webhook.py`, `hermes_cli/webhook.py`)
- Vercel AI Gateway provider aliasing (`hermes_cli/providers.py`, `hermes_cli/auth.py`)

## Not Yet First-class in Core
- No dedicated Infisical adapter module found
- No dedicated Vercel deployment operator module found
- No dedicated GitHub repo/workflow operator module in `tools/` found (MCP can bridge externally)
