# Service Topology

## Processes
1. CLI/TUI entry (`hermes_cli/main.py`, `cli.py`, `ui-tui`)
2. Agent kernel (`run_agent.py`)
3. Tool execution layer (`tools/*`, `tools/terminal_tool.py`)
4. Optional gateway daemon (`gateway/run.py`)
5. Optional web dashboard server (`hermes_cli/web_server.py`)

## Data Stores
- SQLite session DB (`hermes_state.py`)
- Profile-scoped home (`HERMES_HOME`) for config, skills, caches
- Plugin and skill directories (`plugins/`, `skills/`, `~/.hermes/skills`)

## Ingress/Egress
- Messaging adapters (Telegram/Discord/Slack/WhatsApp/SMS/etc.)
- Webhooks (generic + provider-specific)
- MCP servers (external capability expansion)
- External model providers and tool APIs
