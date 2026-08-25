---
name: portfolio-pipeline
description: Operate up to five GitHub projects concurrently through audit, bounded build, independent review, preview, verification, and evidence-backed completion.
---

# Portfolio Pipeline

Use this skill when the owner asks Hermes to work on, pipeline, finish, audit, repair, or ship repositories.

## Required policy

Read `${HERMES_SKILL_DIR}/policy.yaml` before dispatching work. The installed skill directory is the runtime source of policy; repository-level mirrors are documentation/configuration aids only.

## Intake

For every requested repository:

1. resolve the exact canonical `owner/repo`;
2. normalize aliases to the canonical slug before protected-project comparison;
3. block protected projects;
4. check whether it already occupies an active slot;
5. if five projects are active, queue it FIFO;
6. create a durable mission record with outcome, non-goals, proof, budget, and rollback.

## Stage 01 — Audit

Read only. Establish:
- canonical purpose;
- default branch and current HEAD;
- stack/runtime;
- tests/CI;
- deployment target if any;
- environment-variable names only;
- open PRs and blockers;
- duplicate/predecessor/successor relationships;
- security and secret exposure risk.

Disposition: KEEP / FINISH / CONSOLIDATE / MIGRATE / PARK / ARCHIVE.

Do not mutate before the disposition and bounded Definition of Done are recorded.

## Stage 02 — Lock

Record:
- problem;
- desired outcome;
- Definition of Done;
- non-goals;
- proof required;
- rollback;
- worker/runtime;
- cost/runtime budget.

## Stage 03 — Execute

For mutating work:
- create isolated branch/worktree or stronger sandbox;
- never write directly to main;
- use a disposable worker;
- preserve existing architecture unless the mission requires changing it;
- never expose secrets.

## Stage 04 — Test

Run repository-native tests/build/lint/typecheck plus the mission-specific trusted checks.

## Stage 05 — Independent review

Builder cannot self-approve. A separate context/worker returns PASS or BLOCK with evidence.

## Stage 06 — Preview and verify

If a deployment target exists, create preview/staging first. Verify critical routes/API/mobile behavior appropriate to the project. Keep previous production as rollback.

## Stage 07 — PR and promotion

Create/update a PR with proof. Production promotion is allowed only when the configured policy permits it and rollback is known. Otherwise return READY_FOR_APPROVAL.

## Terminal receipt

Return:
- repo;
- base commit;
- result commit;
- branch/PR;
- tests;
- independent review;
- preview/deployment;
- changed files;
- blockers;
- rollback;
- final state.

## Cron behavior

Scheduled jobs must obey the same policy. Read-only checks may run autonomously. Bounded repair jobs may create/update branches and PRs. Cron must never bypass protected-project, direct-main, secret, financial, or unverified-production gates.
