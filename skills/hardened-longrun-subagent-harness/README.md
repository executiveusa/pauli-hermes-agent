# Hardened Long-Running Subagent Harness

A native Hermes skill for durable missions that use bounded parallel subagents without relying on one fragile, endlessly growing agent turn.

## What it adds

- Filesystem-backed mission state
- Dependency-aware task graphs
- Bounded `delegate_task` batches
- Atomic per-worker JSON result files
- Expiring controller locks
- Retry, stall, and unknown-side-effect handling
- Independent spec, quality, and security review gates
- Cron-backed fresh-session continuation
- Canonical `final-report.json`
- Export manifest with SHA-256 hashes
- Downloadable mission ZIP

## Why this design

Hermes can delegate isolated tasks and keep their intermediate tool calls out of the parent context. However, an active child is not resumable after a Hermes process crash. This skill stores durable progress in JSON and treats every Hermes invocation as one bounded epoch. A recurring cron job can start the next epoch from disk after a restart.

## Install location

Bundled repository skill:

```text
skills/hardened-longrun-subagent-harness/
```

User-level installation can also copy this folder to:

```text
~/.hermes/skills/hardened-longrun-subagent-harness/
```

## Quick start

1. Copy the example mission and customize it.

```bash
cp skills/hardened-longrun-subagent-harness/examples/mission.example.json /tmp/my-mission.json
```

2. Initialize the run.

```bash
python skills/hardened-longrun-subagent-harness/scripts/mission_state.py init \
  --mission-file /tmp/my-mission.json \
  --run-root "$PWD/.hermes-runs"
```

3. Ask Hermes:

```text
Use the hardened-longrun-subagent-harness skill. Resume the mission at
/path/to/.hermes-runs/example-course-intelligence. Run exactly one epoch.
```

4. For unattended continuation, ask Hermes to create a skill-backed cron job that runs one epoch every ten minutes.

5. After task completion, create the required review JSON files and synthesis artifacts, then finalize and export.

```bash
python skills/hardened-longrun-subagent-harness/scripts/mission_state.py finalize \
  --run "$PWD/.hermes-runs/example-course-intelligence"

python skills/hardened-longrun-subagent-harness/scripts/export_mission.py \
  --run "$PWD/.hermes-runs/example-course-intelligence"
```

## Worker result rule

A worker's chat summary is not canonical. Each worker must atomically write:

```text
attempts/<task_id>/attempt-<NNN>/result.json
```

The result must match `schemas/worker-result.schema.json` and include:

- identity and attempt
- status
- summary and findings
- evidence and artifacts
- files touched and rollback notes
- validations
- risks and unresolved items
- side effects and idempotency keys
- model usage when available
- content hash

## Files required before final export

```text
outputs/synthesis.json
outputs/completion-criteria.json
reviews/spec-review.json
reviews/quality-review.json
```

`reviews/security-review.json` is additionally required when any task is not classified as `none` side effect.

Optional:

```text
outputs/rollback.json
outputs/next.json
outputs/human-approval.json
candidate-memory/candidate-memory.json
```

## Expected completion-criteria file

```json
{
  "criteria": [
    {
      "id": "C1",
      "description": "All required tasks completed",
      "status": "pass",
      "proof": ["state.json", "attempts/task-01/attempt-001/result.json"]
    }
  ]
}
```

## Expected review file

```json
{
  "reviewer": "independent-spec-reviewer",
  "reviewed_at": "2026-07-27T12:00:00Z",
  "verdict": "pass",
  "findings": [],
  "proof": ["outputs/completion-criteria.json"]
}
```

## Safety defaults

- Maximum three concurrent children unless deliberately configured otherwise
- No nested delegation by default
- No direct worker writes to shared state
- No retries after uncertain external side effects
- No durable-memory approval by workers
- No completion without independent reviews
- No secret values in mission files
- No unbounded self-reflection or task creation

## Validation

The included utilities use only the Python standard library.

Basic syntax check:

```bash
python -m py_compile \
  skills/hardened-longrun-subagent-harness/scripts/mission_state.py \
  skills/hardened-longrun-subagent-harness/scripts/export_mission.py
```

Initialize smoke test:

```bash
TMP_ROOT="$(mktemp -d)"
python skills/hardened-longrun-subagent-harness/scripts/mission_state.py init \
  --mission-file skills/hardened-longrun-subagent-harness/examples/mission.example.json \
  --run-root "$TMP_ROOT"
python skills/hardened-longrun-subagent-harness/scripts/mission_state.py status \
  --run "$TMP_ROOT/example-course-intelligence"
```

## Final deliverables

A completed mission produces:

```text
outputs/final-report.json
outputs/export-manifest.json
outputs/<mission_id>.zip
```

The JSON report is the canonical machine-readable result. The ZIP is the portable handoff and download package.
