# ICM (Interpretable Context Methodology) for YouTube Scraper

This workspace uses the Interpretable Context Methodology — a folder-as-agent pattern where the folder structure IS the orchestration framework.

## Layers

| Layer | Files | Purpose |
|-------|-------|---------|
| **0** | `AGENTS.md`, `CLAUDE.md` | Agent identity, non-negotiable behavior |
| **1** | `CONTEXT.md` | Router — maps user intent to stages |
| **2** | `stages/*/CONTEXT.md` | Stage contract: inputs, process, outputs, exit gates |
| **3** | `guardrails/`, `resources/` | Stable references (rate limits, configs, safety rules) |
| **4** | `runs/<run-id>/` | Working artifacts for current run |

## Why Folder-as-Agent?

- **Portable** — Zip and ship to another IDE or agent
- **Auditable** — Every intermediate artifact lives in `runs/<run-id>/`
- **Clear stop points** — Each stage has defined inputs/outputs
- **Smaller context** — Load only the stage you're in, not the whole workspace
- **Handoff-ready** — Another agent can pick up a run by reading the latest stage output

## Key Rule

**Do not replace this with a hidden orchestration framework** unless later missions require high concurrency. The folder tree IS the framework.

## Navigation Example

```
User: "Scrape my channel and create a workflow"
  ↓
Open CONTEXT.md → "Intent = scrape + generate"
  ↓
Enter stages/01_scrape_playlist/CONTEXT.md → Execute scrape
  ↓
Enter stages/02_process_metadata/CONTEXT.md → Normalize data
  ↓
Enter stages/03_analyze_patterns/CONTEXT.md → Find patterns
  ↓
Enter stages/04_generate_workflow/CONTEXT.md → Call skill, output spec
  ↓
Create run artifact: runs/20260806-123456-<channel>/workflow_spec.json
```

All outputs auditable, all paths documented.
