# Stage 05 — Fine Cut, Sound and QC

### Input
- rough cut that passed Stage 04
- target delivery specifications

### Process
1. Refine pacing, entrance/exit frames, transitions, graphics, reframing, grade and text.
2. Refine dialogue/VO, ambience, foley/SFX, music dynamics, silence and caption timing.
3. Run technical QC: resolution, fps, ratio, codec/container, duration, missing/frozen frames, dialogue intelligibility, caption spelling/safe areas, watermark/placeholders and target crops.
4. For factual/brand work, verify final-cut claims and labels against the source manifest.
5. Playback representative sections and the full master when feasible.

### Output
- candidate master/variants
- `runs/current/FINE_CUT_REVIEW.md`
- `runs/current/QC_REPORT.md`
- checksums/probe evidence

### Gate
PASS only when the candidate master has no critical playback, factual, continuity, caption, rights-known, or target-format defect.

### Receipt
Write `runs/current/receipts/05_fine_cut_qc.json` with render/export/probe actions.
