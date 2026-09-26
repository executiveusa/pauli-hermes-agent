---
name: hardened-longrun-subagent-harness
description: |
  Durable orchestration for multi-hour and multi-day Hermes missions. Converts a large objective into a bounded
  dependency graph, generates complete subagent task packets, runs limited delegate_task batches, persists worker
  heartbeats and micro-checkpoints, safely recovers after process crashes, reconciles external side effects before
  retries, applies provider-aware concurrency and cost backpressure, performs independent reviews, and exports a
  canonical final-report.json plus a portable ZIP.

  Use for course learning, document and video ingestion, browser research, repository audits, parallel coding,
  scheduled monitoring, second-brain preparation, and any task too large or fragile for one agent context.

  Triggers: long running task, keep working until complete, use subagents, spawn agents, fan out, run overnight,
  resume mission, checkpoint work, save results as JSON, durable loop, export mission.
version: 1.1.0
author: ExecutiveUSA
license: MIT
tags:
  - delegation
  - subagents
  - long-running
  - checkpointing
  - crash-recovery
  - side-effect-safety
  - adaptive-concurrency
  - json
  - cron
  - second-brain
triggers:
  - long running task
  - use subagents
  - spawn agents
  - fan out
  - run overnight
  - resume mission
  - checkpoint work
  - durable loop
  - save results as json
  - export mission
---

# Hardened Long-Running Subagent Harness

## Mission

Run large Hermes tasks as a sequence of short, durable **epochs** rather than one endlessly growing model turn.
Every epoch loads state from JSON, recovers interrupted workers, validates complete child packets, dispatches only
one bounded batch, records progress, checkpoints, and exits.

The model process itself is not resumed after a crash. The **task is resumed** from its latest durable micro-checkpoint
by a new child with a new attempt and an explicit `resume_from` path. This is the only supported meaning of resumable.

## Mandatory laws

1. `mission.json`, `state.json`, worker results, checkpoints, and receipts are canonical; chat history is not.
2. One mission uses one run directory.
3. One worker owns one attempt directory and never edits shared mission state.
4. All JSON writes use a temporary file, fsync, and atomic rename.
5. Every mission has epoch, attempt, runtime, token, cost, and concurrency limits.
6. Global concurrency is hard-capped at eight; three is the recommended default.
7. Provider cooldowns and adaptive concurrency are obeyed after rate limits.
8. Every child receives a generated and validated task packet with absolute paths and explicit constraints.
9. Every active worker sends heartbeats and writes checkpoints after completed units of work.
10. External side effects are never retried after an interruption without a reconciliation receipt.
11. `irreversible` tasks are never autonomously dispatched.
12. Builders cannot approve their own outputs.
13. Durable second-brain memory remains approval-gated.
14. Completion requires evidence tied to explicit criteria and independent review.
15. Secrets, passwords, cookies, session tokens, and MFA codes never enter mission JSON.

## Canonical run directory

```text
.hermes-runs/<mission_id>/
├── mission.json
├── state.json
├── events.jsonl
├── locks/controller.lock
├── approvals/<task_id>.json
├── tasks/<task_id>.json
├── attempts/<task_id>/attempt-001/
│   ├── request.json
│   ├── attempt-state.json
│   ├── side-effects.jsonl
│   ├── reconciliation-required.json
│   ├── side-effect-receipt.json
│   ├── checkpoints/checkpoint-0001.json
│   └── result.json
├── checkpoints/epoch-0001.json
├── artifacts/
├── candidate-memory/candidate-memory.json
├── reviews/
│   ├── spec-review.json
│   ├── quality-review.json
│   └── security-review.json
└── outputs/
    ├── synthesis.json
    ├── completion-criteria.json
    ├── final-report.json
    ├── export-manifest.json
    └── <mission_id>.zip
```

Use `~/.hermes/runs/<mission_id>/` for private knowledge. Do not commit private captures.

## Side-effect classes

```text
none               Pure read-only work; safely retryable from the latest checkpoint.
local_reversible   Isolated local changes; retry only from a checkpoint marked local_state_consistent=true.
approval_required  External write; requires a current approval file and reconciliation after interruption.
irreversible       Human-only execution. The agent may prepare instructions but cannot dispatch the action.
```

## Required mission limits

```json
{
  "max_epochs": 20,
  "max_attempts_per_task": 3,
  "max_concurrent_children": 3,
  "provider_concurrency_cap": 3,
  "max_iterations_per_child": 20,
  "max_runtime_minutes": 720,
  "max_total_cost_usd": 5,
  "max_dispatch_cost_usd": 0.75,
  "max_total_tokens": 500000
}
```

`max_concurrent_children` and `provider_concurrency_cap` must be between one and eight.

