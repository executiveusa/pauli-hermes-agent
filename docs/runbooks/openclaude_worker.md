# OpenClaude Worker Runbook

## What Is It?

OpenClaude is an open-source coding agent CLI that supports multiple AI providers (Ollama, OpenRouter, Groq, DeepSeek, OpenAI, and others). In the Pauli stack, it runs as an isolated **leaf worker** inside the Hermes + Flywheel architecture. Hermes assigns coding tasks; OpenClaude executes them.

OpenClaude is **not** an orchestrator. It cannot spawn subagents, access secrets directly, deploy to production, or make architectural decisions. Those responsibilities remain with Hermes.

---

## Role in the Stack

```
User → Hermes (orchestrator)
           └── Flywheel (bead queue + dispatcher)
                    └── OpenClaude worker (coding worker)
                              ├── Reads: bead spec (task description, repo path)
                              ├── Writes: code files in the target repo
                              └── Returns: structured result (files changed, test results)
```

---

## Installation

### Prerequisites

- Node.js >= 22 (`node --version` to check; install via `nvm install 22` if needed)
- npm (comes with Node.js)
- git

### Step 1: Clone the worker

```bash
scripts/pauli/openclaude/install.sh
```

This clones `https://github.com/Gitlawb/openclaude.git` into `vendor/openclaude/`, runs `npm install`, and verifies the binary.

To force a re-clone:

```bash
scripts/pauli/openclaude/install.sh --force
```

### Step 2: Generate the config file

```bash
# With Infisical (recommended):
infisical run -- scripts/pauli/openclaude/generate-config.sh

# With environment variables already set:
OPENROUTER_API_KEY=sk-or-... scripts/pauli/openclaude/generate-config.sh

# Dry run (preview without writing):
scripts/pauli/openclaude/generate-config.sh --dry-run
```

This writes `~/.openclaude.json` with permissions `600`. See `docs/runbooks/openclaude_secrets.md` for the full secrets guide.

### Step 3: Verify the installation

```bash
scripts/pauli/openclaude/healthcheck.sh
```

Exit code 0 = healthy. Other codes:
- 1 = binary missing (re-run install.sh)
- 2 = binary found but worker not running (run start.sh)
- 3 = worker running but gRPC port not responding

---

## Activating the Worker

### Persistent mode (recommended for high-volume use)

```bash
scripts/pauli/openclaude/start.sh --mode grpc
```

Starts OpenClaude as a background gRPC server on port 50051. The Flywheel dispatcher will prefer this path for subsequent dispatches (lower latency, no startup overhead per bead).

### Per-task mode (suitable for low-volume use)

Skip `start.sh`. The Flywheel dispatcher will automatically use CLI mode — it spawns a fresh `openclaude` subprocess for each bead. No persistent process required.

---

## Using the Worker via the Hermes Skill

Once installed and configured, activate the OpenClaude worker from Hermes by saying:

- "use OpenClaude to refactor gateway/run.py"
- "assign this test repair to the coding worker"
- "send this feature to OpenClaude"
- "background coding task: implement the auth module"

Hermes will invoke the `openclaude-worker` skill, which builds a bead spec and calls the Flywheel dispatcher.

---

## Stopping the Worker

```bash
# Read PID and kill
PID=$(cat ~/.hermes/run/openclaude-worker.pid 2>/dev/null)
if [ -n "$PID" ] && kill -0 "$PID" 2>/dev/null; then
  kill "$PID" && echo "Stopped PID $PID"
fi
rm -f ~/.hermes/run/openclaude-worker.pid
```

Or simply kill the process by name:

```bash
pkill -f "openclaude.*headless"
```

---

## Logs

Worker logs are written to:

```
~/.hermes/logs/openclaude-worker.log
```

View in real time:

```bash
tail -f ~/.hermes/logs/openclaude-worker.log
```

---

## Troubleshooting

### "binary not found"

Run `scripts/pauli/openclaude/install.sh`. If Node.js is below version 22, upgrade it first.

### "Worker exited immediately"

Check the log: `cat ~/.hermes/logs/openclaude-worker.log`. Common causes:
- `~/.openclaude.json` missing or malformed. Re-run `generate-config.sh`.
- Port 50051 already in use. Use `OPENCLAUDE_PORT=50052 start.sh`.

### "gRPC port not responding"

The worker may still be starting (give it 5–10 seconds) or crashed. Check the log. The dispatcher will fall back to CLI mode automatically.

### Task type denied

Only these task types are allowed: `refactor`, `test_repair`, `frontend`, `docs`, `mcp`, `codegen`. Ensure the bead's `task_type` is in the allowed list.

---

## Updating

To pull the latest OpenClaude:

```bash
scripts/pauli/openclaude/install.sh
```

The install script does a `git pull` if the vendor directory already exists.
