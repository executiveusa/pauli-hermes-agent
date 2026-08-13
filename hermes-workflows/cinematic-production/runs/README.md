# Runtime Artifacts

Do not use this folder as a permanent dumping ground. Each execution writes to `runs/current/` while active, then may be archived to a timestamped/name-safe run directory after delivery.

Expected runtime shape:

```text
runs/current/
├── production-spec.yaml
├── ROUTING.md
├── SOURCE_AUDIT.md
├── STORY_BRIEF.md or SCRIPT.md
├── visual-bible/
├── CONTINUITY.md
├── shot-list.csv
├── shot-plan.json
├── PRODUCTION_PLAN.md
├── assets/
├── edit-decision-list.json
├── ROUGH_CUT_REVIEW.md
├── FINE_CUT_REVIEW.md
├── QC_REPORT.md
├── JUDGE_*.md
├── receipts/
├── delivery/
├── HANDOFF.md
└── LESSONS.md
```

Large media, secrets, client-private source files and generated caches should follow repository storage/ignore policy rather than being committed by default.
