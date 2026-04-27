# Service Topology (Snapshot: 2026-04-23)

## Core Runtime Topology
1. **Entry surfaces**: CLI (`hermes_cli/main.py`), classic interactive shell (`cli.py`), optional TUI (`ui-tui`).
2. **Agent kernel**: `AIAgent` loop (`run_agent.py`) with tool orchestration (`model_tools.py`).
3. **Capability plane**: tool registry + handlers (`tools/`).
4. **External ingress plane**: messaging/webhook gateway (`gateway/run.py`, `gateway/platforms/*`).
5. **Operator UI plane**: dashboard server (`hermes_cli/web_server.py`) and web frontend (`web/`).

## Data + State
- Session persistence: SQLite via `hermes_state.py`.
- Profile-scoped persistent state: `HERMES_HOME` (config, auth, skills, memory).
- Plugin/skills assets: `plugins/`, `skills/`, optional user skills under home.

## External Planes (Current vs Target)
- **Current**: model providers, messaging channels, generic webhooks, MCP.
- **Target additions (not fully implemented yet)**: GitHub operator adapter, Vercel operator adapter, Infisical secret control plane, Twilio voice pipeline.

## Deployment Surfaces
- Docker image path exists (`Dockerfile`, `docker/entrypoint.sh`).
- CI workflow exists for image publishing (`.github/workflows/docker-publish.yml`).
- Hostinger/Coolify runbook exists, but environment-specific deployment credentials and stack details remain external inputs.
