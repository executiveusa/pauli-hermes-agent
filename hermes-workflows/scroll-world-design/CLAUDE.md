# Scroll World Design — Entry Point

## Outcome

High-quality cinematic scroll websites, every time. No slop ships.

## Invoke

```
/scroll-world-design [brief]
```

Or ask Hermes naturally:
```
"build me a cinematic landing page for..."
"create a 3D scroll website that..."
"design a parallax site like..."
```

## What Gets Built

A production-ready scroll/cinematic website with:
- GSAP-powered animations (60fps GPU)
- Crawford-style depth planes and camera drift
- Atomic rollback via beads (every design change is a bead)
- Design system derived from 14+ reference repos

## Workflow Stages

| # | Stage | Duration | Human? |
|---|-------|----------|--------|
| 00 | Intake brief | ~1m | ✓ (required) |
| 01 | Scrape & graph | ~5m | ✗ |
| 02 | Synthesize design laws | ~3m | ✗ |
| 03 | Build design system | ~5m | ✗ |
| 04 | Generate site | ~10m | ✗ |
| 05 | Judge panel (3 judges) | ~3m | ✗ |
| 06 | Deliver | ~1m | ✓ (approval) |

Total: ~28 minutes, 2 human touchpoints.

## Key Files

- `CONTEXT.md` — Stage router (start here if resuming)
- `AGENTS.md` — Who does what
- `resources/design-repos.md` — Lazy-loaded repos and what they teach
- `resources/design-laws.md` — Synthesized immutable laws
- `resources/archetypes.md` — Crawford scroll archetypes
- `resources/quality-bar.md` — Judge scoring rubric

## Reference Repos (lazy-loaded)

All repos in `WORKFLOW_REGISTRY.json` are loaded only when needed by a stage.
They are never cloned upfront. Each repo is turned into a design law when loaded.

## Beads Integration

Every design decision is a bead:
- One-click rollback to any prior state
- Receipts stored in `runs/current/beads/`
- Uses `gastownhall/beads` pattern

## OpenSpec Handoffs

Subagents communicate via OpenSpec format (Fission-AI/OpenSpec):
- Machine-readable, schema-validated
- Stored in `runs/current/handoffs/`
- Agent cannot proceed without valid OpenSpec from prior agent
