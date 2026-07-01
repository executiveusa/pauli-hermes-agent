# Architecture

## Runtime model

```text
GitHub webhook / A2A task / IDE command
        ↓
Stage 01 normalized event
        ↓
Stage 02 repo inventory
        ↓
Stage 03 Vercel project map
        ↓
Stage 04 deployment triage
        ↓
Stage 05 production redeploy
        ↓
Stage 06 browser verification
        ↓
Stage 07 scoped sub-agent iteration
        ↓
Stage 08 report
```

## Why scripts exist

The agent should not spend model tokens doing deterministic work. Scripts collect repo metadata, call APIs, run CLI commands, and emit JSON. The agent reasons over the JSON and stage gates.

## Hosted future state

The local server in `scripts/server.mjs` can later be ported into a Vercel app route, with run artifacts persisted to a database/blob store instead of local disk. The folder contracts should remain the source of truth.
