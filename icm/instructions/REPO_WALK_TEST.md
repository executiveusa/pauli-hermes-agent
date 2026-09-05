# Mandatory Repository Walk Test

Run this before consequential work in any repository managed by personal Hermes.

## Purpose

Prove that a cold agent can understand the repository's operating boundaries before it changes code, infrastructure, data, or public behavior.

## Walk

1. **Identity** — state repo, product/client, owner, and whether the runtime is personal, client-specific, or shared infrastructure.
2. **Router** — locate `icm/`, `hicm/`, `AGENTS.md`, `CONTEXT.md`, run state, or the closest canonical operating file.
3. **Current state** — identify default branch, active release/work branch, latest relevant commit, open PRs, and current blocker.
4. **Architecture** — identify frontend, backend/runtime, data store, secret boundary, deployment target, and machine interfaces (API/MCP/CLI) relevant to the task.
5. **Dependencies** — identify directly coupled repos/services and inspect only the material recent changes that can invalidate the task plan.
6. **Blast radius** — name files/services/data/ports/domains/accounts that can be changed and those that are protected.
7. **Proof** — state the exact command/request/visual journey that can falsify the completion claim.
8. **Rollback** — identify the previous known-good revision/deploy/config and how to restore it.
9. **Human gate** — state whether external send/publish/spend/production/destructive/admin action requires owner approval.

## Cold-agent pass condition

The walk passes only when a cold agent can answer, from the repo router plus at most two additional targeted reads:
- What am I changing?
- Why is it the current bottleneck?
- What must not change?
- What exact evidence proves success?
- Where is state/evidence written?
- What is the rollback?
- What requires human approval?

If the agent must read the whole repository, the ICM/HiCM layer has failed. Open a bounded documentation/architecture slice before broad implementation.

## Output contract

Persist a short receipt in the active run/evidence surface:

```text
WALK TEST
repo:
revision:
identity:
router:
architecture:
dependencies_checked:
protected_resources:
proof_gate:
rollback:
human_gate:
result: PASS | FAIL
```

A PASS does not mean the feature works. It only admits implementation into the next bounded slice.