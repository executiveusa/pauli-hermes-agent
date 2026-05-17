# MCP Inventory

## Current Hermes Core

- MCP feature surface exists in upstream Hermes codebase
- No external Pauli MCP packages were installed during this run

## Candidate Integrations Requested

- `jgravelle/jcodemunch-mcp`
- `modelcontextprotocol/ext-apps`
- `knowsuchagency/mcp2cli`
- `supabase-community/supabase-mcp`
- `browser-use/browser-harness`

## Current Block

MCP expansion is deferred behind two blockers:

1. Core Hermes chat path is not serving real completions because provider auth fails at runtime.
2. Hermes Desktop cannot complete its local dependency/toolchain validation yet, so there is no stable GUI surface to expose new MCP health/actions.
