---
title: "Openclaude Worker — Dispatch coding tasks to the OpenClaude isolated worker via the Pauli Flywheel"
sidebar_label: "Openclaude Worker"
description: "Dispatch coding tasks to the OpenClaude isolated worker via the Pauli Flywheel"
---

{/* This page is auto-generated from the skill's SKILL.md by website/scripts/generate-skill-docs.py. Edit the source SKILL.md, not this page. */}

# Openclaude Worker

Dispatch coding tasks to the OpenClaude isolated worker via the Pauli Flywheel. Handles refactoring, test repair, feature implementation, documentation, MCP scaffolding, and code generation. OpenClaude is a leaf worker — Hermes remains the orchestrator.

## Skill metadata

| | |
|---|---|
| Source | Bundled (installed by default) |
| Path | `skills/pauli/openclaude-worker` |
| Version | `1.0.0` |
| Author | pauli |
| License | MIT |
| Platforms | linux, macos, windows |
| Tags | `worker`, `coding`, `flywheel`, `openclaude`, `refactor`, `codegen`, `bead`, `background` |

## Reference: full SKILL.md

:::info
The following is the complete skill definition that Hermes loads when this skill is triggered. This is what the agent sees as instructions when the skill is active.
:::

# OpenClaude Worker Skill

## Overview

This skill activates the OpenClaude coding worker integration inside the Hermes + Flywheel stack. When invoked, Hermes acts as the orchestrator and dispatches coding subtasks to OpenClaude — an isolated, multi-provider coding agent — via the Flywheel bead dispatcher.

OpenClaude is a **leaf worker**. It cannot spawn subagents, access secrets directly, deploy to production, or perform destructive git operations. All of those actions remain under Hermes's control with human approval gates where required.

---

## Triggers

Activate this skill when the user says any of:

- "use OpenClaude"
- "send this to OpenClaude"
- "run coding worker"
- "assign bead to OpenClaude"
- "refactor this repo"
- "fix tests"
- "implement this feature"
- "cheap model worker"
- "background coding task"
- "dispatch to worker"
- "have the worker do this"

---

## When to Use

Use this skill when the task is:

- A well-defined coding subtask (refactoring a module, fixing failing tests, implementing a described feature, writing docs for a function, scaffolding an MCP server, generating boilerplate code).
- Repetitive or mechanical coding work where using a cheaper model via OpenRouter, Groq, or Ollama saves cost compared to a full Hermes session.
- A background task that should not block the current Hermes conversation.
- A task that fits one of the allowed task types: `refactor`, `test_repair`, `frontend`, `docs`, `mcp`, `codegen`.

---

## When NOT to Use

Do NOT activate this skill when the task involves:

- Deploying to production (`production_deploy` — blocked).
- Rotating, generating, or accessing API keys or secrets (`secret_rotation` — blocked).
- Force-pushing branches, deleting branches, or other destructive git operations (`destructive_git` — blocked).
- Any action requiring access to the production database without a staging replica.
- Tasks that require live orchestration decisions across multiple systems (keep those in Hermes).
- Tasks where the output must feed back into the current Hermes turn in real time (use `delegate_task` instead).

---

## Required Environment Variables

At least one provider credential must be set. Prefer the cheapest available:

| Variable | Provider | Cost |
|---|---|---|
| `OLLAMA_HOST` | Ollama (local) | Free |
| `OPENROUTER_API_KEY` | OpenRouter free tier | Free/cheap |
| `GROQ_API_KEY` | Groq | Cheap |
| `DEEPSEEK_API_KEY` | DeepSeek | Cheap |
| `OPENAI_API_KEY` | OpenAI | Paid |

---

## Required Tools

- `terminal` (bash) — to run install/start/healthcheck scripts and the dispatcher.
- Python 3.11+ — for the Flywheel dispatcher (`pauli/flywheel/dispatchers/openclaude_dispatcher.py`).
- Node.js >= 22 — for the OpenClaude worker binary.
- `git` — to clone/update `vendor/openclaude`.