## Full epoch procedure

### 1. Initialize once

```bash
python skills/hardened-longrun-subagent-harness/scripts/mission_state.py init \
  --mission-file /absolute/path/mission.json \
  --run-root /absolute/path/.hermes-runs
```

### 2. Acquire one controller lock

```bash
python skills/hardened-longrun-subagent-harness/scripts/mission_state.py lock \
  --run /absolute/path/.hermes-runs/<mission_id> \
  --owner "hermes:<session_id>:epoch-0001" \
  --ttl-seconds 1800
```

A failed lock means another controller owns the mission. Exit without dispatching.

### 3. Recover interrupted attempts

```bash
python skills/hardened-longrun-subagent-harness/scripts/risk_controls.py recover \
  --run /absolute/path/.hermes-runs/<mission_id>
```

Recovery rules:

- A completed `result.json` waits for normal reconciliation.
- An expired read-only worker becomes `retry_scheduled` with `resume_from` set to its latest checkpoint.
- A local reversible worker resumes only from a checkpoint that declares `local_state_consistent: true`.
- An interrupted external action becomes `needs_reconciliation` and blocks the mission.
- Missing attempt state becomes `unknown` and blocks the mission.

### 4. Reconcile finished worker results

```bash
python skills/hardened-longrun-subagent-harness/scripts/mission_state.py reconcile \
  --run /absolute/path/.hermes-runs/<mission_id>
```

A child summary without a valid `result.json` does not complete a task.

### 5. Generate the bounded dispatch plan

```bash
python skills/hardened-longrun-subagent-harness/scripts/risk_controls.py prepare-batch \
  --run /absolute/path/.hermes-runs/<mission_id> \
  --controller "hermes:<session_id>:epoch-0001" \
  --worker-ttl-seconds 900
```

This command is mandatory. Do not manually improvise subagent contexts.

It performs all of the following before returning dispatchable children:

- verifies dependencies;
- rejects missing goal, context, criteria, evidence, input, output, or prohibition fields;
- creates absolute result and checkpoint paths;
- enforces the global hard cap;
- enforces provider concurrency;
- observes provider cooldowns;
- limits each batch by remaining mission and dispatch cost budgets;
- checks approval records for external actions;
- writes one atomic `request.json` and `attempt-state.json` per child;
- includes a `resume_from` path when recovering a crashed task.

If it returns no dispatch items, do not create workers manually.

### 6. Dispatch exactly the returned packets

For each dispatch item, read its `request_path` and pass the complete packet to one leaf `delegate_task` child.
Use the returned `max_iterations`. Do not permit nested delegation unless the mission explicitly defines a second
reduction layer.

Conceptual Hermes call:

```python
delegate_task(tasks=[
    {
        "goal": packet["goal"],
        "context": json.dumps(packet),
        "max_iterations": packet["limits"]["max_iterations"]
    }
])
```

Never dispatch more children than the generated plan.

### 7. Worker heartbeat and checkpoint

After every completed unit of work, the child prepares a checkpoint matching
`schemas/task-checkpoint.schema.json`, then the controller records it:

```bash
python skills/hardened-longrun-subagent-harness/scripts/risk_controls.py heartbeat \
  --run /absolute/path/.hermes-runs/<mission_id> \
  --task-id <task_id> \
  --attempt <attempt> \
  --worker-id <worker_id> \
  --worker-ttl-seconds 900 \
  --checkpoint-file /absolute/path/checkpoint.json
```

A checkpoint contains completed units, next unit, artifacts, side effects, and whether local state is consistent.
A replacement child receives the checkpoint path through `resume_from` and must continue after completed units,
not redo them.

### 8. External side-effect journal and reconciliation

Every external action is journaled before execution with its idempotency key and after execution with the observed
result. If the child disappears, the mission blocks.

A human or independent verifier checks the external system and writes a receipt matching
`schemas/side-effect-receipt.schema.json`:

```bash
python skills/hardened-longrun-subagent-harness/scripts/risk_controls.py reconcile-side-effect \
  --run /absolute/path/.hermes-runs/<mission_id> \
  --task-id <task_id> \
  --attempt <attempt> \
  --receipt-file /absolute/path/receipt.json
```

Outcomes:

- `not_applied` or `rolled_back`: a new attempt may be dispatched.
- `applied`: no retry; task moves to independent review.
- `partially_applied` or `unknown`: mission remains blocked.

### 9. Rate-limit backpressure

On an HTTP 429 or equivalent provider signal:

```bash
python skills/hardened-longrun-subagent-harness/scripts/risk_controls.py record-rate-limit \
  --run /absolute/path/.hermes-runs/<mission_id> \
  --provider <provider> \
  --retry-after-seconds <seconds>
```

