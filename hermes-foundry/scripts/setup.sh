#!/usr/bin/env bash
# scripts/setup.sh — Bootstrap Hermes Foundry development environment
set -euo pipefail

HERMES_HOME="${HERMES_HOME:-$HOME/.hermes}"
MODELS_DIR="$HERMES_HOME/models"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FOUNDRY_ROOT="$(dirname "$SCRIPT_DIR")"

echo "╔════════════════════════════════════════╗"
echo "║    Hermes Foundry — Setup              ║"
echo "╚════════════════════════════════════════╝"
echo ""

# ── Prerequisites check ──────────────────────────────────────────────────────

check_command() {
  if ! command -v "$1" &>/dev/null; then
    echo "  ✗ $1 not found — $2"
    return 1
  else
    echo "  ✓ $1 ($(command -v "$1"))"
    return 0
  fi
}

echo "Checking prerequisites…"
MISSING=0

check_command rustup  "Install from https://rustup.rs"         || MISSING=1
check_command cargo   "Install from https://rustup.rs"         || MISSING=1
check_command node    "Install from https://nodejs.org"        || MISSING=1
check_command pnpm    "Install with: npm install -g pnpm"      || MISSING=1
check_command python3 "Install from https://python.org"        || MISSING=1
check_command ffmpeg  "Install from https://ffmpeg.org"        || MISSING=1
check_command git     "Install from https://git-scm.com"       || MISSING=1

if [ "$MISSING" -eq 1 ]; then
  echo ""
  echo "Fix missing prerequisites, then re-run this script."
  exit 1
fi

echo ""
echo "Rust toolchain…"
rustup update stable
rustup target add x86_64-unknown-linux-gnu 2>/dev/null || true
cargo install tauri-cli --version "^2" 2>/dev/null || true

echo ""
echo "Node dependencies…"
cd "$FOUNDRY_ROOT"
pnpm install

echo ""
echo "Python requirements…"
if [ -f "$FOUNDRY_ROOT/../requirements.txt" ]; then
  pip3 install -r "$FOUNDRY_ROOT/../requirements.txt" --quiet
fi

echo ""
echo "Creating Hermes home directory…"
mkdir -p "$HERMES_HOME"
mkdir -p "$MODELS_DIR"
echo "  ✓ $HERMES_HOME"

echo ""
echo "Initializing git submodules (AtomicBot repos)…"
cd "$FOUNDRY_ROOT"
git submodule update --init --recursive 2>/dev/null || echo "  (skipped — no submodules in shallow clone)"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Setup complete. Next steps:"
echo ""
echo "  1. Download models:"
echo "     ./scripts/install-models.sh"
echo ""
echo "  2. Run the desktop app in dev mode:"
echo "     cd apps/desktop && pnpm tauri dev"
echo ""
echo "  3. Build installer:"
echo "     ./scripts/build-desktop.sh"
echo ""
