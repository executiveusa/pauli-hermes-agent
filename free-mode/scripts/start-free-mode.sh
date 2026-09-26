#!/bin/bash
# Start FREE MODE proxy and configure environment

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "🚀 Starting FREE MODE proxy..."

# Load .env if exists
if [ -f "$REPO_ROOT/.env" ]; then
    export $(grep -v '^#' "$REPO_ROOT/.env" | xargs)
fi

# Verify Docker
if ! command -v docker &> /dev/null; then
    echo "❌ Docker not found. Install Docker first."
    exit 1
fi

# Start proxy
cd "$REPO_ROOT"
docker compose -f docker-compose.free-mode.yml up -d

# Wait for health
echo "⏳ Waiting for proxy to start..."
for i in {1..30}; do
    if curl -sf http://127.0.0.1:4000/health > /dev/null 2>&1; then
        echo "✅ Proxy is healthy"
        break
    fi
    if [ $i -eq 30 ]; then
        echo "❌ Proxy failed to start"
        docker logs free-mode-litellm
        exit 1
    fi
    sleep 1
done

# Export environment for Claude Code / Anthropic
echo ""
echo "📋 FREE MODE Environment Variables:"
echo "======================================"
echo "export FREE_MODE=true"
echo "export ANTHROPIC_BASE_URL=http://127.0.0.1:4000"
echo "export ANTHROPIC_AUTH_TOKEN=${LITELLM_MASTER_KEY:-change-me-local-master-key}"
echo "export CLAUDE_CODE_ENABLE_GATEWAY_MODEL_DISCOVERY=1"
echo ""
echo "To activate in your shell:"
echo "  export FREE_MODE=true"
echo "  export ANTHROPIC_BASE_URL=http://127.0.0.1:4000"
echo "  export ANTHROPIC_AUTH_TOKEN=\${LITELLM_MASTER_KEY:-change-me-local-master-key}"
echo ""
echo "✅ FREE MODE proxy is running!"
echo "   Health: http://127.0.0.1:4000/health"
echo "   Models: http://127.0.0.1:4000/v1/models (requires auth key)"
echo ""
echo "For logs: docker logs -f free-mode-litellm"
