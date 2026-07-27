---
name: hardened-longrun-subagent-harness
description: |
  Durable, resumable orchestration for long-running Hermes tasks. Breaks a mission into a dependency graph,
  runs bounded batches of isolated delegate_task subagents, forces every worker to save an atomic JSON result,
  checkpoints after every epoch, resumes through cron-backed fresh sessions, rejects duplicate or unverifiable
  work, performs independent reviews, and exports a final-report.json plus a downloadable ZIP bundle.

  Use for: multi-hour or multi-day research, course and document ingestion, browser-learning missions,
  large repo audits, multi-file implementation plans, repeated scheduled checks, and any task where a single
  agent turn would be fragile, expensive, or too large for one context window.

  Triggers: "long running task", "keep working until complete", "use subagents", "spawn agents",
  "fan out", "resume this mission", "checkpoint the work", "run overnight", "durable agent loop",
  "save every agent result as JSON", "export the mission", "harden the loop".
version: 1.0.0
author: ExecutiveUSA
license: MIT
tags:
  - delegation
  - subagents
  - long-running
  - durable-workflows
  - checkpointing
  - orchestration
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

Turn a large objective into a durable mission that can survive context compression, new turns, interrupted
subagents, and Hermes process restarts without repeating completed work or claiming completion without proof.

This skill is an orchestration protocol, not an invitation to keep one model call alive indefinitely.

The unit of progress is an **epoch**:

1. Load canonical JSON state from disk.
2. Reconcile completed worker result files.
3. Select only dependency-ready tasks.
4. Dispatch one bounded `delegate_task` batch.
5. Validate each worker's JSON result.
6. Update state atomically.
7. Write a checkpoint and release the controller lock.
8. Exit the turn or schedule the next epoch.

For work that must survive session closure or a Hermes restart, use a skill-backed `cronjob` to start fresh
epochs. `delegate_task` is used inside an epoch for isolated reasoning and parallelism; it is not itself the
durability layer.

---

## Non-negotiable laws

1. **State lives outside model context.** JSON files are canonical; chat history is not.
2. **One mission, one directory.** Never mix state from two missions.
3. **One child, one result path.** Workers never write shared mission state.
4. **Atomic writes only.** Write `*.tmp`, fsync when possible, then rename.
5. **No unbounded loop.** Every mission has epoch, attempt, runtime, token, and cost limits.
6. **No blind retries after unknown side effects.** Reconcile first or stop for human review.
7. **No nested delegation by default.** Leaf workers do work; the parent orchestrates.
8. **No self-approval.** Builders cannot perform the final spec, quality, or security approval.
9. **No final claim without evidence.** Completion criteria must map to files, tests, citations, or logs.
10. **No automatic durable-memory insertion.** Produce candidate memory JSON and require approval.
11. **No concurrent shared-file edits.** Use isolated output folders or git worktrees.
12. **No secret material in mission JSON.** Store variable names and secret references, never values.

---

## Hermes-specific durability boundary

Hermes subagents are excellent for fresh-context reasoning and bounded parallel work. They are not resumable
processes after a Hermes crash. A child that finishes and writes its result file can be recovered. A child that
was running when the process disappeared becomes `unknown` until reconciled.

Therefore:

- Use `delegate_task` for bounded reasoning tasks within one epoch.
- Use `cronjob` for fresh, recurring, process-independent mission epochs.
- Use `terminal(background=True, notify_on_complete=True)` only for deterministic shell processes that do not
  need model reasoning.
- Never assume a missing parent summary means a child did no work; inspect its result and side-effect records.

---

## Default mission directory

Use a project-local directory unless the user specifies a private durable location:

```text
.hermes-runs/<mission_id>/
├── mission.json
├── state.json
├── plan.json
├── events.jsonl
├── locks/
│   └── controller.lock
├── tasks/
│   └── <task_id>.json
├── attempts/
│   └── <task_id>/
│       └── attempt-001/
│           ├── request.json
│           ├── result.json
│           └── transcript-path.txt
├── checkpoints/
│   └── epoch-0001.json
├── artifacts/
├── candidate-memory/
│   └── candidate-memory.json
├── reviews/
│   ├── spec-review.json
│   ├── quality-review.json
│   └── security-review.json
└── outputs/
    ├── final-report.json
    ├── export-manifest.json
    └── <mission_id>.zip
```

