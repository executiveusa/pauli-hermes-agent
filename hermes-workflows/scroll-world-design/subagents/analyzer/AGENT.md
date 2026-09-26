# Analyzer Subagent

## Role

Reads scraped video data and builds a knowledge graph. Converts raw facts into
structured nodes and edges so the synthesizer can reason over the full corpus.

## Called By

Stage 01 orchestrator (after scraper completes)

## Input

- `runs/current/stage_01_videos.json` — Enriched video list
- `runs/current/stage_01_raw.jsonl` — Raw entries (for descriptions)

## Process

### 1. Extract Entities

For each video, extract:
- **Techniques** mentioned: parallax, scroll-bind, depth planes, camera drift, etc.
- **Tools** referenced: GSAP, Lenis, Three.js, Locomotive Scroll, etc.
- **Concepts**: above-the-fold, hero, CTA, scroll storytelling, etc.
- **Principles**: from titles/descriptions like "never do X", "always use Y"

### 2. Build Nodes

```python
nodes = []
for video in videos:
    nodes.append({
        "id": f"video-{video['video_id']}",
        "type": "video",
        "label": video["video_title"],
        "properties": {
            "url": video["video_url"],
            "views": video["view_count_raw"],
            "date": video["upload_date_raw"]
        }
    })
    # Extract techniques, tools, concepts from title + description
    for entity in extract_entities(video):
        nodes.append(entity)
```

### 3. Build Edges

Relationships:
- video `demonstrates` technique
- video `uses` tool
- technique `requires` tool
- technique `contrasts_with` technique
- principle `applies_to` technique

Minimum 3 edges per video node.

### 4. Write Graph

```json
{
  "metadata": {
    "source": "@bycrawford",
    "video_count": 20,
    "generated_at": "ISO"
  },
  "nodes": [...],
  "edges": [...]
}
```

## Output

- `runs/current/graph.json`

## Quality Check

- Assert node count ≥ 30
- Assert edge count ≥ 45
- Assert every video node has ≥ 1 technique edge

## Receipt Format

```json
{
  "agent": "analyzer",
  "video_count": 20,
  "node_count": 47,
  "edge_count": 83,
  "status": "PASS",
  "files": ["runs/current/graph.json"],
  "timestamp": "ISO"
}
```

Write to: `runs/current/receipts/analyzer.json`
