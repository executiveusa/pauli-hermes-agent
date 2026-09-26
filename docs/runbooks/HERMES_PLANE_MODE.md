# Hermes Plane Mode

Purpose: keep Hermes usable as the single operator while the owner is away from a workstation.

## Target state

Hermes runs continuously on the sovereign VPS. GitHub is the source of truth. Hermes may supervise up to five active repositories at once and dispatch isolated workers for mutating work.

Hermes itself is the governor; disposable workers perform builds/tests/reviews. Do not grant broad Docker-host control to arbitrary project code.

## Operating contract

Owner instruction example:

`Pipeline these repos: executiveusa/repo-a, executiveusa/repo-b, executiveusa/repo-c.`

Hermes must:
1. reject protected projects;
2. enforce `max_active_projects: 5`;
3. run a read-only audit first;
4. lock a bounded Definition of Done;
5. create an isolated branch/worktree or sandbox for mutations;
6. run trusted tests;
7. require an independent review context;
8. create/update a PR rather than write directly to main;
9. create a preview when a deployment target is configured;
10. verify proof and preserve rollback;
11. report COMPLETE or BLOCKED with evidence.

## Protected projects

Until the owner explicitly removes the protection, do not modify, deploy, rename, delete, or schedule mutating jobs against:

- `executiveusa/strapi-template-new-world-kids`
- `new-world-kids-v1`

Read-only portfolio inventory should omit these from automated mutation recommendations.

## Five-project pipeline

- `0-5 active`: normal
- sixth project: QUEUED
- only one mutating job per repository
- read-only checks may run concurrently
- terminal jobs free a slot
- failed jobs do not silently retry beyond the configured retry budget

## Cron jobs to install

These are policy intents. Use Hermes' built-in scheduler on the deployed runtime so the jobs survive chat sessions. Keep schedules in the runtime's persistent Hermes home and do not store secret values in Git.

### Hourly — active repository health
Check active repositories for failed CI, new review requests, deployment failure, and blocking issues. Notify only on a material change or blocker.

### Every 6 hours — PR repair loop
For active repositories, inspect PRs with failed CI or requested changes. When the repair is bounded and safe, dispatch an isolated worker to a branch, run tests, and update the PR. Never push directly to main.

### Daily — security and dependency review
Inspect active repositories for new dependency/security/secret/test regressions. Open bounded repair PRs where safe. Never print secret values.

### Daily morning — pipeline briefing
Return active five, queue, state, blockers, verification evidence, deployment state, and next bounded action per project.

### Weekly — portfolio hygiene
Identify stale branches, duplicates, abandoned previews, old experiments, and archive candidates. Report only; do not delete automatically.

## Cron mutation boundary

Autonomous:
- read-only repo/status checks
- summaries
- branch creation
- bounded repair commits
- tests
- independent review
- PR creation/update

Human gate:
- destructive production actions
- financial actions
- secret rotation
- irreversible data migrations
- production deployment when the target or rollback is not already verified

## Runtime verification checklist

A Hermes release is not SHIPPED until all pass:

- container/service restarts automatically after process failure/reboot;
- persistent Hermes home mounted;
- gateway responds;
- configured messaging channel responds;
- scheduler survives restart;
- one read-only cron fires and records a receipt;
- one disposable worker can inspect a test repository;
- one bounded branch/PR job completes without a direct-main write;
- five simultaneous project slots are enforced;
- protected-project guard blocks New World Kids;
- health/status can be checked remotely;
- rollback command/version is recorded.

## Owner-facing commands

The control surface should normalize these intents:

- `pipeline status`
- `pipeline add <owner/repo>`
- `pipeline start <repo...>`
- `pipeline pause <repo>`
- `pipeline queue`
- `pipeline proof <repo>`
- `cron status`
- `cron list`

Natural-language equivalents are acceptable; the behavior and receipts are authoritative.
