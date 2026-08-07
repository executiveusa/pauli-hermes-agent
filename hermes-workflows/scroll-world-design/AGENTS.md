# Scroll World Design — Agent Roles

## Agent Roster

| Agent | Role | Stage | Subagent Dir |
|-------|------|-------|-------------|
| **scraper** | Hits YouTube, extracts video data, builds knowledge graph | 01 | `subagents/scraper/` |
| **analyzer** | Reasons over knowledge graph, identifies patterns | 01→02 | `subagents/analyzer/` |
| **synthesizer** | Converts patterns + repo laws → immutable design laws | 02 | `subagents/synthesizer/` |
| **builder** | Generates site code: HTML/CSS/JS, GSAP animations | 04 | `subagents/builder/` |
| **judge-ux** | Reviews UX quality against rubric | 05 | `subagents/judge-ux/` |
| **judge-perf** | Reviews performance (LCP, CLS, FPS) against rubric | 05 | `subagents/judge-perf/` |
| **judge-design** | Reviews visual design quality against rubric | 05 | `subagents/judge-design/` |
| **handoff** | Formats and validates OpenSpec handoffs between agents | all | `subagents/handoff/` |

## Orchestrator

The main workflow orchestrator (Hermes) reads `CONTEXT.md` to know which stage to run,
then delegates to the appropriate subagent. Orchestrator does not do the work itself —
it routes, validates gates, and surfaces results.

## Agent Communication Protocol

All inter-agent messages use OpenSpec format:

```json
{
  "spec_version": "1.0",
  "from_agent": "scraper",
  "to_agent": "analyzer",
  "stage": "01_scrape_and_graph",
  "payload": {
    "type": "knowledge_graph",
    "path": "runs/current/graph.json",
    "node_count": 47,
    "edge_count": 83
  },
  "gate_status": "PASS",
  "timestamp": "2026-08-07T00:00:00Z"
}
```

All handoffs are validated by the **handoff** subagent before the receiving agent proceeds.

## Authority

- **scraper** — scrapes only URLs provided in stage input; never autonomous browsing
- **builder** — generates code only; never deploys without explicit gate PASS from all 3 judges
- **judges** — read-only; they score but do not modify; their verdict is final

## Auto-activation Triggers

Hermes routes to this workflow when request contains:

- "build" + ("scroll site" / "landing page" / "parallax" / "3D website" / "cinematic")
- "design" + ("web page" / "scroll animation" / "scroll website")
- "/scroll-world-design"
