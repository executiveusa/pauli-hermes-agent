# Social Storytelling Minions

## Governor
Hermes is the only orchestrator. It owns intake, decomposition, routing, retries, evidence collection, cost controls, and the final supervisory report. It does not do every specialist task itself.

## Worker contracts

### story-miner
Reads the full transcript and returns a timestamped story graph. No clipping. No invented copy. Output: hooks, tension, vulnerability, proof, turning points, memorable language, claims needing verification, and candidate arcs. Source material can arrive as a direct transcript, a tagged gateway voice memo/text drop, an `agent-reach`/`youtube-knowledge-extractor` knowledge object, or an `interview-panel` transcript (see `stages/00_intake/CONTEXT.md`) — same graph-extraction contract regardless of origin. When raw material references existing footage, query `skills/local-footage-studio` before assuming new footage is required. Reads `lessons.md` (workflow root, durable cross-run) before mining — see `stages/08_learn/CONTEXT.md` "The learning loop".

### campaign-architect
Maps the story graph to the client outcome and publishing cadence. Ensures every Reel advances the series. Slots material into `skills/social-drop-factory`'s Monday/Wednesday/Friday belief → story → action cadence per `stages/02_series-plan/CONTEXT.md` rather than inventing per-run cadence logic. Output: ordered series plan with purpose, before/after viewer belief, CTA role, and dependency on other posts.

### reel-director
Creates one precise edit brief from source evidence. Output: target length, cold open, required source moments, exact arc, visual/caption rules, forbidden changes, and acceptance tests.

### opus-operator
Uses OpusClip MCP/API before browser UI. Checks usage/cap first, creates or reuses one source project, retrieves transcript/candidates, duplicates before paid edits when possible, exports final HD asset, and writes provider/cost receipts. Never loops paid edits.

### browser-finisher
Optional. Only used when a visual edit is not exposed safely through MCP/API. Works against a duplicated clip/project state, follows a surgical brief, and stops before export/publish for review.

### taste-reviewer
Independent review of hook, pacing, dignity, brand coherence, typography/captions, crop/framing, and campaign role. Cannot approve its own production work. Reads `lessons.md` (workflow root, durable cross-run) before reviewing — see `stages/08_learn/CONTEXT.md` "The learning loop" — so a past rejection reason is checked for on every future reel, not just the one it was raised on.

### truth-privacy-reviewer
Checks transcript fidelity, facts, dates, event claims, privacy, minors/consent, and synthetic-content risk. Blocks on unresolved risk.

### publishing-operator
Backed by Postiz (see `tools.yaml` in this workflow directory for the tiered permission contract). May create Postiz drafts (`create_draft`, `upload_approved_media`, `build_content_calendar_draft`) across all connected channels automatically — drafting is not a public side effect. `schedule_post` and `publish_now` require the reel's manifest status to be `APPROVED` by a human before the call is made; there is no automatic path from draft to scheduled/published. Confirms target account and platform before any side effect. Returns schedule/post IDs and timestamps. This is the same rule stated in `## Retry policy` below as "Public action without approval: forbidden" — the two must not drift: if either document changes the approval condition, update both in the same change.

### verifier
Checks exported media and then live platform state. Live platform state means querying Postiz's `analytics_post` and `posts_list` (see `tools.yaml`, `automatic_read`) after publish — not just re-reading the local export file or manifest, which only proves the pipeline believes it published, not that the platform confirms it. Verifies duration, 9:16, captions, audio, crop, caption text, CTA, account, and URL. Returns PASS/FAIL with evidence.

### cost-accountant
Maintains provider ledger. Reconciles estimated vs actual Opus credit deltas and flags unexpected paid operations.

## Kanban graph template
ROOT: campaign outcome
├─ transcript/story-map [story-miner]
├─ series-plan [campaign-architect] depends on story-map
├─ reel-N-direct [reel-director] depends on series-plan
├─ reel-N-opus [opus-operator] depends on direct
├─ reel-N-taste-review [taste-reviewer] depends on opus
├─ reel-N-truth-review [truth-privacy-reviewer] depends on opus
├─ reel-N-approval [Governor/human gate] depends on both reviews
├─ reel-N-publish [publishing-operator] depends on approval
├─ reel-N-live-verify [verifier] depends on publish
└─ cost-reconcile [cost-accountant] depends on provider work

## Retry policy
- Mechanical/transient failures: max 2 automatic retries.
- Editorial review failure: create a bounded revision task; do not restart the whole pipeline.
- Permission/credential/account mismatch: BLOCK immediately with `needs_input`.
- Repeated provider charge or >3 paid edits on one clip: BLOCK for cost review.
- Public action without approval: forbidden.

## Completion protocol
Every worker must heartbeat during long work and end with either a durable completion summary + evidence metadata or a blocked reason. Narrative claims like “done” without receipts are protocol violations.