This records a cooldown and halves both provider and adaptive concurrency, never below one. Do not bypass the
cooldown by switching to parallel children on the same constrained provider.

### 10. Reconcile and checkpoint the epoch

```bash
python skills/hardened-longrun-subagent-harness/scripts/mission_state.py reconcile \
  --run /absolute/path/.hermes-runs/<mission_id>

python skills/hardened-longrun-subagent-harness/scripts/mission_state.py checkpoint \
  --run /absolute/path/.hermes-runs/<mission_id> \
  --note "Epoch summary with dispatched, completed, recovered, blocked, and pending items."
```

Release the controller lock and exit the epoch.

## Hierarchical fan-out and fan-in

Use reduction layers:

```text
source inventory
  → leaf workers
  → module or subsystem synthesizers
  → mission synthesizer
  → independent reviewers
  → final JSON export
```

Examples:

- Course: lesson students → module professors → course dean → ICM librarian.
- Repository: package auditors → subsystem reviewers → architecture synthesizer.
- Research: source analysts → topic synthesizers → final analyst.

Synthesizers consume validated worker JSON and group syntheses, not every raw transcript.

## Coding isolation

Concurrent code workers receive separate Git worktrees or branches. Each result records commit SHA, files changed,
tests, evidence, and rollback. A separate integrator combines branches. Fresh QA and security reviewers inspect the
integrated state.

## Browser learning

Use one controlled authenticated browser collector unless platform-safe concurrency is proven. Fan out analysis on
local lesson packages. Stop on CAPTCHA, security challenge, access denial, platform warning, or unclear authorization.
Never store credentials or session material in mission files.

## Review gates

Fresh reviewers write:

- `reviews/spec-review.json`
- `reviews/quality-review.json`
- `reviews/security-review.json` when authentication, PII, deployment, communication, publishing, or side effects exist.

Allowed verdicts are `pass`, `pass_with_conditions`, and `fail`. A failure creates bounded repair tasks and does not
silently reopen the entire mission.

## Second-brain policy

Workers may write `candidate-memory/candidate-memory.json`. They cannot write approved durable memory. Every entry
must include statement, type, subject, topics, sources, evidence locations, confidence, conflicts, proposed destination,
usefulness, and `approval_status: candidate`.

## Finalization and export

Required before finalization:

```text
outputs/synthesis.json
outputs/completion-criteria.json
reviews/spec-review.json
reviews/quality-review.json
reviews/security-review.json  # when applicable
```

Then run:

```bash
python skills/hardened-longrun-subagent-harness/scripts/mission_state.py finalize \
  --run /absolute/path/.hermes-runs/<mission_id>

python skills/hardened-longrun-subagent-harness/scripts/export_mission.py \
  --run /absolute/path/.hermes-runs/<mission_id>
```

Required outputs:

```text
outputs/final-report.json
outputs/export-manifest.json
outputs/<mission_id>.zip
```

The final report contains mission identity, status, criteria proof, task attempts, synthesis, evidence, artifacts,
reviews, usage, risks, unresolved items, rollback, next action, approvals, and provenance hashes.

## Cron-backed continuation

For unattended work, create a skill-backed cron job that runs one epoch per invocation:

```python
cronjob(
    action="create",
    skill="hardened-longrun-subagent-harness",
    prompt=(
        "Resume the mission at /absolute/path/.hermes-runs/<mission_id>. "
        "Run one bounded epoch using mission_state.py and risk_controls.py. "
        "Recover, reconcile, prepare only the generated batch, checkpoint, unlock, and exit. "
        "Do not dispatch when completed, blocked, failed, cancelled, or waiting for reconciliation."
    ),
    schedule="every 10m",
    name="Mission <mission_id> controller"
)
```

Pause or remove the cron job at every terminal or blocked state.

## Required verification

Before calling this harness production-ready, run:

```bash
python skills/hardened-longrun-subagent-harness/tests/smoke_test.py
python skills/hardened-longrun-subagent-harness/tests/risk_controls_test.py
```

The risk suite must prove:

- two-level bounded fan-out;
- incomplete context packet rejection;
- forced worker interruption;
- recovery from a durable checkpoint;
- blocked retry for an uncertain external side effect;
- retry only after a valid reconciliation receipt;
- hard concurrency cap;
- provider cooldown and adaptive reduction.

## Completion response

Every mission ends with DECISION, CHANGES, PROOF, STATUS, COMMERCIAL IMPACT, RISKS, ROLLBACK, NEXT, and HUMAN APPROVAL.

Never claim done because children stopped, tests passed without captured output, deployment because a command ran,
memory saved when only a candidate exists, no side effects while reconciliation is pending, or resumability before the
forced interruption and recovery test passes.
