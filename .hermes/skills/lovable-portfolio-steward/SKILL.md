---
name: lovable-portfolio-steward
description: Govern the Lovable project estate from inventory through evidence-based salvage, market research, wiring audit, sovereignty migration, ICM conversion, verification, portfolio publishing, and daily Notion reporting.
---

# Lovable Portfolio Steward

## Mission
Turn the Lovable estate into a governed portfolio of verified products, agents, skills, and proof assets. Notion is the human/agent control plane. GitHub is the canonical code source. Surviving projects must become portable and independently operable.

## Hard invariants
1. Never place project working copies on `C:`.
2. GitHub is source of truth. Do not modify a project until Lovable → GitHub linkage is proven by evidence; name similarity is insufficient.
3. README files are secondary evidence. Derive current behavior from source, config, schemas, migrations, routes, tests, runtime, and git history.
4. Never fabricate completion, wiring status, scores, repo mappings, or ICM facts.
5. Never mutate production data during reconnaissance.
6. Archive, merge, delete, or irreversible migration requires explicit human approval.
7. Proven-Better-New is a research/evidence gate and MUST NOT produce a pass/kill recommendation. Portfolio leverage scoring is a separate later step.
8. Full-stack wiring audit runs only after research on projects still worth investigating.
9. A surviving project may not leave business truth trapped in Lovable Cloud. Standard target is portable Supabase/Postgres with RLS and a tested export path.
10. `AGENTS.md` and ICM docs are updated only from verified reality.
11. `Portfolio Ready` requires a Git SHA, runtime/deployment proof, and verified host-side backup to `E:`. Hermes may record the gate but must not claim the local backup unless host evidence exists.

## Control plane
Use the Notion Lovable Project Registry as the canonical portfolio status ledger. Each row advances only when its evidence exists.

Stages:
`Inventory → Repo Mapping → Code Recon → PBN Research → Prioritized → Wiring Audit → Sovereignty → ICM → Verification → Portfolio Ready`

Terminal classifications after evidence and human review:
`SELL | USE | MERGE | PARK | ARCHIVE`

Product-shape classifications:
`Website | Agent | Skill | Agent+UI`

## Orchestration loop
For each eligible row:

### 1. Repo Mapper
- Read Lovable project identity and code.
- Find candidate GitHub repos.
- Prove linkage using file/commit/config evidence.
- Record `Verified`, `Likely`, `No Repo`, or `Ambiguous`; only `Verified` unlocks modification.

### 2. Code Recon
Inspect the codebase, not the pitch:
- entrypoints and routes
- user-visible promises/actions
- API/server/tool calls
- auth and authorization
- canonical state and duplicate state
- database/schema/migrations
- storage/files
- third-party APIs and SaaS dependencies
- environment variables and secret hygiene
- tests/build/deploy config
- git history when useful
- runtime/deployed surfaces when available

Write a compact evidence-backed statement of what the product actually does now.

### 3. Proven-Better-New research gate
Run the installed Proven-Better-New skill against the code-derived product and durable user instinct.
- Current web research is mandatory.
- Research direct/adjacent analogs, primary audience match, mechanics, pricing, retention, distribution, and graveyard cases.
- Preserve the PBN card as research evidence.
- Do not convert PBN into a verdict or score.

### 4. Portfolio leverage
After PBN is complete, calculate a separate Portfolio Leverage Score using evidence such as:
- strength/clarity of user problem
- demonstrated/proven market floor
- differentiation evidence
- code/runtime completeness
- cost to repair
- ability to become a reusable skill/agent
- fit with existing portfolio/niche clusters
- near-term commercial/proof value
- ownership/portability risk

Record the evidence behind the number. Then propose, but do not destructively execute, `SELL`, `USE`, `MERGE`, `PARK`, or `ARCHIVE`.

### 5. Full-stack wiring audit
For candidates that continue, trace each important promise end-to-end:
`UI → transport → backend/tool → canonical state → runtime result/evidence`

Flag:
- dead buttons and simulated success
- transport/backend mismatches
- duplicate state ownership
- auth/RLS gaps
- orphan tables/routes/integrations
- unhandled failures
- stale UI vs canonical state
- missing observability/rollback

### 6. Sovereignty migration
For survivors with Lovable Cloud or other lock-in:
- inventory owned data and dependencies
- design portable Supabase/Postgres schema
- apply least-privilege RLS
- preserve IDs/timestamps/relationships
- define/export data in a documented portable format
- test export and rollback before removing an old source
- never delete the old source until approved and verified

### 7. ICM conversion
Organize verified project truth into:
- **Intent** — outcome, audience, durable instinct
- **Contracts** — public promises, routes, APIs/tools, schemas, auth, external boundaries
- **Mechanisms** — actual runtime/code path
- **Evidence** — Git SHA, code refs, tests, runtime proof, PBN sources, wiring proof, RLS/export evidence
- **Open Debt** — unknowns, broken promises, duplicate ownership, security risks, migration blockers

Update `AGENTS.md` to point agents at this reality. Do not describe unverified aspirations as capabilities.

### 8. Agentization decision
Prefer **Agent** when the core product is intent understanding + context gathering + tool/API execution + outcome reporting.
Prefer **Skill** when the valuable part is a reusable method/workflow inside a broader agent.
Prefer **Agent+UI** when approvals, dashboards, previews, structured editing, visual outputs, multi-user state, or human review materially help.
Keep **Website** when SEO/discovery, public browsing, brand storytelling, commerce/onboarding, or rich visual interaction is itself core value.

### 9. Verification and portfolio proof
Before `Portfolio Ready`:
- build/tests pass or debt is explicit
- production/preview proof exists
- important workflows are exercised
- Git SHA is recorded
- data ownership/export proof exists when applicable
- `E:` backup is independently verified
- create proof material suitable for case studies/tutorials only from verified outcomes

## Specialized subagents
Dispatch narrow workers rather than one giant agent:
- Repo Mapper
- Code Recon
- Market Researcher (PBN)
- Portfolio Analyst
- Wiring Auditor
- Sovereignty Migrator
- ICM Architect
- Verification Agent
- Portfolio Publisher

Cheap/free models may handle repetitive inventory, extraction, and summarization when allowed. Escalate ambiguous architecture, security, destructive changes, and high-impact product reasoning. Provider choice (including DeepSeek or alternatives) must be resolved from current price/availability/privacy constraints at runtime.

## Daily heartbeat
At least once daily, append an entry to the Notion Lovable Mission Log containing:
- inventory total
- rows advanced
- PBN completions
- wiring completions
- sovereignty migrations
- ICM validations
- portfolio-ready assets
- human approvals needed
- blockers
- evidence added
- next highest-leverage focus

Never report progress that is not supported by the registry/evidence.
