#!/usr/bin/env bash
# scripts/pauli/openclaude/generate-config.sh
# Generate ~/.openclaude.json from environment variables.
#
# SECURITY: This script reads API keys from environment variables only.
# ~/.openclaude.json is NEVER committed to git (see .gitignore).
# Source API keys from Infisical, direnv, or ~/.hermes/.env — never hardcode.
#
# Usage:
#   scripts/pauli/openclaude/generate-config.sh [--provider PROVIDER] [--dry-run]
#
# Environment variables (at least one provider key must be set):
#   OPENROUTER_API_KEY   — OpenRouter key (recommended for free models)
#   GROQ_API_KEY         — Groq key
#   DEEPSEEK_API_KEY     — DeepSeek key
#   OPENAI_API_KEY       — OpenAI key (or any OpenAI-compatible provider)
#   OPENAI_BASE_URL      — Override base URL (e.g. Ollama: http://localhost:11434/v1)
#   OPENAI_MODEL         — Override model name
#   OLLAMA_HOST          — Ollama host (enables Ollama as default provider)

set -euo pipefail

PROVIDER="${1:-}"
DRY_RUN=0
OUTPUT="${HOME}/.openclaude.json"

for arg in "$@"; do
  case "$arg" in
    --provider=*) PROVIDER="${arg#*=}" ;;
    --provider)   shift; PROVIDER="${1:-}" ;;
    --dry-run)    DRY_RUN=1 ;;
    --output=*)   OUTPUT="${arg#*=}" ;;
  esac
done

log()  { echo "[generate-config] $*"; }
die()  { echo "[generate-config] ERROR: $*" >&2; exit 1; }
warn() { echo "[generate-config] WARN: $*"; }

# ---------------------------------------------------------------------------
# Auto-select provider (cheapest first) if not specified
# ---------------------------------------------------------------------------
if [[  -z "${PROVIDER}" ]]; then
  if [[  -n "${OLLAMA_HOST:-}" ]] || command -v ollama >/dev/null 2>&1; then
    PROVIDER="ollama"
  elif [[  -n "${OPENROUTER_API_KEY:-}" ]]; then
    PROVIDER="openrouter"
  elif [[  -n "${GROQ_API_KEY:-}" ]]; then
    PROVIDER="groq"
  elif [[  -n "${DEEPSEEK_API_KEY:-}" ]]; then
    PROVIDER="deepseek"
  elif [[  -n "${OPENAI_API_KEY:-}" ]]; then
    PROVIDER="openai"
  else
    die "No provider key found in environment. Set OPENROUTER_API_KEY, GROQ_API_KEY, DEEPSEEK_API_KEY, OPENAI_API_KEY, or OLLAMA_HOST."
  fi
fi

log "Selected provider: ${PROVIDER}"

# ---------------------------------------------------------------------------
# Build config JSON per provider
# ---------------------------------------------------------------------------
case "${PROVIDER}" in
  ollama)
    OLLAMA_ENDPOINT="${OLLAMA_HOST:-http://localhost:11434}/v1"
    MODEL="${OPENAI_MODEL:-qwen2.5-coder:7b}"
    CONFIG=$(cat <<JSON
{
  "provider": "ollama",
  "apiKey": "ollama",
  "model": "${MODEL}",
  "baseUrl": "${OLLAMA_ENDPOINT}"
}
JSON
)
    ;;
  openrouter)
    [[  -z "${OPENROUTER_API_KEY:-}" ]] && die "OPENROUTER_API_KEY is not set"
    MODEL="${OPENAI_MODEL:-meta-llama/llama-3.1-8b-instruct:free}"
    CONFIG=$(cat <<JSON
{
  "provider": "openrouter",
  "apiKey": "${OPENROUTER_API_KEY}",
  "model": "${MODEL}",
  "baseUrl": "https://openrouter.ai/api/v1"
}
JSON
)
    ;;
  groq)
    [[  -z "${GROQ_API_KEY:-}" ]] && die "GROQ_API_KEY is not set"
    MODEL="${OPENAI_MODEL:-llama-3.1-8b-instant}"
    CONFIG=$(cat <<JSON
{
  "provider": "groq",
  "apiKey": "${GROQ_API_KEY}",
  "model": "${MODEL}",
  "baseUrl": "https://api.groq.com/openai/v1"
}
JSON
)
    ;;
  deepseek)
    [[  -z "${DEEPSEEK_API_KEY:-}" ]] && die "DEEPSEEK_API_KEY is not set"
    MODEL="${OPENAI_MODEL:-deepseek-coder}"
    CONFIG=$(cat <<JSON
{
  "provider": "deepseek",
  "apiKey": "${DEEPSEEK_API_KEY}",
  "model": "${MODEL}",
  "baseUrl": "https://api.deepseek.com/v1"
}
JSON
)
    ;;
  openai)
    [[  -z "${OPENAI_API_KEY:-}" ]] && die "OPENAI_API_KEY is not set"
    MODEL="${OPENAI_MODEL:-gpt-4o-mini}"
    BASE_URL="${OPENAI_BASE_URL:-https://api.openai.com/v1}"
    CONFIG=$(cat <<JSON
{
  "provider": "openai",
  "apiKey": "${OPENAI_API_KEY}",
  "model": "${MODEL}",
  "baseUrl": "${BASE_URL}"
}
JSON
)
    ;;
  *)
    die "Unknown provider: ${PROVIDER}. Supported: ollama, openrouter, groq, deepseek, openai"
    ;;
esac

# ---------------------------------------------------------------------------
# Safety: verify no raw key appears literally in the JSON (sanity check)
# ---------------------------------------------------------------------------
if echo "${CONFIG}" | grep -qE '"apiKey": "sk-[A-Za-z0-9]{20,}"'; then
  warn "Output contains what looks like a real API key. Verify your .gitignore includes ~/.openclaude.json"
fi

# ---------------------------------------------------------------------------
# Write or preview
# ---------------------------------------------------------------------------
if [[  "${DRY_RUN}" -eq 1 ]]; then
  log "DRY RUN — would write to ${OUTPUT}:"
  # Redact the key in dry-run output
  echo "${CONFIG}" | sed 's/"apiKey": "[^"]*"/"apiKey": "***REDACTED***"/'
else
  # Write with restrictive permissions (owner-read/write only)
  install -m 600 /dev/null "${OUTPUT}"
  echo "${CONFIG}" > "${OUTPUT}"
  log "Config written to ${OUTPUT} (permissions: 600)"
  log "Key value is NOT shown — verify the file manually if needed"
fi

log ""
log "IMPORTANT: ${OUTPUT} contains secrets. It is in .gitignore and must NEVER be committed."
