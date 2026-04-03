#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

echo "🚀 Starting Hermes Night Shift stack..."
docker compose up -d --build

echo "✅ Stack started."
echo "- API: http://localhost:8787/health"
echo "- Temporal UI: http://localhost:8080"
