# Synthesizer Subagent

## Role

Reads the knowledge graph and lazy-loaded repo teachings, then writes immutable
design laws. These laws govern all subsequent stages — no law can be overridden.

## Called By

Stage 02 orchestrator

## Input

- `runs/current/graph.json`
- `runs/current/brief.json`
- Reference repos (loaded lazily — only README and key files)

## Process

### 1. Cluster Graph

Group nodes by type:
- Technique cluster: what scroll techniques appear most?
- Tool cluster: what libraries are always co-present?
- Principle cluster: what does Crawford explicitly teach?

### 2. Load Repos (in order)

Load each repo by fetching its README and 2-3 key files. Extract the core teaching.
Log each load as a receipt.

Priority order (load all, but in this order):
1. `greensock/GSAP` → animation axioms
2. `oso95/scroll-world` → scroll binding patterns
3. `emilkowalski/skills` → interaction quality bar
4. `pbakaus/impeccable` → design constraint philosophy
5. `gastownhall/beads` → atomic rollback pattern
6. `robonuggets/cinematic-site-components` → depth plane techniques
7. `ihlamury/design-skills` → hierarchy and rhythm
8. `ytx-readings/design-ui-ux` → anti-patterns
9. `darula-hpp/uigen` → component generation
10. `safishamsi/graphify` → graph-to-insight
11. `Fission-AI/OpenSpec` → handoff format
12. `willseltzer/claude-handoff` → agent chain-of-custody
13. `vercel-labs/opensrc` → deploy patterns
14. `atomicdotdev/atomic` → token → component → template

### 3. Write Design Laws

Each law = graph pattern + repo teaching merged.

Example laws:
```json
[
  {
    "id": "law-01",
    "title": "All animation uses transform and opacity only",
    "statement": "Never animate width, height, top, left, margin, or padding — only transform and opacity trigger GPU compositing.",
    "rationale": "Layout-thrashing properties cause jank and fail the 60fps requirement.",
    "source_repos": ["greensock/GSAP"],
    "source_graph_nodes": ["concept-60fps", "technique-gpu-compositing"],
    "beads_required": false
  },
  {
    "id": "law-02",
    "title": "Every scroll section has a defined depth plane",
    "statement": "Foreground, midground, and background must have distinct parallax multipliers (1.0, 0.6, 0.3).",
    "rationale": "Depth planes create the cinematic effect Crawford demonstrates in every high-view video.",
    "source_repos": ["robonuggets/cinematic-site-components", "oso95/scroll-world"],
    "source_graph_nodes": ["technique-depth-planes", "technique-parallax"],
    "beads_required": true
  }
]
```

Write ≥10 laws total.

## Output

- `runs/current/design_laws.json`
- `resources/design-laws.md` (permanent, human-readable)

## Quality Check

- Assert ≥10 laws
- Assert every law has `source_repos` and `source_graph_nodes`
- Assert at least 1 law cites the graph

## Receipt

```json
{
  "agent": "synthesizer",
  "law_count": 12,
  "repos_consulted": ["list of 14"],
  "status": "PASS",
  "files": ["runs/current/design_laws.json", "resources/design-laws.md"],
  "timestamp": "ISO"
}
```

Write to: `runs/current/receipts/synthesizer.json`
