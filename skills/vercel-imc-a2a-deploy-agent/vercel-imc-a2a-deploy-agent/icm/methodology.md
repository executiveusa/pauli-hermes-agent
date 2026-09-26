# IMC / ICM Methodology for This Agent

This workspace uses the user’s IMC framing and the Jake Van Clief / Interpretable Context Methodology folder-as-agent pattern.

## Layers

| Layer | File/folder | Purpose |
|---:|---|---|
| 0 | `AGENTS.md`, `CLAUDE.md` | Global identity and non-negotiable behavior. |
| 1 | `CONTEXT.md` | Router that tells the agent which stage to open. |
| 2 | `stages/*/CONTEXT.md` | One stage contract: inputs, process, outputs, gates. |
| 3 | `guardrails/`, `skills/`, `resources/`, `subagents/` | Stable references and capabilities. |
| 4 | `runs/`, `stages/*/output/` | Working artifacts for the current run. |

## Rule

Do not replace this with a hidden orchestration framework unless a later mission explicitly requires high concurrency. The folder tree is the orchestration framework.

## Why it matters

- Easier handoff between IDEs and agents.
- Smaller context windows.
- Auditable intermediate artifacts.
- Clear stop points.
- Portable as a zip file.
