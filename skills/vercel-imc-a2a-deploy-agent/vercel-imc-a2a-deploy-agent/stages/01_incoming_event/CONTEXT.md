# Stage 01_incoming_event: Incoming Event


## Inputs

- GitHub webhook payload or A2A task payload
- `hooks/github-webhook-contract.md`
- `guardrails/production-safety.md`
- `config/production-policy.json`

## Process

1. Verify auth/signature.
2. Extract event type, repo, branch, commit SHA, sender, and delivery ID.
3. Reject unsafe events.
4. Write normalized event JSON.

## Outputs

- `runs/<run-id>/event.json`
- `stages/01_incoming_event/output/event-normalized.json`

## Gate

Proceed only if branch and event source are approved.

