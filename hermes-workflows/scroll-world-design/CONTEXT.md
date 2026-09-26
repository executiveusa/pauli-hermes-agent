# Scroll World Design — Stage Router

## How to Use This File

If you are resuming this workflow, check `runs/current/state.json` to find the last
completed stage, then open the corresponding stage CONTEXT.md.

## Stage Map

```
00_intake          → User provides brief
  ↓ [GATE: brief validated]
01_scrape_and_graph → Scrape @bycrawford, build knowledge graph
  ↓ [GATE: ≥15 videos scraped, graph written]
02_synthesize_laws  → Graph + repo laws → design system laws
  ↓ [GATE: ≥10 laws written, all repos loaded]
03_design_system    → Laws → component spec + style tokens
  ↓ [GATE: spec passes OpenSpec schema]
04_generate_site    → Spec → working code (HTML/GSAP/CSS)
  ↓ [GATE: code runs, no console errors]
05_judge_panel      → 3 judges score output
  ↓ [GATE: all 3 judges PASS]
06_deliver          → Package + present to user
```

## Current Stage

Check `runs/current/state.json`:
```json
{
  "workflow": "scroll-world-design",
  "current_stage": "NN",
  "last_gate": "PASS | BLOCK | null",
  "started_at": "ISO timestamp",
  "run_id": "run_YYYYMMDD_HHMMSS"
}
```

If `state.json` does not exist → go to Stage 00.

## Stage Files

| Stage | File |
|-------|------|
| 00 | `stages/00_intake/CONTEXT.md` |
| 01 | `stages/01_scrape_and_graph/CONTEXT.md` |
| 02 | `stages/02_synthesize_laws/CONTEXT.md` |
| 03 | `stages/03_design_system/CONTEXT.md` |
| 04 | `stages/04_generate_site/CONTEXT.md` |
| 05 | `stages/05_judge_panel/CONTEXT.md` |
| 06 | `stages/06_deliver/CONTEXT.md` |

## On BLOCK

1. Read `runs/current/BLOCK_REASON.md`
2. Surface reason to user with remediation options
3. Do not advance to next stage until gate PASS
4. Record block in `runs/current/receipts/`
