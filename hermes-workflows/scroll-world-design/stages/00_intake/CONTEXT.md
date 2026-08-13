# Stage 00 — Intake Brief

## Purpose

Capture and validate the user's brief before any automated work begins.
This is the first and only human-required stage before delivery.

## Input

User-provided brief (natural language). May include:
- Target audience
- Brand / aesthetic direction
- Key sections / scroll sequences
- Reference sites or videos
- Technical constraints

## Process

1. Parse brief into structured fields
2. Fill in any missing required fields with sensible defaults (log them)
3. Write structured brief to `runs/current/brief.json`
4. Write `runs/current/state.json` with `current_stage: "00"`

## Output

`runs/current/brief.json`:
```json
{
  "run_id": "run_YYYYMMDD_HHMMSS",
  "audience": "string",
  "aesthetic": "string (e.g. dark-cinematic, light-minimal, brutalist)",
  "scroll_archetype": "string | null (auto-selected in stage 02 if null)",
  "sections": ["hero", "feature-1", "feature-2", "cta"],
  "references": ["url | @channel | description"],
  "constraints": {
    "framework": "vanilla | react | next",
    "deploy_target": "vercel | cloudflare | static"
  },
  "raw_brief": "verbatim user input"
}
```

## Gate

**PASS** if `brief.json` contains `audience`, `aesthetic`, and `raw_brief`.
**BLOCK** if user provided no usable brief text.

On BLOCK: surface a 3-question intake form to the user. Do not proceed until answered.

## Receipt

```json
{
  "stage": "00_intake",
  "action": "parse_brief",
  "result": "PASS | BLOCK",
  "files_written": ["runs/current/brief.json", "runs/current/state.json"],
  "timestamp": "ISO"
}
```

## Next Stage

→ `stages/01_scrape_and_graph/CONTEXT.md`

## Collision Check

Does not collide with any other workflow. Brief is workflow-specific.
