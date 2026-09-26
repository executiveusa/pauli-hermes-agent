---
name: a2a-router
description: Publish agent capabilities and accept Agent2Agent task requests through the local HTTP receiver.
---

# A2A Router

## Use when

- Another agent needs to discover or call this agent.
- The local server needs to serve an agent card.
- A task arrives through `/a2a/tasks`.

## Script

```bash
node scripts/server.mjs
```

## Endpoints

- `GET /.well-known/agent-card.json`
- `POST /a2a/tasks`
- `POST /webhooks/github`
- `GET /healthz`
