# ICM — Interpretable Context Methodology

## Origin

ICM (Interpretable Context Methodology) is a folder-as-agent pattern developed to make
complex multi-step agentic work legible, resumable, and auditable.

Core principle: **every decision produces a contract; every contract gates the next step.**

## Layer Map

| Layer | What Lives Here | Role |
|-------|----------------|------|
| 0 — Identity | `CLAUDE.md`, `AGENTS.md` | Who I am and what I can do |
| 1 — Router | `CONTEXT.md` | Where am I; which stage to open |
| 2 — Stage Contracts | `stages/NN_name/CONTEXT.md` | Inputs → process → outputs → gate |
| 3 — Stable References | `resources/` | Design laws, archetypes, quality bars |
| 4 — Run Artifacts | `runs/` | Timestamped outputs from each execution |

## Contracts

Every stage CONTEXT.md must declare:

```
## Stage NN — Name
### Input
  - What arrives from stage NN-1 (exact file or field)
### Process
  - What this stage does (steps, tools, subagents called)
### Output
  - Exact files written, schema
### Gate
  - Validation check that must pass before stage N+1 starts
### Receipt
  - Side-effect log (files written, external calls made)
```

## Gates

A gate is a binary: **PASS** or **BLOCK**.

- **PASS** → next stage starts automatically
- **BLOCK** → workflow halts, writes a `BLOCK_REASON.md` to `runs/current/`, surfaces to user

No partial passes. No silent degradation.

## Receipts

Every side effect (scrape, write, API call) produces a receipt:

```json
{
  "stage": "01_scrape_and_graph",
  "action": "scrape_channel",
  "target": "https://www.youtube.com/@bycrawford/videos",
  "result": "PASS",
  "files_written": ["runs/current/stage_01_raw.jsonl"],
  "timestamp": "2026-08-07T00:00:00Z"
}
```

Stored in `runs/current/receipts/`.

## Workflow vs Skill

- **Skill** answers: *"How do I do X?"* — one capability, one input → one output
- **Workflow** answers: *"How do I achieve Y from start to finish, with no slop?"*

Workflows chain ≥2 skills. They follow ICM from first to last stage.

## Anti-Slop Contract

Every workflow must include in its final stage a multi-judge panel.
Judges are independent subagents that each score the output against a known quality bar.
Output only ships when all judges PASS.

No exceptions.
