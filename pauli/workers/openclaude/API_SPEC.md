# OpenClaude Worker — REST API Specification

## Base URL

All endpoints are served under `/api/workers/openclaude` by the Hermes FastAPI server (`hermes_cli/web_server.py`).

## Authentication

All endpoints require the session token issued at dashboard startup. Pass it as:
- Header: `Authorization: Bearer <token>`
- Query param: `?token=<token>` (WebSocket/SSE only — browsers cannot set headers on EventSource)

---

## Endpoints

### GET /api/workers

List all registered workers from `config/pauli_worker_registry.yaml`.

**Response 200:**
```json
{
  "workers": [
    {
      "name": "openclaude",
      "type": "coding_worker",
      "enabled": true,
      "status": "healthy"
    }
  ]
}
```

**Error cases:**
- 500: Registry file missing or malformed YAML.

---

### GET /api/workers/openclaude/status

Return the current status of the OpenClaude worker.

**Response 200:**
```json
{
  "name": "openclaude",
  "status": "healthy",
  "pid": 12345,
  "grpc_port": 50051,
  "grpc_port_open": true,
  "binary_path": "/path/to/vendor/openclaude/bin/openclaude",
  "version": "openclaude 0.9.2",
  "provider": "openrouter",
  "model": "meta-llama/llama-3.1-8b-instruct:free",
  "active_beads": 0,
  "max_parallel_beads": 2
}
```

**Status values:** `"healthy"` | `"starting"` | `"offline"` | `"error"` | `"cli-stub"`

**Error cases:**
- 500: Registry unreadable.

---

### POST /api/workers/openclaude/start

Start the OpenClaude worker in gRPC mode.

**Request body:** (optional)
```json
{
  "mode": "grpc",
  "port": 50051
}
```

**Response 200:**
```json
{
  "started": true,
  "pid": 12345,
  "mode": "grpc"
}
```

**Response 409 (already running):**
```json
{
  "started": false,
  "reason": "Worker already running at PID 12345"
}
```

**Error cases:**
- 500: install.sh not found, Node.js version too low, binary missing.
- 503: Worker started but failed healthcheck within 30 seconds.

---

### POST /api/workers/openclaude/stop

Stop the running worker process.

**Request body:** empty

**Response 200:**
```json
{
  "stopped": true,
  "pid": 12345
}
```

**Response 404:**
```json
{
  "stopped": false,
  "reason": "Worker is not running"
}
```

---

### POST /api/workers/openclaude/restart

Stop then start the worker. Equivalent to calling stop then start sequentially.

**Request body:** (optional)
```json
{
  "mode": "grpc",
  "port": 50051
}
```

**Response 200:**
```json
{
  "restarted": true,
  "pid": 67890
}
```

**Error cases:**
- 503: Failed to restart within timeout.

---

### POST /api/workers/openclaude/healthcheck

Trigger an immediate health check and return the result.

**Request body:** empty

**Response 200:**
```json
{
  "binary_found": true,
  "binary_path": "/path/to/openclaude",
  "version": "openclaude 0.9.2",
  "grpc_port_open": true,
  "error": null
}
```

**Response 200 (unhealthy):**
```json
{
  "binary_found": false,
  "binary_path": null,
  "version": null,
  "grpc_port_open": false,
  "error": "openclaude binary not found. Run: scripts/pauli/openclaude/install.sh"
}
```

---

### POST /api/workers/openclaude/assign-bead

Dispatch a new bead to the worker.

**Request body:**
```json
{
  "bead_id": "bead_001",
  "task_type": "refactor",
  "description": "Extract auth logic from gateway/run.py into gateway/auth.py",
  "repo_path": "/path/to/repo",
  "allowed_files": ["gateway/run.py", "gateway/auth.py"],
  "max_tokens": 8192,
  "timeout_seconds": 300
}
```

`bead_id` is optional — auto-generated if absent. All other fields as defined in `BeadSpec`.

**Response 200 (dispatched):**
```json
{
  "bead_id": "bead_001",
  "status": "running",
  "provider": "openrouter",
  "model_used": "meta-llama/llama-3.1-8b-instruct:free"
}
```

**Response 422 (denied task type):**
```json
{
  "error": "DeniedTaskTypeError",
  "message": "Task type 'production_deploy' is in the deny-list.",
  "denied_types": ["production_deploy", "secret_rotation", "destructive_git"]
}
```

**Response 503 (worker not installed):**
```json
{
  "error": "WorkerNotInstalledError",
  "message": "openclaude binary not found. Run: scripts/pauli/openclaude/install.sh"
}
```

---

### POST /api/workers/openclaude/chat

Send a message to the worker and stream the response.

**Request body:**
```json
{
  "message": "Refactor the login function to use the new AuthManager class",
  "bead_id": "bead_001",
  "model_override": "llama-3.3-70b-versatile"
}
```

`bead_id` optional — creates a new `codegen` bead if absent. `model_override` optional.

**Response:** Server-sent events (SSE), `Content-Type: text/event-stream`

```
data: {"type": "token", "content": "I'll start by"}
data: {"type": "token", "content": " reading the login"}
data: {"type": "tool", "name": "read_file", "path": "auth.py"}
data: {"type": "token", "content": " function..."}
data: {"type": "done", "files_changed": ["auth.py"], "test_results": {"passed": 5, "failed": 0}}
```

**Error cases:**
- 422: Denied task type.
- 503: Worker not installed.
- 500: Subprocess failure.

---

### GET /api/workers/openclaude/logs

Stream recent log lines from the worker.

**Query parameters:**
- `lines` (int, default 100): Number of recent lines to return.
- `follow` (bool, default false): If true, keep the connection open and stream new lines.

**Response:** `Content-Type: text/event-stream` when `follow=true`, else `text/plain`.

```
data: [2026-05-08 12:01:23] Worker started (PID 12345)
data: [2026-05-08 12:01:45] Dispatching bead_001: refactor
data: [2026-05-08 12:02:19] bead_001: success (34.2s, 2 files changed)
```

---

### GET /api/workers/openclaude/tasks

List recent beads and their status.

**Query parameters:**
- `limit` (int, default 20): Max beads to return.
- `status` (str, optional): Filter by status (`running`, `success`, `failed`, `blocked`).

**Response 200:**
```json
{
  "tasks": [
    {
      "bead_id": "bead_001",
      "task_type": "refactor",
      "status": "success",
      "provider": "openrouter",
      "model_used": "meta-llama/llama-3.1-8b-instruct:free",
      "duration_seconds": 34.2,
      "files_changed": ["gateway/auth.py", "tests/gateway/test_auth.py"],
      "test_results": {"passed": 8, "failed": 0},
      "error": null,
      "started_at": "2026-05-08T12:01:45Z",
      "finished_at": "2026-05-08T12:02:19Z"
    }
  ],
  "total": 1
}
```

---

### GET /api/workers/openclaude/changed-files

List all files changed across all recent beads.

**Query parameters:**
- `bead_id` (str, optional): Filter to a specific bead.
- `limit` (int, default 50): Max files to return.

**Response 200:**
```json
{
  "files": [
    {
      "path": "gateway/auth.py",
      "bead_id": "bead_001",
      "operation": "Modified",
      "changed_at": "2026-05-08T12:02:19Z"
    }
  ]
}
```
