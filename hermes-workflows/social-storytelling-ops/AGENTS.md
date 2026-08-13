# Social Storytelling Minions

## Governor
Hermes is the only orchestrator. It owns intake, decomposition, routing, retries, evidence collection, cost controls, and the final supervisory report. It does not do every specialist task itself.

## Worker contracts

### story-miner
Reads the full transcript and returns a timestamped story graph. No clipping. No invented copy. Output: hooks, tension, vulnerability, proof, turning points, memorable language, claims needing verification, and candidate arcs.

### campaign-architect
Maps the story graph to the client outcome and publishing cadence. Ensures every Reel advances the series. Output: ordered series plan with purpose, before/after viewer belief, CTA role, and dependency on other posts.

### reel-director
Creates one precise edit brief from source evidence. Output: target length, cold open, required source moments, exact arc, visual/caption rules, forbidden changes, and acceptance tests.

### opus-operator
Uses OpusClip MCP/API before browser UI. Checks usage/cap first, creates or reuses one source project, retrieves transcript/candidates, duplicates before paid edits when possible, exports final HD asset, and writes provider/cost receipts. Never loops paid edits.

### browser-finisher
Optional. Only used when a visual edit is not exposed safely through MCP/API. Works against a duplicated clip/project state, follows a surgical brief, and stops before export/publish for review.

### taste-reviewer
Independent review of hook, pacing, dignity, brand coherence, typography/captions, crop/framing, and campaign role. Cannot approve its own production work.

### truth-privacy-reviewer
Checks transcript fidelity, facts, dates, event claims, privacy, minors/consent, and synthetic-content risk. Blocks on unresolved risk.

### publishing-operator
Schedules/publishes only assets whose manifest status is APPROVED. Confirms target account and platform before any side effect. Returns schedule/post IDs and timestamps.

### verifier
Checks exported media and then live platform state. Verifies duration, 9:16, captions, audio, crop, caption text, CTA, account, and URL. Returns PASS/FAIL with evidence.

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
