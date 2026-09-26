#!/usr/bin/env bash
# Bakes the standard mcp2cli tool configs for this repo. Idempotent — safe to
# re-run. Secrets are always referenced via env:/file:, never inlined.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

command -v mcp2cli >/dev/null 2>&1 || {
  echo "error: mcp2cli not found. Run: uv tool install mcp2cli" >&2
  exit 127
}

echo "==> baking filesystem MCP (repo working tree)"
mcp2cli bake create filesystem \
  --mcp-stdio "npx -y @modelcontextprotocol/server-filesystem $REPO_ROOT" \
  --cache-ttl 3600 \
  --description "Read/write access to the pauli-hermes-agent repo working tree" \
  --force

echo "==> baking github MCP (fallback only — gh CLI is preferred)"
mcp2cli bake create github \
  --mcp "https://api.githubcopilot.com/mcp/" \
  --auth-header "Authorization:Bearer env:GITHUB_TOKEN" \
  --cache-ttl 3600 \
  --description "GitHub MCP fallback (gh CLI is preferred; use this only when gh is unavailable)" \
  --force

if [[ -n "${PAULI_BRAND_STUDIO_ROOT:-}" && -f "${PAULI_BRAND_STUDIO_ROOT}/interfaces/mcp/server.mjs" ]]; then
  echo "==> baking Pauli Design Factory MCP"
  mcp2cli bake create brand-studio \
    --mcp-stdio "node ${PAULI_BRAND_STUDIO_ROOT}/interfaces/mcp/server.mjs" \
    --cache-ttl 300 \
    --description "Pauli Design Factory — ICM design workflows, design capabilities, canonical request normalization and guarded reference runs" \
    --force
else
  echo "==> Pauli Design Factory MCP skipped"
  echo "    Set PAULI_BRAND_STUDIO_ROOT to a checkout of executiveusa/brand-kit-builder-"
  echo "    containing interfaces/mcp/server.mjs, then rerun this bootstrap."
fi

echo "==> browser/search MCP: none configured in this environment."
echo "    Native WebSearch/WebFetch cover this today. To bake a standalone"
echo "    one later: mcp2cli bake create browser --mcp <url> --auth-header ..."

echo "==> docs/search MCP: none configured in this environment."
echo "    Context7 (mcp__Context7__*) covers this natively today. To bake a"
echo "    standalone one later: mcp2cli bake create docs --mcp <url> --auth-header ..."

echo "==> baked tools:"
mcp2cli bake list
