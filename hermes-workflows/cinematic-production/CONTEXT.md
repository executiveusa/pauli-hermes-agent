# Cinematic Production — Router

Follow `../_icm/methodology.md`. Open only the current stage contract plus the files it explicitly requires.

## Route

| Stage | Folder | Advances when |
|---|---|---|
| 00 | `stages/00_intake_route/` | format, outcome, audience, constraints, proof and budget are explicit |
| 01 | `stages/01_story_source_lock/` | story/argument and source burden are locked |
| 02 | `stages/02_visual_continuity/` | visual bible, continuity rules and shot plan pass |
| 03 | `stages/03_production/` | required media exists with generation/capture receipts |
| 04 | `stages/04_rough_cut/` | structure, comprehension and edit causality pass |
| 05 | `stages/05_fine_cut_qc/` | polished cut passes technical/factual QC |
| 06 | `stages/06_judge_deliver/` | all independent judges PASS and owner deliverables exist |

## Runtime State

Use `runs/current/` while active. Each stage writes its declared outputs plus receipts to `runs/current/receipts/`. A BLOCK writes `runs/current/BLOCK_REASON.md` and halts automatic advancement.

## Skill Dependencies

Primary: `skills/studio/cinematic-master-editor/`.
Load other skills only when the selected production method requires them. Reuse existing local-footage, YouTube intelligence, campaign, long-run harness, browser/capture, Remotion/HyperFrames or deployment capabilities rather than duplicating them here.