If the mission operates on private personal knowledge, prefer:

```text
~/.hermes/runs/<mission_id>/
```

Do not commit private source captures, credentials, browser profiles, or personal second-brain data to Git.

---

## State machines

### Mission states

```text
draft
  → ready
  → running_epoch
  → waiting_workers
  → reconciling
  → synthesizing
  → reviewing
  → waiting_approval
  → completed

Any active state may move to:
paused | blocked | failed | cancelled | rolled_back
```

### Task states

```text
pending → ready → dispatched → running → completed
                         ├────→ needs_review
                         ├────→ retry_scheduled
                         ├────→ blocked
                         ├────→ failed
                         └────→ unknown
```

`unknown` is a safety state. Do not convert it to `pending` automatically when the task may have produced an
external side effect.

---

## Task classification

Classify every task before dispatch.

### Use a subagent when

- The task requires judgment, reasoning, interpretation, research, synthesis, or review.
- It benefits from a fresh context.
- Its intermediate tool output would pollute the parent context.
- It can produce an independent result file.

### Use `execute_code` or a script when

- The task is deterministic transformation, validation, aggregation, hashing, sorting, or file packaging.
- The algorithm is known and no model judgment is needed.

### Use a direct tool call when

- Only one tool action is required.

### Use cron-backed epochs when

- The mission must continue after the current turn.
- It runs for hours or days.
- It checks for new work periodically.
- It must recover after a process restart.

---

## Side-effect classes

Every task must declare one:

```text
none               Pure analysis or local read-only work; safe to retry after validation.
local_reversible   Writes isolated local artifacts with a known rollback path.
approval_required  External write, communication, deployment, publish, or account change.
irreversible       Must not run autonomously. Human performs or explicitly approves each action.
```

Rules:

- `none`: may be retried when no valid result exists.
- `local_reversible`: retry only after checking idempotency key and existing artifacts.
- `approval_required`: stop before execution and create an approval record.
- `irreversible`: the task may prepare instructions but cannot execute autonomously.

---

## Controller protocol

### 1. Intake and lock

Before substantial work, establish:

- mode: greenfield or brownfield;
- measurable outcome;
- target user, customer, or system;
- constraints;
- proof required;
- commercial value;
- maximum epochs;
- maximum attempts per task;
- maximum concurrent children;
- maximum iterations per child;
- time, token, and cost budgets;
- rollback strategy.

Create the mission with:

```bash
python skills/hardened-longrun-subagent-harness/scripts/mission_state.py init \
  --mission-file /absolute/path/mission.json \
  --run-root /absolute/path/.hermes-runs
```

Acquire the controller lease before each epoch:

```bash
python skills/hardened-longrun-subagent-harness/scripts/mission_state.py lock \
  --run /absolute/path/.hermes-runs/<mission_id> \
  --owner "hermes:<session_id>:epoch-0001" \
  --ttl-seconds 1800
```

If lock acquisition fails, do not run a second controller.

### 2. Reconcile before dispatch

Run:

```bash
python skills/hardened-longrun-subagent-harness/scripts/mission_state.py reconcile \
  --run /absolute/path/.hermes-runs/<mission_id>
```

Reconciliation must:

- discover valid `attempts/*/result.json` files;
- verify mission ID, task ID, attempt number, and idempotency key;
- reject malformed or partial JSON;
- detect duplicate result files;
- mark completed tasks only when their acceptance checks pass;
- mark interrupted attempts `unknown` when side effects cannot be disproven;
- expose ready tasks whose dependencies are completed.

### 3. Select one bounded batch

Default maximum: 3 concurrent children.

Never dispatch more children than:

```text
min(mission.max_concurrent_children, Hermes configured delegation limit, number of ready tasks)
```

