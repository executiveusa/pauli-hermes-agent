# OpenClaude Worker

## Role

OpenClaude is the **coding worker** in the Pauli + Hermes architecture. It is a leaf node — it receives coding task beads from the Flywheel dispatcher, executes them, and returns structured results. It is explicitly **not an orchestrator** and cannot spawn subagents, access production systems, or manage secrets.

```
Hermes (orchestrator)
  └── Flywheel dispatcher
        └── OpenClaude worker  ← YOU ARE HERE
              ├── Executes coding tasks
              ├── Reads/writes code files
              └── Reports changed files + test results
```

## Supported Task Types

| Task Type | Description |
|---|---|
| `refactor` | Restructure or clean up existing code without changing behavior |
| `test_repair` | Fix failing tests, update test fixtures, improve coverage |
| `frontend` | UI components, styles, client-side JavaScript/TypeScript |
| `docs` | Docstrings, README files, API documentation, comments |
| `mcp` | Scaffold or extend MCP (Model Context Protocol) servers |
| `codegen` | Generate boilerplate, type stubs, migration files, schemas |

## Denied Task Types

The following are always blocked — the dispatcher raises `DeniedTaskTypeError` before invoking OpenClaude:

- `production_deploy`
- `secret_rotation`
- `destructive_git`

## Installation

```bash
# Install the worker binary
scripts/pauli/openclaude/install.sh

# Generate provider config (never commits secrets)
scripts/pauli/openclaude/generate-config.sh

# Verify
scripts/pauli/openclaude/healthcheck.sh
```

Requirements:
- Node.js >= 22
- npm
- git
- At least one provider credential (see `docs/runbooks/openclaude_secrets.md`)

## Starting the Worker

### gRPC mode (preferred for sustained use)

```bash
scripts/pauli/openclaude/start.sh --mode grpc
```

Starts a persistent gRPC server on port 50051. The Flywheel dispatcher will prefer this interface.

### Per-task CLI mode (no persistent process)

Simply don't run `start.sh`. The dispatcher spawns a fresh subprocess for each bead. Suitable for infrequent tasks.

## Dispatching a Task

Via Python:

```python
from pauli.flywheel.dispatchers.openclaude_dispatcher import OpenClaudeDispatcher

dispatcher = OpenClaudeDispatcher()
result = dispatcher.dispatch({
    "bead_id": "bead_001",
    "task_type": "refactor",
    "description": "Extract the auth middleware from gateway/run.py into a separate module.",
    "repo_path": "/path/to/repo",
    "allowed_files": ["gateway/run.py", "gateway/auth.py", "tests/gateway/"],
})
print(result)
```

Via Hermes skill:

```
> assign bead to OpenClaude: refactor the authentication module
```

## Output Format

Every dispatch returns:

```json
{
  "status": "success",
  "bead_id": "bead_001",
  "model_used": "meta-llama/llama-3.1-8b-instruct:free",
  "provider": "openrouter",
  "logs": "...(redacted)",
  "files_changed": ["gateway/auth.py", "tests/gateway/test_auth.py"],
  "test_results": {"passed": 8, "failed": 0},
  "duration_seconds": 34.2,
  "error": null
}
```

## API Reference

See `pauli/workers/openclaude/API_SPEC.md` for the full REST API specification.

## More Documentation

- `docs/runbooks/openclaude_worker.md` — full install and usage guide
- `docs/runbooks/openclaude_flywheel.md` — bead lifecycle and dispatcher internals
- `docs/runbooks/openclaude_secrets.md` — secret management
- `docs/runbooks/openclaude_models.md` — provider priority and model selection
- `docs/analysis/openclaude_integration_report.md` — architecture analysis
