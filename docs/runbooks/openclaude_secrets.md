# OpenClaude Secrets Runbook

## Overview

This runbook describes how API keys and credentials flow into the OpenClaude worker. The core rule is simple: **secrets never touch the git repository**. They flow at runtime from a secret manager (Infisical recommended) through environment variables into a local config file that lives only on the operator's machine.

---

## Secret Flow Diagram

```
Infisical (or .env file or CI secret store)
    |
    | (hermes setup / direnv / dotenv-vault)
    v
Environment variables (current shell session)
    |
    | scripts/pauli/openclaude/generate-config.sh
    v
~/.openclaude.json  (owner-read only, never committed)
    |
    | OpenClaude worker reads at startup
    v
Worker process (in-memory only — not logged, not propagated)
```

---

## What Never Gets Committed

The following must NEVER appear in any committed file, log, or CI artifact:

| Item | Why |
|---|---|
| `~/.openclaude.json` | Contains raw API keys |
| Any file matching `*.openclaude.json` | Same |
| Raw key values in bead specs | Beads are logged |
| Raw key values in dispatcher logs | Logs may be shipped |
| Keys in shell history | Use env injection, not inline args |

Both `~/.openclaude.json` and `vendor/openclaude/` are in `.gitignore`.

---

## Supported Provider Environment Variables

| Variable | Provider | Where to Get It |
|---|---|---|
| `OLLAMA_HOST` | Ollama (local) | Set to `http://localhost:11434` if running locally |
| `OPENROUTER_API_KEY` | OpenRouter | https://openrouter.ai/keys |
| `GROQ_API_KEY` | Groq | https://console.groq.com/keys |
| `DEEPSEEK_API_KEY` | DeepSeek | https://platform.deepseek.com |
| `OPENAI_API_KEY` | OpenAI (or compatible) | https://platform.openai.com/api-keys |
| `OPENAI_BASE_URL` | Any OpenAI-compatible provider | Set to custom endpoint |
| `OPENAI_MODEL` | Any provider | Override model name |

---

## Setting Up Secrets with Infisical (Recommended)

Infisical is the recommended secret manager for the Pauli stack. It eliminates dotfiles with raw keys.

### 1. Install Infisical CLI

```bash
brew install infisical/get-cli/infisical   # macOS
# or: curl -1sLf 'https://dl.cloudsmith.io/public/infisical/infisical-cli/setup.deb.sh' | bash && apt install infisical
```

### 2. Authenticate

```bash
infisical login
infisical init   # in the repo root — links to your project
```

### 3. Store secrets

```bash
infisical secrets set OPENROUTER_API_KEY=sk-or-...
infisical secrets set GROQ_API_KEY=gsk_...
```

### 4. Run generate-config with Infisical

```bash
infisical run -- scripts/pauli/openclaude/generate-config.sh
```

The script reads `OPENROUTER_API_KEY` from the Infisical-injected env and writes `~/.openclaude.json` with permissions `600`.

---

## Setting Up Secrets with direnv (.envrc)

If you use direnv for local development:

```bash
# .envrc (add to .gitignore if not already)
export OPENROUTER_API_KEY="sk-or-your-key-here"
export GROQ_API_KEY="gsk_your-key-here"
```

Then:

```bash
direnv allow
scripts/pauli/openclaude/generate-config.sh
```

**Warning:** `.envrc` files with raw keys must themselves be in `.gitignore`.

---

## Setting Up Secrets with ~/.hermes/.env

Hermes loads `~/.hermes/.env` at startup. Add your OpenClaude provider keys there:

```bash
# ~/.hermes/.env — never committed, owner-read only
OPENROUTER_API_KEY=sk-or-...
GROQ_API_KEY=gsk_...
```

Then run:

```bash
source ~/.hermes/.env   # or let Hermes load it
scripts/pauli/openclaude/generate-config.sh
```

---

## The ~/.openclaude.json File

The generated config looks like:

```json
{
  "provider": "openrouter",
  "apiKey": "<redacted — never shown in logs>",
  "model": "meta-llama/llama-3.1-8b-instruct:free",
  "baseUrl": "https://openrouter.ai/api/v1"
}
```

File permissions are set to `600` (owner read/write only) by the generate-config script.

### Verifying permissions

```bash
ls -la ~/.openclaude.json
# Should show: -rw------- 1 youruser ...
```

If permissions are wrong:

```bash
chmod 600 ~/.openclaude.json
```

---

## Rotating Keys

When a key is compromised or expired:

1. Revoke the old key at the provider's dashboard.
2. Generate a new key.
3. Update the secret in Infisical (or your chosen store).
4. Re-run `generate-config.sh` to overwrite `~/.openclaude.json`.
5. Restart the OpenClaude worker: `scripts/pauli/openclaude/start.sh`.

Never edit `~/.openclaude.json` directly with the key in your shell history. Always use the generate-config script or a secrets manager.

---

## CI/CD Secret Injection

For CI environments (GitHub Actions, GitLab CI, etc.):

```yaml
# .github/workflows/openclaude-worker.yml
env:
  OPENROUTER_API_KEY: ${{ secrets.OPENROUTER_API_KEY }}

steps:
  - name: Generate OpenClaude config
    run: scripts/pauli/openclaude/generate-config.sh
```

The config file is written to `~/.openclaude.json` in the CI runner's home directory and is not persisted between runs.

---

## What the Dispatcher Does With Secrets

The `OpenClaudeDispatcher` never reads `~/.openclaude.json` directly. It:

1. Passes only the env vars from the `env_passthrough` allowlist to the subprocess.
2. Sets `OPENAI_API_KEY` from a provider-specific key (e.g. `OPENROUTER_API_KEY`) when the selected provider requires it.
3. Calls `_redact_secrets()` on all captured output before returning results.

This means even if a key leaks into stdout, it is redacted before reaching Hermes or being written to any log.
