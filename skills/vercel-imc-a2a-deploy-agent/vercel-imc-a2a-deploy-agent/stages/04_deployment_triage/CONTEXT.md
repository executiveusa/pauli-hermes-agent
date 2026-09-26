# Stage 04_deployment_triage: Deployment Triage


## Inputs

- `runs/<run-id>/project-map.json`
- Recent deployment metadata
- Build logs if available
- `guardrails/no-code-boundary.md`

## Process

1. Identify current production deployment state.
2. Inspect failed/errored deployments.
3. Classify blocker as platform/config/no-code-safe/source-code-required/unknown.
4. Produce a triage brief.

## Outputs

- `runs/<run-id>/triage.json`
- `runs/<run-id>/triage.md`

## Gate

Continue only when the next action is no-code-safe.

