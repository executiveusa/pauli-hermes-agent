---
name: no-code-repair-loop
description: Iterate deployment visibility failures using only no-code-safe actions and stop when source-code changes are required.
---

# No-Code Repair Loop

## Use when

- Deployment exists but site is not visible.
- 404, build error, stale deployment, wrong project link, or protection wall is detected.

## Safe action order

1. Verify latest deployment URL.
2. Inspect logs.
3. Confirm Vercel scope/project match.
4. Relink if mapping is wrong and exact project is known.
5. Redeploy current main.
6. Redeploy once with `--force`.
7. Verify URL again.
8. Stop and report if code or env mutation is required.

## Attempt limit

Maximum two production deploy attempts per run.
