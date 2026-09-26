# Hermes Upstream Selective Upgrade Audit — 2026-08-08

## Decision
DO NOT merge `NousResearch/hermes-agent/main` wholesale into `executiveusa/pauli-hermes-agent`.

The fork contains owner-specific skills, ICM workflows, governance, integrations, and operational behavior that must remain authoritative. Upstream capabilities are candidates to inspect and selectively port behind tests.

## Verified baseline
- Fork default branch HEAD at audit start: `1876b261f9d779385f2794e68df0644459965826`.
- Fork `pyproject.toml` declares Hermes `0.16.0`.
- Upstream `pyproject.toml` currently declares Hermes `0.20.0`.
- The version string is not a reliable feature-diff proxy: the fork already contains later capabilities such as durable Kanban documentation/tooling and the webhook adapter, indicating selective backports/cherry-picks.
- Upstream main is high velocity. A current upstream commit reports a standards sweep over 191 in-repo skills: 77 bundled and 114 optional.

## Existing fork capabilities confirmed and protected
1. `hermes-workflows/` ICM workflow system.
2. Owner-added skills and `SKILL_REGISTRY.json` governance.
3. Durable multi-agent Kanban documentation/tooling with heartbeats, retries, reclaim/zombie handling, worker evidence and orchestrator fan-out.
4. Webhook adapter with HMAC validation, replay protection/idempotency/rate limits, route-specific prompts/skills, and delivery targets.
5. MCP integration surface.
6. Owner-specific Hostinger/agent-payments/Freenet/YouTube/security/design additions visible in recent fork commits.
7. Custom SOUL/persona behavior and project memory must never be replaced by upstream defaults without explicit review.

## Upstream release families to evaluate selectively

### v0.17 — Reach
Candidate value:
- broader gateway/platform reach
- team/deployment improvements
- deeper integrations
- operational hardening

Policy: inspect only patches touching capabilities we actually use. Do not import platform adapters we do not need merely because they exist.

### v0.18 — Judgment
High-priority candidates:
- evidence-based completion/judgment behavior
- `/goal` completion contracts
- `/learn` and `/journey` steering/observability
- Mixture-of-Agents where it improves independent review
- background subagent fan-out
- gateway scale-to-zero/drain coordination
- P0/P1 and security hardening

These align strongly with owner requirements: prove before claim, builder cannot approve itself, autonomous background work, and observable agent behavior.

### v0.18.1 / v0.18.2 — stabilization
High-priority candidates:
- installer/updater self-healing, especially Windows
- gateway/dashboard/MCP/provider bug fixes
- dependency/install reliability

These are candidates for a targeted stability lane, not a blind version jump.

### v0.19 / current main / v0.20 development line
Candidate categories visible on current upstream:
- session-title provenance and faster cheap-model titling
- desktop/HUD reliability and performance fixes
- media generation/upscaling routing
- plugin/provider evolution
- ongoing MCP, gateway, security, desktop and skills-standard work

Policy: adopt only if required by an active use case or if it closes a verified security/reliability gap in the fork.

## Skill inventory status
Upstream current main reports 191 in-repo skills (77 bundled, 114 optional). Top-level bundled categories currently include:
- apple
- autonomous-ai-agents
- creative
- email
- github
- index-cache
- media
- mlops
- note-taking
- productivity
- research
- smart-home
- social-media
- software-development

Do not copy all skills into the fork. Perform a machine-generated inventory and classify each as:
- ALREADY PRESENT / equivalent
- ADOPT
- ADAPT
- REFERENCE ONLY
- REJECT / duplicate / unnecessary

## Upgrade priority for this owner
P0 — preserve first:
- custom persona/SOUL
- ICM workflow architecture
- owner skills and registries
- secret handling
- existing deployments/gateway behavior

P1 — selectively adopt:
- Kanban worker lifecycle improvements not already present
- evidence/completion contracts
- webhook security/reliability fixes
- MCP OAuth/filtering/security improvements
- Windows updater/installer reliability
- background task/subagent improvements
- security patches applicable to installed surfaces

P2 — evaluate when active:
- desktop/HUD UX
- extra messaging platforms
- broad media-generation additions
- optional skills unrelated to current client/revenue workflows

PARK:
- wholesale upstream merge
- mass skill installation
- version-number chasing
- cosmetic desktop changes without an active requirement

## Safe port procedure
For each upstream candidate:
1. identify exact upstream commits/files and dependencies
2. map collision with owner-modified files
3. define acceptance tests and rollback
4. port to isolated branch/worktree
5. run existing fork tests first
6. run targeted new tests
7. independent review
8. merge only after proof
9. record provenance in this audit

## Required exhaustive follow-up artifact
Generate `UPSTREAM_SKILL_MATRIX.json` and `UPSTREAM_PATCH_MATRIX.json` from Git trees/commit history when a runtime with full GitHub tree access is available. The matrix must contain exact path/SHA/category/decision/reason/test requirement. This audit intentionally does not pretend that a version number equals a complete tree diff.
