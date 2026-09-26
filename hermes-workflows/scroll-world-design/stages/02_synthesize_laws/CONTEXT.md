# Stage 02 — Synthesize Design Laws

## Purpose

Combine the knowledge graph with lazy-loaded reference repos to produce a set of
immutable design laws — the axioms that all subsequent stages must never violate.

## Input

From Stage 01:
- `runs/current/graph.json` — Knowledge graph
- `runs/current/handoffs/01_to_02.json` — OpenSpec handoff

## Process

### Step 1: Load Reference Repos (lazy)

Load each repo only when its domain is needed. Do not clone — fetch README and key files.

| Repo | Domain | Laws It Contributes |
|------|--------|---------------------|
| `oso95/scroll-world` | Scroll techniques | Scroll binding, momentum, snap |
| `greensock/GSAP` | Animation | 60fps, GPU compositing, timeline sync |
| `gastownhall/beads` | Atomic rollback | Every design change = bead |
| `pbakaus/impeccable` | Design quality | Constraint → craft |
| `robonuggets/cinematic-site-components` | Cinematic UI | Depth planes, parallax layers |
| `darula-hpp/uigen` | UI generation | Component structure |
| `emilkowalski/skills` | Interaction design | Micro-interactions, spring physics |
| `ihlamury/design-skills` | Design patterns | Hierarchy, contrast, rhythm |
| `ytx-readings/design-ui-ux` | UI/UX readings | Anti-patterns to avoid |
| `safishamsi/graphify` | Data viz | Graph-to-insight pipeline |
| `Fission-AI/OpenSpec` | Handoff format | Agent communication spec |
| `willseltzer/claude-handoff` | Agent handoff | Chain-of-custody pattern |
| `vercel-labs/opensrc` | Deploy patterns | Vercel edge config |
| `atomicdotdev/atomic` | Atomic design | Token → component → template |

### Step 2: Reason Over Graph

For each cluster of nodes in the knowledge graph, derive patterns:
- Which techniques appear in most-viewed videos?
- What tools are always co-present?
- What sequence does Crawford always follow?
- What does he explicitly warn against?

### Step 3: Write Design Laws

Merge graph patterns + repo teachings into ≥10 immutable laws.

Format for each law:
```json
{
  "id": "law-NN",
  "title": "Short imperative title",
  "statement": "One declarative sentence.",
  "rationale": "One sentence: why this law exists.",
  "source_repos": ["repo1", "repo2"],
  "source_graph_nodes": ["node-id-1", "node-id-2"],
  "beads_required": true | false
}
```

Write to `runs/current/design_laws.json`

Also write human-readable version to `resources/design-laws.md` (permanent reference).

## Output

- `runs/current/design_laws.json` — Machine-readable laws
- `resources/design-laws.md` — Human-readable laws (permanent)
- `runs/current/receipts/stage_02.json`

## Gate

**PASS** if:
- `design_laws.json` has ≥10 laws
- All 14 repos were consulted (even if briefly)
- At least one law cites the knowledge graph

**BLOCK** if fewer than 10 laws — extend repo consultation.

## OpenSpec Handoff to Stage 03

`runs/current/handoffs/02_to_03.json`:
```json
{
  "spec_version": "1.0",
  "from_agent": "synthesizer",
  "to_agent": "builder",
  "stage": "02_synthesize_laws",
  "payload": {
    "type": "design_laws",
    "path": "runs/current/design_laws.json",
    "law_count": "<actual>",
    "repos_consulted": ["list"]
  },
  "gate_status": "PASS",
  "timestamp": "ISO"
}
```

## Next Stage

→ `stages/03_design_system/CONTEXT.md`