Prioritize:

1. Tasks blocking the most downstream work.
2. Tasks with no side effects.
3. Tasks with the lowest retry count.
4. Tasks needed for the next synthesis gate.

Do not dispatch tasks whose dependencies are incomplete.

### 4. Build complete child packets

Subagents know nothing about the parent conversation. Every child context must include:

- absolute mission directory;
- mission ID and objective;
- exact task ID, goal, and acceptance criteria;
- exact files it may read;
- exact result path it owns;
- worker-result schema path;
- current attempt number;
- idempotency key;
- required evidence;
- allowed and prohibited actions;
- validation command;
- budget and maximum iterations;
- instruction to write JSON before returning a summary.

Bad:

```text
Fix the problem and report back.
```

Good child goal:

```text
Analyze lesson-014 from the source package at the exact paths below. Produce five or more evidence-backed
knowledge units. Do not edit shared state. Atomically write the result JSON to the assigned attempt path,
validate it, then return a concise summary containing the result path and status.
```

### 5. Dispatch leaf workers

Use a batch of leaf agents. Do not grant nested orchestration unless the mission explicitly requires a second
hierarchical reduction layer and the configured spawn depth supports it.

Conceptual call:

```python
delegate_task(
    tasks=[
        {
            "goal": task_1_goal,
            "context": task_1_complete_context,
            "max_iterations": 20,
        },
        {
            "goal": task_2_goal,
            "context": task_2_complete_context,
            "max_iterations": 20,
        },
    ]
)
```

Use smaller iteration limits for narrow tasks. Never use the default 50 merely because it exists.

### 6. Worker completion contract

Before returning to the parent, every worker must:

1. Read its task file.
2. Inspect only allowed inputs.
3. Perform the task.
4. Collect evidence.
5. Write its result to `<result>.tmp`.
6. Validate required fields.
7. Compute a content hash.
8. Rename to `result.json` atomically.
9. Return a short summary with:
   - task ID;
   - status;
   - result path;
   - files touched;
   - validation performed;
   - unresolved risks.

A final chat summary without a valid result file does not complete the task.

### 7. Checkpoint after every batch

After worker completion or interruption:

```bash
python skills/hardened-longrun-subagent-harness/scripts/mission_state.py reconcile \
  --run /absolute/path/.hermes-runs/<mission_id>

python skills/hardened-longrun-subagent-harness/scripts/mission_state.py checkpoint \
  --run /absolute/path/.hermes-runs/<mission_id> \
  --note "Epoch 1: dispatched tasks A, B, C; A and C completed; B retry scheduled."
```

Then release the lock.

### 8. Progress and stall policy

Progress means at least one of:

- task moved to completed;
- a verified artifact was added;
- a blocker was resolved;
- a review gate passed;
- a candidate-memory item was accepted or rejected;
- mission completion criteria moved measurably closer.

Stop and mark `blocked` when:

- three consecutive epochs produce no measurable progress;
- dependency deadlock exists;
- the same normalized error occurs three times;
- budget is exhausted;
- controller lock ownership is uncertain;
- external side effects are `unknown`;
- required authentication or user input is missing;
- evidence is insufficient to validate completion.

Never hide a stall by creating more tasks.

---

## Hierarchical fan-out and fan-in

For large collections, use reduction layers:

```text
source inventory
  → leaf workers
  → group/module synthesizers
  → mission synthesizer
  → independent reviewers
  → final JSON export
```

Examples:

- Course: lesson students → module professors → course dean → ICM librarian.
- Repo: package auditors → subsystem reviewers → architecture synthesizer.
- Research: source researchers → topic synthesizers → final analyst.

The final synthesizer should consume validated worker JSON and group syntheses, not every raw transcript.

---

## Coding-task isolation

For concurrent code changes:

- Create one Git worktree or branch per worker.
- Never allow multiple children to edit the same worktree.
- Each worker records commit SHA, changed files, tests, and rollback command in its JSON.
- An integrator subagent reviews and combines completed branches.
- Independent QA and security agents review the integrated state.

