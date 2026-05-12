#!/usr/bin/env bash
# scripts/pauli/openclaude/start.sh
# Start OpenClaude worker in headless gRPC mode (or CLI mode as fallback).
#
# Usage:
#   scripts/pauli/openclaude/start.sh [--mode grpc|cli] [--port PORT]
#
# Environment:
#   OPENCLAUDE_MODE    grpc|cli  (default: grpc)
#   OPENCLAUDE_PORT    gRPC port (default: 50051)
#   OPENCLAUDE_LOG     Path to log file (default: ~/.hermes/logs/openclaude-worker.log)

set -euo pipefail

# ---------------------------------------------------------------------------
# Config
# ---------------------------------------------------------------------------
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
VENDOR_DIR="${REPO_ROOT}/vendor/openclaude"
MODE="${OPENCLAUDE_MODE:-grpc}"
PORT="${OPENCLAUDE_PORT:-50051}"
HERMES_HOME="${HERMES_HOME:-${HOME}/.hermes}"
LOG_FILE="${OPENCLAUDE_LOG:-${HERMES_HOME}/logs/openclaude-worker.log}"
PID_FILE="${HERMES_HOME}/run/openclaude-worker.pid"

for arg in "$@"; do
  case "$arg" in
    --mode=*)  MODE="${arg#*=}" ;;
    --port=*)  PORT="${arg#*=}" ;;
    --mode)    shift; MODE="$1" ;;
    --port)    shift; PORT="$1" ;;
  esac
done

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------
log() { echo "[start.sh] $*"; }
die() { echo "[start.sh] ERROR: $*" >&2; exit 1; }

# ---------------------------------------------------------------------------
# Find binary
# ---------------------------------------------------------------------------
BIN=""
if [[  -f "${VENDOR_DIR}/bin/openclaude" ]]; then
  BIN="${VENDOR_DIR}/bin/openclaude"
elif [[  -f "${VENDOR_DIR}/node_modules/.bin/openclaude" ]]; then
  BIN="${VENDOR_DIR}/node_modules/.bin/openclaude"
elif command -v openclaude >/dev/null 2>&1; then
  BIN="openclaude"
else
  die "openclaude binary not found. Run: scripts/pauli/openclaude/install.sh"
fi

# ---------------------------------------------------------------------------
# Check if already running
# ---------------------------------------------------------------------------
if [[  -f "${PID_FILE}" ]]; then
  OLD_PID=$(cat "${PID_FILE}")
  if kill -0 "${OLD_PID}" 2>/dev/null; then
    log "OpenClaude worker already running (PID ${OLD_PID}). Use stop or restart."
    exit 0
  else
    log "Stale PID file found (PID ${OLD_PID} is dead). Cleaning up."
    rm -f "${PID_FILE}"
  fi
fi

# ---------------------------------------------------------------------------
# Ensure log and run dirs exist
# ---------------------------------------------------------------------------
mkdir -p "$(dirname "${LOG_FILE}")"
mkdir -p "$(dirname "${PID_FILE}")"

# ---------------------------------------------------------------------------
# Load config (secrets from env only — never from committed files)
# ---------------------------------------------------------------------------
CONFIG_FILE="${HOME}/.openclaude.json"
if [[  ! -f "${CONFIG_FILE}" ]]; then
  log "WARNING: ${CONFIG_FILE} not found. Run generate-config.sh to create it."
fi

# ---------------------------------------------------------------------------
# Start worker
# ---------------------------------------------------------------------------
log "Starting OpenClaude worker (mode=${MODE}, port=${PORT})"
log "Log: ${LOG_FILE}"

if [[  "${MODE}" == "grpc" ]]; then
  nohup "${BIN}" --headless --grpc-port "${PORT}" \
    >> "${LOG_FILE}" 2>&1 &
else
  # CLI mode: keeps a persistent session ready for dispatch via stdin
  log "WARNING: CLI mode does not support persistent sessions. Each task spawns a new process."
  log "Worker started in CLI stub mode (no background process — tasks are dispatched per-invocation)."
  echo "cli-stub" > "${PID_FILE}"
  exit 0
fi

WORKER_PID=$!
echo "${WORKER_PID}" > "${PID_FILE}"
log "Worker started with PID ${WORKER_PID}"

# Give it a moment to fail fast if config is broken
sleep 2
if ! kill -0 "${WORKER_PID}" 2>/dev/null; then
  die "Worker exited immediately. Check log: ${LOG_FILE}"
fi

log "Worker is running (PID ${WORKER_PID})"
log "Healthcheck: scripts/pauli/openclaude/healthcheck.sh"
