# Agents Workspace Context
# Load this when: spawning sub-agents, creating PRDs, orchestrating workflows

## Orchestration Pattern (Vibe Graphing)
1. `/graph <intent>` → JSON topology blueprint
2. Review and approve node roles
3. `delegate_task` → executes the graph
4. Trace2Skill patch loop if failures

## Known Sub-Agents
| Agent | Purpose | Language |
|-------|---------|---------|
| coding-agent | Receives PRDs, implements features | EN |
| ivette-agent | Kupuri/CDMX tasks | CDMX Spanish |
| archonx-agent | Business automation, Archon X | EN |
| research-agent | Read-only codebase exploration | EN |
| browser-agent | Browserbase + Playwright testing | EN |
| trace-analyst | Post-session skill evolution | EN |

## PRD Handoff Format
Save to: `workspace/agents/handoffs/<project>/PRD.md`
Structure: Goal → Context → User Stories → Tech Spec → Success Criteria

## Graph Blueprints
Reusable JSON graphs: `workspace/agents/graph-blueprints/`
- `literature-review.json` — retriever → reader → synthesizer → critic
- `lead-gen.json` — scraper → enricher → writer → sender
- `site-build.json` — designer → developer → tester → deployer

## Systems This Agent Controls
- **Archon X** (`executiveusa/archonx-os`) — Pauli's business OS
- **AKASHPORTFOLIO** (`executiveusa/AKASHPORTFOLIO`) — Ivette's ecosystem
- **pauli-comic-funnel** — Comic funnel pipeline
