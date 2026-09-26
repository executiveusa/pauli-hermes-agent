# Hardened Long-Running Subagent Harness

Native Hermes skill for durable long-running missions using bounded parallel subagents, filesystem JSON state,
worker leases, micro-checkpoints, safe side-effect reconciliation, adaptive concurrency, independent reviews,
and final JSON/ZIP export.

## Risk controls

The harness closes the primary long-running-agent failure modes:

- **Interrupted children:** a replacement child resumes the task from its latest validated checkpoint. The original
  model process is not resumed.
- **External side effects:** interrupted external actions block until an independent reconciliation receipt proves
  whether the action was applied, not applied, rolled back, partial, or unknown.
- **Cost and rate limits:** global and provider concurrency are capped at eight, dispatch cost is budgeted, and a
  provider rate limit creates a cooldown while reducing adaptive concurrency.
- **Incomplete child context:** task packets are generated from validated mission contracts. Missing context, paths,
  criteria, evidence requirements, outputs, or constraints block dispatch.
- **Release verification:** lifecycle and risk-control tests run in the dedicated GitHub Actions workflow.

## Quick start

Initialize the example mission:

```bash
python skills/hardened-longrun-subagent-harness/scripts/mission_state.py init \
  --mission-file skills/hardened-longrun-subagent-harness/examples/mission.example.json \
  --run-root "$PWD/.hermes-runs"
```

Prepare one bounded batch:

```bash
python skills/hardened-longrun-subagent-harness/scripts/risk_controls.py prepare-batch \
  --run "$PWD/.hermes-runs/example-course-intelligence" \
  --controller "hermes:session:epoch-0001"
```

Hermes must delegate only the task packets returned by `prepare-batch`.

## Worker checkpoints

Workers checkpoint after each completed unit:

```bash
python skills/hardened-longrun-subagent-harness/scripts/risk_controls.py heartbeat \
  --run /path/to/run \
  --task-id lesson-01 \
  --attempt 1 \
  --worker-id student-01 \
  --checkpoint-file /path/to/checkpoint.json
```

If the worker lease expires, recover the mission:

```bash
python skills/hardened-longrun-subagent-harness/scripts/risk_controls.py recover \
  --run /path/to/run
```

## Side-effect reconciliation

An interrupted external action cannot be retried until a receipt is verified:

```bash
python skills/hardened-longrun-subagent-harness/scripts/risk_controls.py reconcile-side-effect \
  --run /path/to/run \
  --task-id publish-task \
  --attempt 1 \
  --receipt-file /path/to/receipt.json
```

## Rate-limit response

```bash
python skills/hardened-longrun-subagent-harness/scripts/risk_controls.py record-rate-limit \
  --run /path/to/run \
  --provider cheap-model \
  --retry-after-seconds 60
```

## Verification

```bash
python skills/hardened-longrun-subagent-harness/tests/smoke_test.py
python skills/hardened-longrun-subagent-harness/tests/risk_controls_test.py
```

The risk suite deliberately expires a worker lease, resumes from a checkpoint, blocks an uncertain external side
effect, validates a reconciliation receipt, rejects an incomplete context packet, enforces provider/global concurrency,
and proves cooldown backpressure.

## Worker result rule

A worker's chat summary is not canonical. Each worker atomically writes:

```text
attempts/<task_id>/attempt-<NNN>/result.json
```

The result must match `schemas/worker-result.schema.json` and include identity, status, findings, evidence, artifacts,
validations, side effects, usage, idempotency key, and content hash.

## Finalization

Before finalization create:

```text
outputs/synthesis.json
outputs/completion-criteria.json
reviews/spec-review.json
reviews/quality-review.json
reviews/security-review.json  # when applicable
```

Then run:

```bash
python skills/hardened-longrun-subagent-harness/scripts/mission_state.py finalize --run /path/to/run
python skills/hardened-longrun-subagent-harness/scripts/export_mission.py --run /path/to/run
```

Outputs:

```text
outputs/final-report.json
outputs/export-manifest.json
outputs/<mission_id>.zip
```

## Safety defaults

- Three concurrent children by default; hard maximum eight
- Provider-specific caps and cooldowns
- No nested delegation by default
- No blind retries after uncertain effects
- No worker writes to shared state
- No durable-memory insertion without approval
- No completion without independent review
- No secret values in mission JSON
- No unbounded loops
