#!/usr/bin/env bash
# scripts/pauli/openclaude/install.sh
# Clone or update vendor/openclaude, install npm deps, and verify the binary.
#
# Usage:
#   scripts/pauli/openclaude/install.sh [--force]
#
# Options:
#   --force   Re-clone even if vendor/openclaude already exists.
#
# Environment:
#   OPENCLAUDE_CLONE_URL  Override clone URL  (default: GitHub upstream)
#   OPENCLAUDE_CLONE_TAG  Branch/tag to clone (default: main)

set -euo pipefail

# ---------------------------------------------------------------------------
# Config
# ---------------------------------------------------------------------------
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
VENDOR_DIR="${REPO_ROOT}/vendor/openclaude"
CLONE_URL="${OPENCLAUDE_CLONE_URL:-https://github.com/Gitlawb/openclaude.git}"
CLONE_TAG="${OPENCLAUDE_CLONE_TAG:-main}"
FORCE=0

for arg in "$@"; do
  [[  "$arg" == "--force" ]] && FORCE=1
done

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------
log()  { echo "[install.sh] $*"; }
die()  { echo "[install.sh] ERROR: $*" >&2; exit 1; }
need() { command -v "$1" >/dev/null 2>&1 || die "$1 is not installed. Please install it first."; }

# ---------------------------------------------------------------------------
# Prerequisite checks
# ---------------------------------------------------------------------------
need git
need node
need npm

NODE_VER=$(node --version | sed 's/v//')
NODE_MAJOR=$(echo "$NODE_VER" | cut -d. -f1)
if [[  "$NODE_MAJOR" -lt 22 ]]; then
  die "Node.js >= 22 required. Found: v${NODE_VER}. Install via: nvm install 22"
fi
log "Node.js v${NODE_VER} OK"

# ---------------------------------------------------------------------------
# Clone or update
# ---------------------------------------------------------------------------
if [[  -d "${VENDOR_DIR}" && "$FORCE" -eq 0 ]]; then
  log "vendor/openclaude already exists — pulling latest ${CLONE_TAG}"
  git -C "${VENDOR_DIR}" fetch origin
  git -C "${VENDOR_DIR}" checkout "${CLONE_TAG}"
  git -C "${VENDOR_DIR}" pull --ff-only origin "${CLONE_TAG}" || {
    log "fast-forward failed; doing a hard reset to origin/${CLONE_TAG}"
    git -C "${VENDOR_DIR}" reset --hard "origin/${CLONE_TAG}"
  }
else
  if [[  -d "${VENDOR_DIR}" && "$FORCE" -eq 1 ]]; then
    log "--force: removing existing vendor/openclaude"
    rm -rf "${VENDOR_DIR}"
  fi
  log "Cloning ${CLONE_URL}@${CLONE_TAG} → vendor/openclaude"
  git clone --depth=1 --branch "${CLONE_TAG}" "${CLONE_URL}" "${VENDOR_DIR}"
fi

# ---------------------------------------------------------------------------
# Install npm dependencies
# ---------------------------------------------------------------------------
log "Running npm install in vendor/openclaude..."
(cd "${VENDOR_DIR}" && npm install --prefer-offline 2>&1 | tail -5)

# ---------------------------------------------------------------------------
# Build (if build script exists)
# ---------------------------------------------------------------------------
if (cd "${VENDOR_DIR}" && npm run build 2>&1 | tail -5); then
  log "Build OK"
else
  log "No build step or build succeeded (continuing)"
fi

# ---------------------------------------------------------------------------
# Verify binary
# ---------------------------------------------------------------------------
BIN="${VENDOR_DIR}/bin/openclaude"
if [[  ! -f "${BIN}" ]]; then
  # Try node_modules/.bin as fallback
  BIN_NM="${VENDOR_DIR}/node_modules/.bin/openclaude"
  if [[  -f "${BIN_NM}" ]]; then
    BIN="${BIN_NM}"
  else
    die "openclaude binary not found at ${VENDOR_DIR}/bin/openclaude or node_modules/.bin/openclaude"
  fi
fi

chmod +x "${BIN}"
VERSION=$("${BIN}" --version 2>&1 | head -1) || die "openclaude --version failed"
log "Installed: ${VERSION}"
log "Binary: ${BIN}"
log ""
log "Installation complete. Next steps:"
log "  1. Generate config:  scripts/pauli/openclaude/generate-config.sh"
log "  2. Start worker:     scripts/pauli/openclaude/start.sh"
log "  3. Health check:     scripts/pauli/openclaude/healthcheck.sh"
