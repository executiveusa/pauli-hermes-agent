# Stage 03 — Media Production

### Input
- approved shot plan, production plan and continuity references

### Process
1. Use the cheapest reliable method per shot: existing media, capture, code render, still+motion, avatar, generated image or generated video.
2. For paid generation, verify the approved budget gate first.
3. Save prompts, model/version, references, source paths/URLs, timestamps and failures.
4. Diagnose failures before rerolling; change one meaningful variable at a time.

### Output
- `runs/current/assets/`
- `runs/current/generation-log.jsonl`
- updated source/material manifest

### Gate
PASS only when every required shot has an approved usable asset or an explicit BLOCK reason, continuity-critical assets pass contact-sheet/sequence inspection, and all paid/external calls have receipts.

### Receipt
Write `runs/current/receipts/03_production.json` with generation/capture/backend actions and costs where available.