---

## Safety Gates

The dispatcher enforces these safety gates at runtime:

1. **Denied task types**: Tasks with `task_type` in `[production_deploy, secret_rotation, destructive_git]` raise `DeniedTaskTypeError` immediately — the task is never dispatched.

2. **Approval-required actions**: Tasks that require `git_push`, `deploy`, `delete_files`, or `migrate_database` MUST receive explicit human approval via Hermes before the dispatcher invokes OpenClaude. Hermes presents a confirmation prompt. Automation cannot bypass this.

3. **No secrets in bead spec**: The bead JSON never contains raw API keys. Provider credentials are passed only via environment variables to the subprocess. The dispatcher strips any key-shaped strings from the bead before logging.

4. **Timeout**: The dispatcher enforces a hard timeout (default 10 minutes) per bead. If OpenClaude hangs, the subprocess is killed and the bead is marked `failed`.

5. **Sandboxed workdir**: Each bead runs in an isolated scratch directory. OpenClaude cannot write outside its assigned working directory unless the bead spec explicitly grants broader access.

---

## Workflow

### Step 1 — Check installation

```bash
scripts/pauli/openclaude/healthcheck.sh
```

If exit code is 1 (binary missing), run the installer:

```bash
scripts/pauli/openclaude/install.sh
scripts/pauli/openclaude/generate-config.sh
```

### Step 2 — Start worker (gRPC mode, optional)

For high-throughput use, start the worker once:

```bash
scripts/pauli/openclaude/start.sh --mode grpc
```

For low-volume use, skip this step. The dispatcher falls back to CLI mode (one subprocess per bead).

### Step 3 — Assemble a bead spec

A bead is a structured task unit. Minimum required fields:

```json
{
  "bead_id": "bead_<uuid>",
  "task_type": "refactor",
  "description": "Extract the authentication logic from gateway/run.py into gateway/auth.py with full test coverage.",
  "repo_path": "/path/to/target/repo",
  "allowed_files": ["gateway/run.py", "gateway/auth.py", "tests/gateway/"],
  "max_tokens": 8192
}
```

### Step 4 — Dispatch via Flywheel

```python
from pauli.flywheel.dispatchers.openclaude_dispatcher import OpenClaudeDispatcher

dispatcher = OpenClaudeDispatcher()
result = dispatcher.dispatch(bead)
print(result)
```

Or from the CLI:

```bash
python -c "
from pauli.flywheel.dispatchers.openclaude_dispatcher import OpenClaudeDispatcher
import json
bead = json.loads(open('bead.json').read())
d = OpenClaudeDispatcher()
print(json.dumps(d.dispatch(bead), indent=2))
"
```

### Step 5 — Review result

The dispatcher returns:

```json
{
  "status": "success",
  "bead_id": "bead_abc123",
  "model_used": "meta-llama/llama-3.1-8b-instruct:free",
  "provider": "openrouter",
  "logs": "...",
  "files_changed": ["gateway/auth.py", "tests/gateway/test_auth.py"],
  "test_results": {"passed": 12, "failed": 0},
  "duration_seconds": 47.3
}
```

Hermes surfaces this result to the user and updates the bead status.

---

## Output Contract

Every successful dispatch returns a dict with:

| Field | Type | Description |
|---|---|---|
| `status` | `"success"` \| `"failed"` \| `"blocked"` | Overall result |
| `bead_id` | str | The bead that was dispatched |
| `model_used` | str | Actual model the worker used |
| `provider` | str | Actual provider (ollama/openrouter/groq/…) |
| `logs` | str | Captured stdout/stderr (secrets redacted) |
| `files_changed` | list[str] | Files modified by the worker |
| `test_results` | dict \| null | `{passed, failed}` if tests were run |
| `duration_seconds` | float | Wall time of the dispatch |
| `error` | str \| null | Error message if `status == "failed"` |

A `blocked` status means the task type is denied or approval was not granted. The bead is left in `pending_approval` state for a human to unblock.
