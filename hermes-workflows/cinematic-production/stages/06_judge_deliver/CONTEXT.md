# Stage 06 — Independent Judge and Delivery

### Input
- candidate master/variants
- production spec, source/continuity manifests, rough/fine reviews and QC report

### Process
1. Run the three independent judges in `AGENTS.md` without giving them builder authority.
2. Each judge returns PASS or BLOCK plus defects and evidence.
3. If any judge BLOCKS, route the defect to the responsible earlier stage and halt delivery.
4. If all PASS, package final master(s), editable project/draft, captions/transcript when required, source/provenance manifest, QC, prompts/generation logs, rights notes and rollback/handoff instructions.

### Output
- `runs/current/JUDGE_STORY_TASTE.md`
- `runs/current/JUDGE_CONTINUITY_INTEGRITY.md`
- `runs/current/JUDGE_TECHNICAL_SOVEREIGNTY.md`
- `runs/current/delivery/`
- `runs/current/HANDOFF.md`
- `runs/current/LESSONS.md`

### Gate
PASS only when all judges PASS, requested deliverables exist, playback/QC evidence is present, editable/owner-controlled assets are included when required, and no secret/private path is exposed in the handoff.

### Receipt
Write `runs/current/receipts/06_judge_deliver.json` and a final manifest/checksum list.
