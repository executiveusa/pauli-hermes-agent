# Social Storytelling Operations — ICM Router

## Outcome
Turn long-form client footage and transcripts into a coherent, campaign-aware short-form storytelling series that is edited, reviewed, approved, scheduled, published, verified, and learned from with minimal owner intervention.

## Operating mode
BROWNFIELD by default. Preserve client assets, brand rules, source footage, transcripts, approved copy, posting cadence, and existing app/workflow contracts. Never rewrite or delete source media.

## Human role
The owner supervises outcomes, taste, style, public claims, budget thresholds, and final approval gates. The owner is not the production router.

## Agent role
Hermes is the Governor/Producer. Hermes decomposes the campaign into durable Kanban work, assigns specialist workers, verifies evidence, tracks provider cost, and reports exceptions instead of asking the owner to manually coordinate normal production.

## Prime workflow
INGEST → STORY MAP → SERIES PLAN → COLD OPEN → EDIT BRIEF → OPUS EXECUTION → PRECISION QA → APPROVAL → SCHEDULE → LIVE VERIFY → LEARN

## Stage routing
1. `stages/00_intake/CONTEXT.md` — identify client, objective, audience, protected assets, campaign cadence, source files, deadlines, and proof.
2. `stages/01_story-map/CONTEXT.md` — read the full transcript before clipping; extract narrative arcs, hooks, tension, proof, turning points, human stakes, calls to action, and factual claims.
3. `stages/02_series-plan/CONTEXT.md` — sequence reels across the campaign so each post advances the story rather than competing with it.
4. `stages/03_edit-brief/CONTEXT.md` — create one precise, source-grounded brief per reel.
5. `stages/04_opus/CONTEXT.md` — execute through OpusClip using MCP/API first; browser control only when visual editor work is not exposed safely.
6. `stages/05_review/CONTEXT.md` — independent story, truth, visual, caption, brand, mobile, privacy, and cost review.
7. `stages/06_publish/CONTEXT.md` — require approval, then schedule/publish through the connected social surface.
8. `stages/07_verify/CONTEXT.md` — verify the live post, record URL/time/evidence, and confirm media/caption/crop.
9. `stages/08_learn/CONTEXT.md` — record performance, cost, editorial lessons, and reusable patterns without turning the next client into a template clone.

## Minion topology
Hermes should prefer durable Kanban workers over one giant prompt.

- Story Miner — reads transcript and returns story graph with timestamps/evidence.
- Campaign Architect — sequences stories against business/social outcome and calendar.
- Reel Director — turns one story into a cut brief and acceptance contract.
- Opus Operator — performs API/MCP execution, duplication, edits, export, and provider receipts.
- Browser Finisher — optional specialist for visual-only Opus editor adjustments; never first choice.
- Brand/Taste Reviewer — checks tone, typography, pacing, dignity, and campaign fit.
- Truth/Privacy Reviewer — checks source fidelity, claims, consent, minors/privacy, and event facts.
- Publishing Operator — schedules only approved assets and returns platform receipts.
- Verifier — independently verifies the final/live artifact; cannot approve its own production work.
- Cost Accountant — reconciles Opus/API/edit usage against the run ledger and flags anomalies.

## Stripe-Minions-inspired operating rules
- One owner instruction should create a complete, inspectable work graph.
- Workers operate in isolated tasks with explicit acceptance criteria.
- Context is gathered before execution, not discovered after a 30-minute failure.
- Every task has a completion contract and bounded retry budget.
- Local/read-only checks precede paid or public side effects.
- CI/evidence/judges are feedback loops, not decoration.
- Production/public actions remain gated when risk is material.
- Fail early on missing permissions, credentials, source media, consent, or account connections.

## Cost governor
Before any paid Opus operation, create/update a run receipt containing:
- client/project
- source duration
- estimated credits
- current API cap/usage when available
- operation type
- approval requirement
- credits before/after when available
- actual credit delta
- outputs created

Rules:
- Read/list/transcript/inspect operations may proceed when they do not incur provider charges.
- New processing jobs require an estimate first.
- Duplicate before substantive server-side edits when possible.
- More than 3 paid edit/re-render operations on one clip requires escalation.
- Never loop paid edits or publishing actions.
- Never publish without an explicit approval state on the asset.

## Tool routing order
1. Native deterministic tool / local file operation.
2. Opus MCP/API.
3. Structured browser automation.
4. Pixel/mouse browser control as last resort.

## Evidence contract
A reel is not DONE because a render exists.
Required final evidence:
- source transcript/timestamps
- final story brief
- provider project + clip identifiers
- final duration/aspect
- caption verification
- exported MP4 location
- review verdicts
- approval record
- scheduled/published receipt if applicable
- live URL and live verification after publishing
- cost receipt

## Stop conditions
Stop and escalate instead of improvising when:
- source ownership/permission is unclear
- minors/consent risk is unresolved
- event facts conflict
- a paid action exceeds the configured budget threshold
- social account is not the intended client account
- the model would need to invent dialogue or facts
- the owner has not approved a public-facing final asset
- a worker cannot access a required tool or credential

## Cold-walk test
A memoryless worker must be able to answer: client, objective, current stage, source of truth, next task, allowed tools, expected output, evidence required, cost state, and approval owner from the run files alone.
