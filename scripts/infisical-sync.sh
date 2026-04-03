#!/usr/bin/env bash
# infisical-sync.sh — Pull secrets from Infisical and write to .env
# Infisical slug: agent-hermes
# Project ID: e2f5c669-4fdd-4c9f-be8c-fc56bf62549c
# Usage: bash scripts/infisical-sync.sh [environment]
# environment: dev (default) | staging | prod

set -euo pipefail

ENVIRONMENT="${1:-dev}"
INFISICAL_PROJECT_ID="e2f5c669-4fdd-4c9f-be8c-fc56bf62549c"
OUTPUT_FILE=".env"

echo "🔐 Syncing secrets from Infisical (slug: agent-hermes, env: $ENVIRONMENT)..."

# Check infisical CLI is installed
if ! command -v infisical &>/dev/null; then
    echo "Installing Infisical CLI..."
    curl -1sLf 'https://dl.cloudsmith.io/public/infisical/infisical-cli/setup.deb.sh' | sudo bash
    sudo apt-get install -y infisical
fi

# Export via infisical CLI
infisical export \
    --projectId "$INFISICAL_PROJECT_ID" \
    --env "$ENVIRONMENT" \
    --format dotenv \
    > "$OUTPUT_FILE"

echo "✅ Secrets written to $OUTPUT_FILE ($(wc -l < "$OUTPUT_FILE") keys)"
echo "⚠️  Never commit .env — it's in .gitignore"
