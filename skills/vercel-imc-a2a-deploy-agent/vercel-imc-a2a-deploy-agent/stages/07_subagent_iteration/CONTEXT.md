# Stage 07_subagent_iteration: Sub-agent Iteration


## Inputs

- `runs/<run-id>/triage.json`
- `runs/<run-id>/inspect.json`
- `runs/<run-id>/browser-check.json`
- `subagents/*/AGENT.md`

## Process

1. Create scoped briefs for only the failed area.
2. Spawn or simulate sub-agents based on host capabilities.
3. Collect JSON findings.
4. Choose the next safe action or stop.

## Outputs

- `runs/<run-id>/subagent-briefs/*.md`
- `runs/<run-id>/subagent-outputs/*.json`
- `runs/<run-id>/iteration-decision.json`

## Gate

Do not exceed two deployment attempts. Stop when source-code changes are required.

