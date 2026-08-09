# Stage 04 — Assembly and Rough Cut

### Input
- usable media assets, final/working narration, shot plan, continuity ledger

### Process
1. Assemble an editable timeline using CapCutAPI/MCP, CapCut Mate/VectCut where selected, or the approved programmatic editor.
2. Keep picture, spoken audio, music, ambience/SFX/foley, captions and graphics logically separable.
3. Review structure before polish: comprehension, emotional/information progression, shot purpose, dead time, duplicate beats and continuity.
4. Record edit decisions and defects.

### Output
- editable draft/project
- `runs/current/edit-decision-list.json`
- `runs/current/ROUGH_CUT_REVIEW.md`

### Gate
PASS only when the rough cut is understandable on its own, every retained shot earns its duration, critical continuity/factual problems are absent, and the editable draft is recoverable.

### Receipt
Write `runs/current/receipts/04_rough_cut.json`, including CapCut/API or render actions.
