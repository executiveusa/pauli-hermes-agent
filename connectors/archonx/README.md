# Archon X — External Agent Connectors

This directory contains configuration and documentation for connecting
external agents and services to the Archon X orchestration layer.

## Connected Agents

| Agent | Repo | Status |
|-------|------|--------|
| Hermes | pauli-hermes-agent (this repo) | Active |
| Agent Zero | docker compose service (:8643) | Active |
| Darya | github.com/kupuri/darya-designs | Planned |
| Pi | github.com/kupuri/pi_agent_rust | Planned |
| Humanizer | github.com/kupuri/humanizer | Planned |

## MCP Servers

MCP servers are configured in `~/.hermes/config.yaml` under the `mcp` key.

### Active
- **supabase-mcp** — Supabase management (schema, data, auth)
- **context-mode** — Context management for agent sessions

### Planned
- **stitch-mcp** — Generative UI component stitching
- **ext-apps** — External application integration

## Adding a New Agent

1. Add entry to `agents/registry.yaml` with name, endpoint, type, capabilities
2. If remote: configure SSH or API access
3. If Docker: add service to `docker-compose.yml`
4. Register health check in the API server
5. Update the dashboard agent grid (auto from registry.yaml)
