#!/usr/bin/env bash
# Start the complete Hermes Agent stack:
# - API server (port 8642) for web UI
# - NIM proxy (port 8082) for free Claude Code inference
# - Hermes MCP servers

set -e

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$REPO_DIR"

echo "🚀 Starting Hermes Agent Stack..."

# Check for .env
if [ ! -f ~/.hermes/.env ]; then
    echo "⚠️  ~/.hermes/.env not found. Creating from template..."
    mkdir -p ~/.hermes
    touch ~/.hermes/.env
    echo "NVIDIA_NIM_API_KEY=nvapi-your-key-here" >> ~/.hermes/.env
    echo "Please edit ~/.hermes/.env and add your NVIDIA NIM API key"
fi

# Load environment
set -a
[ -f ~/.hermes/.env ] && source ~/.hermes/.env
set +a

# Start NIM proxy in background (if not already running)
if ! nc -z localhost 8082 2>/dev/null; then
    echo "📡 Starting NIM proxy on port 8082..."
    cd services/nim-proxy

    # Check if .env exists, create if needed
    if [ ! -f .env ]; then
        cp .env.example .env
        echo "⚠️  Created services/nim-proxy/.env — please configure NVIDIA_NIM_API_KEY"
    fi

    python3 -m uvicorn server:app --host 0.0.0.0 --port 8082 --log-level warning &
    NIM_PID=$!
    cd "$REPO_DIR"
    sleep 2
else
    echo "✅ NIM proxy already running on 8082"
fi

# Start Agent API server on port 8642
echo "🤖 Starting Agent API server on port 8642..."
export API_SERVER_PORT=8642
export API_SERVER_HOST=0.0.0.0
export API_SERVER_CORS_ORIGINS="http://localhost:3000,http://localhost:8642,https://pauli-hermes-agent.vercel.app"

python3 api_server.py &
API_PID=$!

echo ""
echo "✅ Hermes Agent Stack Running!"
echo ""
echo "📱 Web UI (voice agent):        http://localhost:8642 or https://pauli-hermes-agent.vercel.app"
echo "🎙️  Voice API:                   http://localhost:8642/api/chat"
echo "💻 Claude Code (free inference): ANTHROPIC_BASE_URL=http://localhost:8082"
echo ""
echo "Press Ctrl+C to stop all services"
echo ""

# Wait for processes
wait
