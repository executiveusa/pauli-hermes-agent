# ICM Methodology — YouTube Channel Scraper

This workflow uses the Interpretable Context Methodology (ICM) folder-as-agent pattern.

## Layers

| Layer | File/folder | Purpose |
|------:|---|---|
| 0 | `AGENTS.md`, `CLAUDE.md` | Global identity and behavior |
| 1 | `CONTEXT.md` | Router that tells agent which stage to open |
| 2 | `stages/*/CONTEXT.md` | One stage contract: inputs, process, outputs, gates |
| 3 | `guardrails/`, `subagents/`, `resources/` | Stable references and capabilities |
| 4 | `runs/`, `stages/*/output/` | Working artifacts for current run |

## Why This Structure

- **Handoff-friendly** — Easy to pass between IDEs and agents
- **Small context** — Each stage is ~300 lines, fits in local context
- **Auditable** — Every intermediate artifact is saved
- **Clear stop points** — Each stage has entry/exit gates
- **Portable** — Zip entire `skills/youtube-channel-scraper/` directory

## Stage Execution Pattern

Each stage follows:

1. **Input** — Read from `stages/[N]/input/`
2. **Validate** — Check gates before proceeding
3. **Process** — Execute stage-specific logic
4. **Output** — Write to `stages/[N]/output/`
5. **Report** — Summarize completion, list next steps

## No Hidden Orchestration

The folder tree IS the orchestration. If you need to inspect the workflow:
- Look at `CONTEXT.md` for the flow diagram
- Check `stages/[N]/CONTEXT.md` for that stage's details
- Read `stages/[N]/output/` to see what happened

No YAML, no abstract state machine. Just folders and markdown.
