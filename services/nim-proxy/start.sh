#!/usr/bin/env bash
# Start the NVIDIA NIM free inference proxy
# Routes all Claude API calls to moonshotai/kimi-k2-thinking (free on NVIDIA NIM)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

if [ ! -f ".env" ]; then
    echo "ERROR: .env not found. Copy .env.example to .env and set NVIDIA_NIM_API_KEY"
    exit 1
fi

# Install deps if needed
if ! python3 -c "import fastapi" 2>/dev/null; then
    echo "Installing dependencies..."
    pip install fastapi uvicorn httpx markdown-it-py pydantic python-dotenv tiktoken websockets pydantic-settings aiolimiter openai
fi

echo "Starting NIM proxy on port 8082..."
python3 -m uvicorn server:app --host 0.0.0.0 --port 8082 --log-level info
