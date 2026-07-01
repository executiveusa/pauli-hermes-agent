---
name: github-webhook-trigger
description: Verify and normalize GitHub webhook payloads for deployment automation. Use when a GitHub push, deployment_status, or workflow_run event arrives.
---

# GitHub Webhook Trigger

## Use when

- GitHub sends a webhook.
- Another agent supplies a GitHub event payload.
- A deployment cycle must be started from a commit notification.

## Workflow

1. Verify `X-Hub-Signature-256` if `GITHUB_WEBHOOK_SECRET` is set.
2. Extract event, repo, branch, SHA, sender, and delivery ID.
3. Reject unsupported branches or unsafe events.
4. Write `runs/<run-id>/event.json`.
5. Call Stage 02.

## Script

```bash
node scripts/server.mjs
```

## Output

JSON only from endpoints. Run artifacts on disk.
