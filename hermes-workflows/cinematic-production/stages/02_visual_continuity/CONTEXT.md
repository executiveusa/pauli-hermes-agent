# Stage 02 — Visual Bible, Continuity and Shot Plan

### Input
- locked story/script and source audit
- approved brand/character/world references

### Process
1. Define visual language using `TASTE.md`.
2. Lock recurring identity, wardrobe, props, locations, light logic, screen direction, voice and title rules when applicable.
3. Break the story into shot beats. Every shot declares narrative job, blocking, framing, motivated movement, sound anchor, continuity anchors, duration and production method.
4. Separate immutable identity/style constraints from mutable per-shot action.

### Output
- `runs/current/visual-bible/`
- `runs/current/CONTINUITY.md`
- `runs/current/shot-list.csv`
- `runs/current/shot-plan.json`
- `runs/current/PRODUCTION_PLAN.md`

### Gate
PASS only when no required shot lacks a job, continuity anchor, production route or budget classification, and continuity-heavy work has approved references before bulk generation.

### Receipt
Write `runs/current/receipts/02_visual_continuity.json`.
