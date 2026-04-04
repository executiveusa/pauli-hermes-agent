#!/usr/bin/env bash
# scripts/build-desktop.sh — Build Tauri desktop installer
#
# Usage:
#   ./scripts/build-desktop.sh              # Current platform
#   ./scripts/build-desktop.sh --target mac # macOS universal binary
#   ./scripts/build-desktop.sh --debug      # Debug build (no installer)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FOUNDRY_ROOT="$(dirname "$SCRIPT_DIR")"
APP_DIR="$FOUNDRY_ROOT/apps/desktop"

TARGET=""
DEBUG=false

for arg in "$@"; do
  case $arg in
    --debug)   DEBUG=true ;;
    --target)  ;;
    mac)       TARGET="universal-apple-darwin" ;;
    linux)     TARGET="x86_64-unknown-linux-gnu" ;;
    windows)   TARGET="x86_64-pc-windows-msvc" ;;
  esac
done

echo "╔════════════════════════════════════════╗"
echo "║    Hermes Foundry — Build              ║"
echo "╚════════════════════════════════════════╝"
echo ""

echo "Building frontend…"
cd "$APP_DIR"
pnpm build

echo ""
echo "Building Tauri app…"
if [ "$DEBUG" = true ]; then
  cargo tauri build --debug
elif [ -n "$TARGET" ]; then
  cargo tauri build --target "$TARGET"
else
  cargo tauri build
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Build complete."
echo "Installers in: apps/desktop/src-tauri/target/release/bundle/"
