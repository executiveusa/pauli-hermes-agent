#!/usr/bin/env bash
set -euo pipefail
# Agent MAXX — Bootstrap: first-time server setup

echo "🐕 Agent MAXX Bootstrap"

# Ensure docker + compose
if ! command -v docker &>/dev/null; then
  echo "Installing Docker..."
  curl -fsSL https://get.docker.com | sh
fi

if ! docker compose version &>/dev/null; then
  echo "ERROR: docker compose plugin not found"
  exit 1
fi

# Copy env
if [ ! -f .env ]; then
  cp .env.example .env
  echo "⚠️  Created .env from .env.example — fill in your secrets!"
  echo "   Then run: docker compose up -d"
  exit 0
fi

# Build and start
echo "Building images..."
docker compose build

echo "Starting services..."
docker compose up -d

echo "Waiting for health checks..."
sleep 10

# Smoke test
if curl -sf http://localhost:8642/healthz >/dev/null 2>&1; then
  echo "✅ hermes-api healthy"
else
  echo "⚠️  hermes-api not responding — check logs: docker compose logs hermes-api"
fi

if curl -sf http://localhost:3000/healthz >/dev/null 2>&1; then
  echo "✅ dashboard healthy"
else
  echo "⚠️  dashboard not responding — check logs: docker compose logs hermes-dashboard"
fi

echo ""
echo "🐕 Agent MAXX is running!"
echo "   Dashboard: http://localhost:3000"
echo "   API:       http://localhost:8642"
echo "   MCP:       http://localhost:8642/mcp"
