# OpenClaude Integration Report

## What Is OpenClaude?

OpenClaude (npm package `@gitlawb/openclaude`, v0.9.2) is an open-source coding-agent CLI and headless gRPC server that exposes agentic coding workflows to any LLM provider. It is emphatically **not** a fork of Claude Code — the name is a deliberate wink at Anthropic's ecosystem, but the runtime is entirely independent. The project describes itself as "OpenClaude opens coding-agent workflows to any LLM — OpenAI, Gemini, DeepSeek, Ollama, and 200+ models."

It is MIT-licensed and maintained at https://github.com/Gitlawb/openclaude.

---

## Interface Options

### 1. Interactive CLI (primary interface)

The binary `openclaude` launches a terminal REPL with prompt, streaming output, tool invocation, and multi-step agent loops. This is the interface humans interact with directly. It is suitable as a CLI fallback when gRPC connectivity is unavailable or not yet configured.

```
openclaude                          # interactive mode, uses ~/.openclaude.json config
openclaude --provider openrouter    # provider override
```

### 2. Headless gRPC Server (preferred integration interface)

OpenClaude can run as a long-lived background service exposing port **50051** with bidirectional streaming gRPC. This lets external orchestrators (like Hermes + Flywheel) dispatch tasks without spawning a new process per task:

```
openclaude --headless --grpc-port 50051
```

The gRPC API streams tool calls and responses back to the caller, allowing real-time progress monitoring and early cancellation. This is the preferred interface for the Flywheel dispatcher when the worker is already running.

### 3. VS Code Extension

A bundled extension provides launch integration and theme support. Not relevant to the Hermes integration.

---

## Technology Stack

| Layer | Technology |
|---|---|
| Runtime | Bun (TypeScript, Node >= 22 required) |
| Package manager | npm (global install) or bun |
| gRPC | `@grpc/grpc-js` + protobuf |
| CLI framework | Commander + Chalk |
| LLM SDKs | Anthropic SDK, OpenAI SDK, Gemini SDK |
| Build | `bun run build` |
| Tests | Bun test runner |

---

## Provider Support

OpenClaude routes to any OpenAI-compatible endpoint plus native adapters:

| Category | Providers |
|---|---|
| Free/local | Ollama (no key needed), LM Studio |
| Cheap/hosted | Groq, DeepSeek, Mistral, OpenRouter (200+ models) |
| Mid-tier | GitHub Models, Gemini API |
| Premium | OpenAI, Anthropic Bedrock, Vertex AI, Azure Foundry |

### Provider Priority for Pauli Flywheel (cheapest-first)

1. **Ollama** — local inference, zero cost, zero API key required
2. **OpenRouter free tier** — routes to free community models (Llama 3, Mistral 7B, etc.)
3. **Groq** — fast inference on open-source models, generous free tier
4. **DeepSeek** — extremely low cost for coding tasks ($0.14/M input tokens)
5. **GitHub Models** — free for GitHub users, gated by usage limits
6. **Mistral** — European mid-tier with code models (Codestral)
7. **OpenAI / Anthropic** — paid fallback only

Configuration is done via `~/.openclaude.json` (secrets) or environment variables (`OPENAI_API_KEY`, `OPENAI_BASE_URL`, `OPENAI_MODEL`).

---

## Configuration

### ~/.openclaude.json (secrets file — NEVER commit)

```json
{
  "provider": "openrouter",
  "apiKey": "<OPENROUTER_API_KEY>",
  "model": "meta-llama/llama-3.1-8b-instruct:free",
  "baseUrl": "https://openrouter.ai/api/v1"
}
```

### Environment variable overrides

```bash
OPENAI_API_KEY=...        # key for any OpenAI-compatible provider
OPENAI_BASE_URL=...       # endpoint override (Ollama: http://localhost:11434/v1)
OPENAI_MODEL=...          # model name
OPENCLAUDE_PORT=50051     # gRPC port (headless mode)
```

---

## Architecture Fit: OpenClaude as a Hermes Worker

### Role Boundaries

```
Hermes (orchestrator)
  └── Flywheel (task queue + bead dispatcher)
        └── OpenClaude worker (coding worker)
              ├── gRPC interface (preferred, port 50051)
              └── CLI interface (fallback, subprocess)
```

Hermes remains the **sole orchestrator**. It creates beads (task units), assigns them, and collects results. OpenClaude executes coding subtasks: refactoring, test repair, documentation generation, feature implementation, MCP server scaffolding, and similar coding work.

OpenClaude must **never** be given access to:
- Production deployment commands
- Secret rotation workflows
- Destructive git operations (force-push, branch deletion)
- Database migration scripts

### Integration Points

1. **Health check**: `openclaude --version` or gRPC `HealthCheck` call to confirm the worker is alive.
2. **Task dispatch**: Flywheel serializes bead spec to JSON and passes it as a CLI argument or gRPC request.
3. **Result collection**: stdout/stderr capture (CLI mode) or streaming gRPC response (gRPC mode).
4. **Approval gate**: Risky actions (git push, file deletion) require Hermes to prompt the user before OpenClaude proceeds.

### Installation Path

```
vendor/openclaude/       <- git submodule or cloned repo
  package.json
  bin/openclaude         <- the worker binary
```

The Flywheel dispatcher sets `PATH` to include `vendor/openclaude/bin/` so it can invoke `openclaude` without a global install.

---

## Risk Assessment

| Risk | Mitigation |
|---|---|
| OpenClaude writes to wrong files | Sandboxed workdir per bead; CWD is always the bead's scratch directory |
| OpenClaude exfiltrates secrets | No API keys in bead spec; secrets injected only via env and never logged |
| Worker hangs | Subprocess timeout enforced by dispatcher (default 10 minutes per bead) |
| gRPC unavailable | Dispatcher falls back to CLI mode transparently |
| Node version mismatch | `install.sh` checks `node --version` before proceeding |
| Provider key exhausted | Dispatcher cycles through provider priority list |

---

## Conclusion

OpenClaude is a mature, multi-provider coding agent with both a human-facing CLI and a machine-facing gRPC server. It is well-suited as an isolated coding worker inside the Hermes + Flywheel stack. The Flywheel dispatcher (`pauli/flywheel/dispatchers/openclaude_dispatcher.py`) manages the full lifecycle: install check, health check, provider selection, task dispatch, result capture, and bead status update. Hermes never cedes orchestration authority — OpenClaude is strictly a leaf worker.
