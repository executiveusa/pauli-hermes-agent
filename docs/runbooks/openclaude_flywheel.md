# OpenClaude Flywheel Runbook

## Overview

The Flywheel is the task-routing layer between Hermes (orchestrator) and the OpenClaude coding worker. It manages beads — structured task units — through their lifecycle: creation, validation, dispatch, result capture, and status update.

This runbook explains how beads flow through the dispatcher and how to debug issues at each stage.

---

## Bead Lifecycle

```
Hermes creates bead spec (dict)
         |
         v
BeadSpec.from_dict()  ← validates required fields, assigns bead_id if absent
         |
         v
Gate 1: Deny-list check
  - task_type in denied_task_types → DeniedTaskTypeError (bead never dispatched)
         |
         v
Gate 2: Approval check
  - requires_actions intersects requires_approval_for → prompt user
  - User denies → ApprovalRequiredError
  - User approves → continue
         |
         v
Binary discovery
  - vendor/openclaude/bin/openclaude  (preferred)
  - vendor/openclaude/node_modules/.bin/openclaude
  - system PATH
  - not found → WorkerNotInstalledError
         |
         v
Model selection (cheapest available)
  - Ollama → OpenRouter free → Groq → DeepSeek → OpenAI
         |
         v
Interface selection
  - gRPC port 50051 open? → _dispatch_grpc()
  - else                  → _dispatch_cli()
         |
         v
Subprocess invocation
  - openclaude --print "<prompt>"  (+ optional --grpc-endpoint host:port)
  - timeout enforced
  - stdout/stderr captured
         |
         v
Output parsing
  - _parse_files_changed()   ← extracts "Modified: X" / "Created: X" lines
  - _parse_test_results()    ← extracts pytest / jest pass/fail counts
  - _redact_secrets()        ← strips API key patterns before returning
         |
         v
DispatchResult returned to Hermes
  - {status, bead_id, model_used, provider, logs, files_changed, test_results, duration_seconds}
```

---

## Bead Spec Format

Minimum required fields:

```json
{
  "bead_id": "bead_abc123",
  "task_type": "refactor",
  "description": "Extract auth logic from gateway/run.py into gateway/auth.py"
}
```

Optional fields:

```json
{
  "repo_path": "/absolute/path/to/repo",
  "allowed_files": ["gateway/run.py", "gateway/auth.py", "tests/gateway/"],
  "max_tokens": 8192,
  "timeout_seconds": 300,
  "metadata": {
    "requires_actions": ["git_push"]
  }
}
```

---

## Running the Dispatcher Manually

```python
from pauli.flywheel.dispatchers.openclaude_dispatcher import OpenClaudeDispatcher

dispatcher = OpenClaudeDispatcher()
result = dispatcher.dispatch({
    "bead_id": "bead_manual001",
    "task_type": "docs",
    "description": "Write docstrings for all public functions in tools/kanban_tools.py",
    "repo_path": "/path/to/hermes-agent",
    "allowed_files": ["tools/kanban_tools.py"],
})
print(result)
```

---

## Allowed vs. Denied Task Types

Defined in `config/pauli_worker_registry.yaml`:

| Task Type | Allowed |
|---|---|
| `refactor` | Yes |
| `test_repair` | Yes |
| `frontend` | Yes |
| `docs` | Yes |
| `mcp` | Yes |
| `codegen` | Yes |
| `production_deploy` | **No — DeniedTaskTypeError** |
| `secret_rotation` | **No — DeniedTaskTypeError** |
| `destructive_git` | **No — DeniedTaskTypeError** |

---

## Approval-Required Actions

If a bead's `metadata.requires_actions` list contains any of `[git_push, deploy, delete_files, migrate_database]`, the dispatcher checks for approval before proceeding.

Wiring in approval:

```python
def my_approval_callback(bead, actions):
    response = input(f"Approve {actions} for bead {bead.bead_id}? [y/N] ")
    return response.lower() == "y"

dispatcher = OpenClaudeDispatcher(
    require_approval_callback=my_approval_callback
)
```

If no callback is set and approval is required, `ApprovalRequiredError` is raised and the bead remains undispatched.

---

## Interface Selection: gRPC vs. CLI

| Condition | Interface Used |
|---|---|
| Worker in headless mode, port 50051 open | gRPC (lower latency) |
| Port 50051 closed or worker not started | CLI subprocess (fallback) |

The dispatcher selects automatically. No configuration change is needed.

To force CLI mode even when gRPC is available, start the dispatcher with a dummy port override or simply don't start the worker in headless mode.

---

## Timeout and Failure Handling

Default timeout: 600 seconds (10 minutes) per bead. Override per bead:

```json
{"timeout_seconds": 120, "task_type": "docs", "description": "..."}
```

If a bead times out:
- The subprocess is killed.
- `DispatchResult.status` = `"failed"`
- `DispatchResult.error` = `"Worker timed out after Xs"`
- `DispatchResult.logs` contains whatever was captured before the timeout.

---

## Debugging

### Enable debug logging

```python
import logging
logging.basicConfig(level=logging.DEBUG)
```

The dispatcher logs at `DEBUG` level:
- `CLI dispatch: cmd=... workdir=...`
- `gRPC dispatch: cmd=... workdir=...`

And at `INFO` level:
- `Dispatched bead <id>: status=<status>, provider=<p>, model=<m>, duration=<t>s`

### Check what model was selected

```python
from pauli.flywheel.dispatchers.openclaude_dispatcher import select_model, get_worker_config, load_worker_registry
r = load_worker_registry()
cfg = get_worker_config(r)
provider, model = select_model(cfg)
print(provider, model)
```

### Inspect a bead spec before dispatch

```python
bead = BeadSpec.from_dict(my_bead_dict)
print(bead)
```

Prints the resolved bead with all defaults filled in.
