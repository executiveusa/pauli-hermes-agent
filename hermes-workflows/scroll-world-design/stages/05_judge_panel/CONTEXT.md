# Stage 05 — Judge Panel

## Purpose

Three independent specialist judges review the generated site against a known quality bar.
All three must PASS before the site is delivered. No exceptions.

## Input

From Stage 04:
- `runs/current/site/` — Complete generated site
- `runs/current/design_laws.json` — Laws to check against
- `runs/current/component_specs.json` — Intended behavior
- `resources/quality-bar.md` — Scoring rubric

## Judges

Each judge is an independent subagent. They do not share results with each other.
Orchestrator collects all three verdicts before proceeding.

### Judge 1: UX (`subagents/judge-ux/`)

Evaluates:
- Scroll behavior feels natural (not janky, not too slow)
- Content hierarchy is clear at each scroll position
- CTA is visible and accessible
- Section transitions don't cause disorientation
- Mobile usability (viewport meta set, no horizontal scroll)

Scoring rubric: `resources/quality-bar.md#ux`

### Judge 2: Performance (`subagents/judge-perf/`)

Evaluates:
- GSAP animations use `transform` and `opacity` only (no layout-thrashing properties)
- No `will-change` on more than 3 elements simultaneously
- Images have explicit width/height (no CLS)
- GSAP loaded with `defer` or at end of `<body>`
- ScrollTrigger uses `scrub` not `snap` on long-scroll sections (prevents jank)
- No synchronous XHR or blocking scripts

Scoring rubric: `resources/quality-bar.md#performance`

### Judge 3: Design (`subagents/judge-design/`)

Evaluates:
- Color contrast ≥4.5:1 for body text
- Typography scale has ≥3 distinct sizes
- Spacing is consistent with token scale
- Depth plane effect is visible and intentional
- No Lorem Ipsum in final output
- No placeholder `[IMAGE]` text in final output
- Overall "slop" test: would this embarrass a senior designer?

Scoring rubric: `resources/quality-bar.md#design`

## Process

1. Orchestrator passes `site/` path to all three judges simultaneously
2. Each judge reads the site files and scores against their rubric
3. Each judge writes their verdict:

```json
{
  "judge": "ux | perf | design",
  "verdict": "PASS | BLOCK",
  "score": 0-100,
  "findings": [
    { "severity": "blocking | warning", "description": "..." }
  ],
  "timestamp": "ISO"
}
```

Output files:
- `runs/current/judge_ux.json`
- `runs/current/judge_perf.json`
- `runs/current/judge_design.json`

4. Orchestrator reads all three verdicts.

## Gate

**PASS** if all three judges return `verdict: "PASS"`.

**BLOCK** if any judge returns `verdict: "BLOCK"`:
- Collect all blocking findings across all judges
- Route back to Stage 04 builder with specific remediation instructions
- Builder addresses findings and increments bead ID
- Re-run judge panel (max 2 revision rounds before escalating to user)

After 2 failed revision rounds:
- Surface all blocking findings to user with context
- User decides: accept as-is, provide new brief, or abandon

## Output

- `runs/current/judge_ux.json`
- `runs/current/judge_perf.json`
- `runs/current/judge_design.json`
- `runs/current/receipts/stage_05.json`

## Next Stage

→ `stages/06_deliver/CONTEXT.md` (only if all judges PASS)
