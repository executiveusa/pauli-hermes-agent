# Stage 01 — Scrape & Graph

## Purpose

Scrape the target YouTube channel for design knowledge, then convert all scraped data
into a knowledge graph (nodes + edges) so reasoning can operate over the whole corpus —
not just individual videos.

## Input

From Stage 00:
- `runs/current/brief.json` (specifically: any reference channels/URLs in `references`)

Default scrape target (always included):
- `https://www.youtube.com/@bycrawford/videos` — 20 videos minimum

Additional channels from brief if provided.

## Process

### Step 1: Scrape (delegated to `subagents/scraper/`)

Uses `skills/youtube-channel-scraper` skill:
```
stage_runner.py "scrape https://www.youtube.com/@bycrawford/videos"
```

Config:
- `max_videos: 20`
- `fetch_descriptions: true`
- `rate_limit: 40/min`

Outputs raw data to `runs/current/stage_01_raw.jsonl`

### Step 2: Build Knowledge Graph (delegated to `subagents/analyzer/`)

Convert raw video list to graph:

**Nodes** (one per entity):
```json
{
  "id": "unique-id",
  "type": "video | technique | tool | concept | principle",
  "label": "human-readable name",
  "properties": {}
}
```

**Edges** (relationships between nodes):
```json
{
  "from": "node-id",
  "to": "node-id",
  "relationship": "uses | demonstrates | requires | contrasts_with | extends"
}
```

Graph rules:
- Every video → node
- Every tool mentioned (GSAP, Three.js, Lenis) → node
- Every technique (parallax, depth planes, camera drift) → node
- Every edge captures a *directional relationship*
- Minimum 3 edges per video node

Output: `runs/current/graph.json`

### Step 3: Validate Graph

Count nodes and edges. Verify minimum thresholds.

## Output

- `runs/current/stage_01_raw.jsonl` — Raw scraped video entries
- `runs/current/graph.json` — Knowledge graph (nodes + edges)
- `runs/current/receipts/stage_01.json` — Receipt

## Gate

**PASS** if:
- `stage_01_raw.jsonl` has ≥15 entries
- `graph.json` has ≥30 nodes and ≥45 edges

**BLOCK** if:
- Fewer than 15 videos scraped (rate-limit or access issue) → retry once with 60s backoff
- Graph has insufficient nodes/edges → expand scrape to more videos before graphing

## OpenSpec Handoff to Stage 02

Written to `runs/current/handoffs/01_to_02.json`:
```json
{
  "spec_version": "1.0",
  "from_agent": "scraper+analyzer",
  "to_agent": "synthesizer",
  "stage": "01_scrape_and_graph",
  "payload": {
    "type": "knowledge_graph",
    "path": "runs/current/graph.json",
    "node_count": "<actual>",
    "edge_count": "<actual>",
    "video_count": "<actual>"
  },
  "gate_status": "PASS",
  "timestamp": "ISO"
}
```

## Next Stage

→ `stages/02_synthesize_laws/CONTEXT.md`

## Collision Check

- youtube-channel-scraper skill: no collision (this workflow calls it as a sub-step)
- No overlap with other workflows (graph output is scroll-world-design specific)
