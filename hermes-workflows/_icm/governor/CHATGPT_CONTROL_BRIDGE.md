# ChatGPT → Hermes Control Bridge

## Goal
Allow the owner to supervise Hermes from ChatGPT without manually copying tasks into Hermes.

## V1 — GitHub control bus (recommended now)
Use GitHub as the durable command and receipt channel because ChatGPT already has authenticated GitHub actions and Hermes already has webhook + GitHub tooling.

Flow:
1. Owner tells ChatGPT the desired outcome.
2. ChatGPT creates a GitHub issue in the Hermes repo (or dedicated private control repo) with label `hermes-dispatch` and a structured task contract.
3. GitHub sends an `issues` webhook to Hermes Gateway.
4. Hermes validates the webhook signature, loads the Governor workflow, creates a Kanban root task + specialist child tasks, and runs them unattended.
5. Hermes posts milestone/blocked/final receipts back to the GitHub issue and optionally Telegram.
6. ChatGPT can read the issue/comments and summarize state for the owner.
7. Owner only intervenes for explicit approval/taste/risk gates.

This is deliberately not a raw unauthenticated HTTP command endpoint.

## Hermes webhook route
Use the built-in Hermes webhook adapter. Keep the HMAC secret outside Git.

Example `~/.hermes/config.yaml` shape:
```yaml
platforms:
  webhook:
    enabled: true
    extra:
      port: 8644
      secret: "${WEBHOOK_SECRET}"
      routes:
        github-hermes-dispatch:
          events: ["issues", "issue_comment"]
          secret: "${GITHUB_HERMES_WEBHOOK_SECRET}"
          prompt: |
            You are the Pauli Hermes Governor.
            Treat this GitHub event as a durable supervisory command.
            Repository: {repository.full_name}
            Action: {action}
            Issue: {issue.number}
            Title: {issue.title}
            Body: {issue.body}
            Comment: {comment.body}

            Only execute if the issue carries the `hermes-dispatch` label or the comment is an allowed governor command on an already-dispatched issue.
            Load the relevant ICM workflow, create/reconcile Kanban work, execute within authority, and return a concise evidence-based status report.
          skills: ["icm-organizer"]
          deliver: "github_comment"
          deliver_extra:
            repo: "{repository.full_name}"
            pr_number: "{issue.number}"
```

Note: validate the exact GitHub-issue delivery field supported by the installed Hermes version before production. If `github_comment` is PR-only in the runtime, deliver to Telegram/log and use a GitHub tool/`gh` for issue comments from the agent. Do not guess.

## Command issue schema
Every ChatGPT-created dispatch issue should contain:

```md
MODE: brownfield | greenfield
OUTCOME: measurable result
TARGET: customer/user/system
CONSTRAINTS: what must not change
PROTECTED ASSETS: source-of-truth assets
PROOF: evidence required
BUDGET: provider/tool budget threshold
DEADLINE: if any
WORKFLOW: named ICM workflow
APPROVAL GATES: public/financial/taste/legal
ROLLBACK: expected restoration path
```

## Allowed owner commands in comments
- `STATUS` — report current Kanban graph + blockers + cost.
- `PAUSE` — stop new side effects; active safe reads may finish.
- `RESUME` — continue from durable state.
- `APPROVE <artifact/task>` — satisfy one explicit approval gate.
- `REJECT <artifact/task>: <reason>` — create bounded revision work.
- `CANCEL` — block remaining work and preserve receipts/outputs.
- `ROLLBACK <change>` — invoke documented rollback if available.

Never interpret casual discussion as approval.

## Security requirements
- HMAC V2/replay-resistant webhook signature when using generic webhook senders.
- GitHub webhook secret per route.
- Private control repo preferred for sensitive work.
- Allowlist repository and label.
- Rate limit.
- Idempotency/delivery IDs.
- No secrets in issue bodies/comments.
- Provider/public actions remain governed by task approval state.

## V2 — Hermes as an MCP server
Hermes can run `hermes mcp serve` and expose conversation/task capabilities to another MCP client. Use this when ChatGPT has a trusted MCP/App connector to the deployed Hermes endpoint. This is cleaner than GitHub for low-latency interactive control, but should only replace V1 after authentication, least-privilege tool exposure, audit logging, and owner-controlled deployment are verified.

## Why GitHub first
- already authenticated from ChatGPT
- durable, inspectable commands
- comments are receipts
- easy owner supervision
- natural idempotency/task IDs
- no new public command surface required
- rollback/history is obvious

The GitHub issue becomes the supervisory envelope; Hermes Kanban becomes the execution graph.