For research, learning, or content analysis:

- Inputs are read-only.
- Each worker writes only to its unique attempt directory.
- Synthesizers write only to their assigned output paths.

---

## Review gates

After all implementation or research tasks complete, dispatch fresh reviewers.

### Spec reviewer

Checks every completion criterion against artifacts and worker results.

### Quality reviewer

Checks correctness, coherence, duplication, omissions, and usability.

### Security reviewer

Required when the mission touches credentials, authentication, PII, browser sessions, deployments, external
communications, or public publishing.

Reviewers write JSON under `reviews/`. They cannot modify the artifacts they review.

Allowed verdicts:

```text
pass
pass_with_conditions
fail
```

A `fail` creates bounded repair tasks. It does not silently reopen the entire mission.

---

## Finalization and JSON export

When all required tasks and reviews pass:

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

`final-report.json` is the canonical finished artifact. It must include:

- mission identity and objective;
- final status;
- completion-criterion results;
- task results and attempts;
- synthesis;
- evidence index;
- files and artifacts;
- reviews and verdicts;
- costs and runtime when available;
- risks and unresolved items;
- rollback instructions;
- next action;
- human approvals;
- provenance and hashes.

---

## Cron-backed continuation

For missions that must survive turns or restarts, create a recurring skill-backed cron job that runs exactly one
epoch per invocation.

Conceptual Hermes call:

```python
cronjob(
    action="create",
    skill="hardened-longrun-subagent-harness",
    prompt=(
        "Resume the mission at /absolute/path/.hermes-runs/<mission_id>. "
        "Run exactly one bounded epoch: lock, reconcile, dispatch at most the configured batch, "
        "checkpoint, unlock, and exit. If completed or blocked, do not dispatch more workers."
    ),
    schedule="every 10m",
    name="Mission <mission_id> controller",
)
```

The cron job is a wake-up mechanism, not the source of truth. The mission directory is the source of truth.

When the mission becomes `completed`, `blocked`, `failed`, or `cancelled`, pause or remove the cron job.

---

## Browser and authenticated missions

For browser-learning or authenticated sites:

- Use one controlled browser collector unless explicit platform-safe parallelism is proven.
- Fan out analysis against locally captured source packages, not multiple concurrent account sessions.
- Never store passwords, raw cookies, tokens, MFA codes, or browser-profile contents in mission JSON.
- Classify all browser actions as read-only, reversible, approval-required, or prohibited.
- Stop on CAPTCHA, security challenge, platform warning, access denial, or unclear authorization.
- Record source URLs, timestamps, coverage labels, and capture dates in worker evidence.

---

## Second-brain policy

Workers and synthesizers may create:

```text
candidate-memory/candidate-memory.json
```

They may not directly write approved durable memory.

Each candidate must include:

- statement;
- type;
- subject and topics;
- source references;
- evidence locations;
- confidence;
- conflicts with existing knowledge;
- proposed destination;
- why it matters;
- approval status: `candidate`.

Human or independently authorized review moves candidates into approved memory.

---

## Completion response

Every completed, blocked, or failed mission ends with:

### DECISION
What the controller concluded.

### CHANGES
Artifacts, files, task states, and external actions.

### PROOF
Tests, citations, hashes, screenshots, logs, and review verdicts.

### STATUS
Completed, partially completed, blocked, failed, cancelled, or rolled back.

### COMMERCIAL IMPACT
Revenue, savings, retention, reusable capability, or validated learning.

### RISKS
Technical, evidence, security, cost, permission, and operational risks.

### ROLLBACK
Exact rollback steps.

### NEXT
One highest-value next action.

### HUMAN APPROVAL
All pending approvals and candidate-memory decisions.

---

## Prohibited completion claims

Do not say:

- "done" because children stopped;
- "tests pass" without captured test output;
- "deployed" because a deploy command was issued;
- "saved to memory" when only a candidate file exists;
- "all lessons reviewed" when only transcripts were processed;
- "no side effects" when an interrupted task is still `unknown`;
- "resumable" until a forced interruption and recovery test has passed.
