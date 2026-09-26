# 08 Learn + Reuse
Record performance, cost, editorial outcomes, failed assumptions, useful provider patterns, and client-specific taste notes. Promote only verified reusable process improvements into shared workflow knowledge. Do not convert one client's creative decisions into universal templates. Output `lessons.md` (per-run record, in this run's directory), `cost-summary.json`, and any proposed skill/workflow updates for review.

## The learning loop (durable, cross-run)

"Shared workflow knowledge" above is a specific file:
`hermes-workflows/social-storytelling-ops/lessons.md`, at the workflow
root — durable and append-only across every run, distinct from this
stage's per-run `lessons.md`. Two writers feed it:

1. **Automatic, mid-run:** `scripts/approval.py`'s `reject()` appends every
   rejection reason to it the moment a human rejects a draft (see
   `cron/daily-content-digest.json` and the `/content-reject` command in
   `gateway/run.py`) — this does not wait for this stage to run.
2. **This stage, end-of-run:** promote verified reusable *editorial*
   patterns from the run's own findings (not raw rejection reasons, which
   are already captured automatically) — e.g. a pattern that held across
   multiple reels this run, not a one-off preference.

`story-miner` (`stages/01_story-map/CONTEXT.md`) and `taste-reviewer`
(`stages/05_review/CONTEXT.md`) both read this file before their run so a
rejection reason durably improves future drafts instead of only fixing the
one draft that was rejected. Do not overwrite it — append-only, same as
`approval.py`'s writer.