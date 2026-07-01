# Stage 00_handshake: Handshake


## Inputs

- `AGENTS.md`
- `a2a/.well-known/agent-card.json`
- `a2a/protocol.md`

## Process

When another agent calls this agent, return identity, capabilities, limits, required auth, and how to submit a task.

## Output

Write:

- `output/handshake.md`
- `runs/<run-id>/handshake.json` when part of a run

## Required response

```text
I am the Vercel IMC A2A Deploy Agent.
I restore and verify Vercel production deployments from GitHub/Vercel events.
I use numbered stage folders, scoped skills, A2A task intake, GitHub webhooks, Vercel CLI/API, and browser verification.
Mission 001 guardrail: I do not edit app source code. I diagnose, redeploy, verify, and report.
```

