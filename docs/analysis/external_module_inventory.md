# External Module Inventory

Date: `2026-05-07`

## Cloned Into `E:\ACTIVE PROJECTS-PIPELINE\vendor-repos`

| Module | Path | Commit | Class | Notes |
|---|---|---:|---|---|
| OpenChronicle | `E:\ACTIVE PROJECTS-PIPELINE\vendor-repos\OpenChronicle` | `d780c62` | future adapter only | Useful local-first memory layer, but upstream README currently marks it macOS-only and early alpha. |
| Ralphy | `E:\ACTIVE PROJECTS-PIPELINE\vendor-repos\ralphy` | `506eea0` | reusable orchestration layer | Strong fit for PRD/task execution and engine switching; should be adapted into Pauli flywheel wrappers rather than vendored into Hermes core. |
| jcodemunch-mcp | `E:\ACTIVE PROJECTS-PIPELINE\vendor-repos\jcodemunch-mcp` | `115a0b7` | MCP server | Strong token-saver for repo retrieval; commercial licensing applies for business use. |
| browser-harness | `E:\ACTIVE PROJECTS-PIPELINE\vendor-repos\browser-harness` | `2106221` | browser automation harness | Primary browser-control path for Pauli browser ops and dynamic dashboard work. |
| ext-apps | `E:\ACTIVE PROJECTS-PIPELINE\vendor-repos\ext-apps` | cloned | MCP apps | Candidate app-control bridge layer. |
| mcp2cli | `E:\ACTIVE PROJECTS-PIPELINE\vendor-repos\mcp2cli` | cloned | MCP bridge | Candidate CLI bridge for MCP services. |
| supabase-mcp | `E:\ACTIVE PROJECTS-PIPELINE\vendor-repos\supabase-mcp` | cloned | MCP server | Candidate Supabase operations layer for memory/search. |
| mattpocock-skills | `E:\ACTIVE PROJECTS-PIPELINE\vendor-repos\mattpocock-skills` | cloned | skill library | Reference skill library for course/test/dev workflows. |

## Immediate Integration Decisions

- `browser-harness`: install and use now.
- `jcodemunch-mcp`: integrate now, but keep license note visible for commercial workflows.
- `Ralphy`: integrate as an orchestration adapter in the flywheel layer.
- `OpenChronicle`: integrate as a preferred optional memory/browser-context layer on supported macOS hosts; document as platform-blocked on this Windows machine.

## Still Pending Ingestion

- `zarazhangrui/codebase-to-course` or equivalent course skill source
- `bradautomates/claude-video`
- public Pauli-side repos that are reachable over HTTPS

## Access Caveats

- Repos only provided as `git@github.com:` may require GitHub SSH or alternate HTTPS access.
- OpenChronicle cannot be honestly marked active on the current Windows host until its platform support changes or a compatible adapter exists.
