# A2A Protocol Surface

## Discovery

Serve this file as JSON at:

```text
/.well-known/agent-card.json
```

Local copy:

```text
a2a/.well-known/agent-card.json
```

## Task endpoint

Local server:

```text
POST /a2a/tasks
```

Accepted payload shape:

```json
{
  "id": "external-task-id",
  "method": "tasks/send",
  "params": {
    "skill": "vercel-visibility-sweep",
    "repo": "owner/repo",
    "branch": "main",
    "dryRun": true,
    "message": "Deploy and verify production main without code changes."
  }
}
```

## Response shape

```json
{
  "jsonrpc": "2.0",
  "id": "external-task-id",
  "result": {
    "taskId": "...",
    "state": "accepted",
    "runDir": "runs/..."
  }
}
```

## Authentication

Mutating endpoints require:

```text
Authorization: Bearer $A2A_SHARED_SECRET
```

## State model

Task states:

- `accepted`
- `working`
- `input-required`
- `completed`
- `failed`
- `cancelled`

The filesystem is the durable state store. Each A2A request creates or updates a run directory.
