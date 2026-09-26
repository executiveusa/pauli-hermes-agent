#!/bin/bash
# Stop FREE MODE proxy

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "🛑 Stopping FREE MODE proxy..."

cd "$REPO_ROOT"
docker compose -f docker-compose.free-mode.yml down

echo "✅ FREE MODE proxy stopped"
echo ""
echo "To disable FREE MODE in your shell:"
echo "  unset FREE_MODE"
echo "  unset ANTHROPIC_BASE_URL"
echo "  unset ANTHROPIC_AUTH_TOKEN"
