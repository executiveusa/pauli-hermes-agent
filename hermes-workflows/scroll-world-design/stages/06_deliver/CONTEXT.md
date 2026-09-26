# Stage 06 — Deliver

## Purpose

Package the final site, generate a delivery report, and present to the user.
This is the second and final human touchpoint.

## Input

From Stage 05:
- `runs/current/site/` — Judge-approved site
- `runs/current/judge_ux.json`
- `runs/current/judge_perf.json`
- `runs/current/judge_design.json`
- `runs/current/design_laws.json`
- `runs/current/beads/` — Full bead history

## Process

### Step 1: Package

Copy `runs/current/site/` to `runs/RUN_ID/site/`.
This freezes the deliverable — current/ may be reused for future runs.

### Step 2: Generate Delivery Report

`runs/RUN_ID/DELIVERY_REPORT.md`:

```markdown
# Scroll World Design — Delivery Report

## Site Summary
- Sections: [list]
- Archetype: [chosen archetype]
- Design laws applied: [count]

## Judge Scores
- UX: [score]/100 — PASS
- Performance: [score]/100 — PASS
- Design: [score]/100 — PASS

## Bead History (Atomic Rollback Available)
[list each bead with ID, what changed, timestamp]

## Files
- `site/index.html` — Entry point
- `site/styles/` — Tokens + styles
- `site/scripts/` — GSAP animations

## How to Deploy

### Vercel (recommended)
```bash
vercel site/
```

### Static hosting
Upload `site/` directory to any static host.

## How to Roll Back

Each design decision is a bead. To roll back:
```bash
# Roll back to bead ID
beads rollback [bead_id]
```

Full bead history: `runs/RUN_ID/beads/`
```

### Step 3: Present to User

Display:
- Delivery report (human-readable)
- Judge scores
- Deploy instructions
- Rollback options

Ask user: approve delivery or request revisions.

## Output

- `runs/RUN_ID/site/` — Frozen deliverable
- `runs/RUN_ID/DELIVERY_REPORT.md` — Report
- `runs/RUN_ID/receipts/stage_06.json`

## Gate

This stage has no automated gate — it is the delivery gate, approved by the human user.

**User approves** → workflow complete, `state.json` updated to `completed`.
**User requests revision** → return to Stage 04 with notes, increment bead.

## Collision Check

Delivery is always workflow-specific. No collision risk.
