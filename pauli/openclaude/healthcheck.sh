#!/usr/bin/env bash
# scripts/pauli/openclaude/healthcheck.sh
# Check whether the OpenClaude worker is installed and healthy.
#
# Exit codes:
#   0  — worker is healthy
#   1  — worker binary not found (install needed)
#   2  — worker binary found but not running (start needed)
#   3  — worker is running but gRPC port is not responding

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
VENDOR_DIR="${REPO_ROOT}/vendor/openclaude"
PORT="${OPENCLAUDE_PORT:-50051}"
HERMES_HOME="${HERMES_HOME:-${HOME}/.hermes}"
PID_FILE="${HERMES_HOME}/run/openclaude-worker.pid"

log()  { echo "[healthcheck] $*"; }
warn() { echo "[healthcheck] WARN: $*"; }
fail() { echo "[healthcheck] FAIL: $*" >&2; }

# ---------------------------------------------------------------------------
# 1. Check binary exists
# ---------------------------------------------------------------------------
BIN=""
if [[  -f "${VENDOR_DIR}/bin/openclaude" ]]; then
  BIN="${VENDOR_DIR}/bin/openclaude"
elif [[  -f "${VENDOR_DIR}/node_modules/.bin/openclaude" ]]; then
  BIN="${VENDOR_DIR}/node_modules/.bin/openclaude"
elif command -v openclaude >/dev/null 2>&1; then
  BIN="$(command -v openclaude)"
fi

if [[  -z "${BIN}" ]]; then
  fail "openclaude binary not found in vendor/openclaude or PATH"
  fail "Run: scripts/pauli/openclaude/install.sh"
  exit 1
fi

# Verify binary is executable and returns a version
VERSION=$("${BIN}" --version 2>&1 | head -1) || {
  fail "openclaude --version failed. Binary may be corrupted."
  exit 1
}
log "Binary OK: ${BIN}"
log "Version:   ${VERSION}"

# ---------------------------------------------------------------------------
# 2. Check if worker process is running
# ---------------------------------------------------------------------------
if [[  -f "${PID_FILE}" ]]; then
  PID=$(cat "${PID_FILE}")
  if [[  "${PID}" == "cli-stub" ]]; then
    log "Worker mode: cli-stub (no persistent process, per-task dispatch)"
    log "Status: OK (cli-stub mode — no healthcheck needed for gRPC)"
    exit 0
  fi
  if kill -0 "${PID}" 2>/dev/null; then
    log "Process:   running (PID ${PID})"
  else
    fail "PID file found (${PID}) but process is dead"
    fail "Run: scripts/pauli/openclaude/start.sh"
    exit 2
  fi
else
  warn "No PID file at ${PID_FILE} — worker may not have been started"
  warn "Run: scripts/pauli/openclaude/start.sh"
  exit 2
fi

# ---------------------------------------------------------------------------
# 3. Check gRPC port responds (requires nc or curl)
# ---------------------------------------------------------------------------
GRPC_OK=0
if command -v nc >/dev/null 2>&1; then
  if nc -z localhost "${PORT}" 2>/dev/null; then
    GRPC_OK=1
  fi
elif command -v curl >/dev/null 2>&1; then
  # HTTP/2 probe — gRPC over h2 should at least accept the connection
  if curl -s --max-time 3 "http://localhost:${PORT}" >/dev/null 2>&1; then
    GRPC_OK=1
  fi
fi

if [[  "${GRPC_OK}" -eq 1 ]]; then
  log "gRPC port ${PORT}: responding"
elif command -v nc >/dev/null 2>&1 || command -v curl >/dev/null 2>&1; then
  fail "gRPC port ${PORT} is not responding (worker may still be starting)"
  exit 3
else
  warn "Cannot probe gRPC port (nc and curl not available). Skipping port check."
fi

log "Status: HEALTHY"
exit 0
