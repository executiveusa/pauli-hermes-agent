#!/usr/bin/env bash
# Activate FREE MODE with secret files — wires up everything
#
# Usage:
#   bash free-mode/scripts/activate-secret-agent.sh
#
# This script:
# 1. Checks for .env file and validates required secrets
# 2. Starts the LiteLLM proxy
# 3. Exports FREE MODE environment variables
# 4. Verifies proxy health
# 5. Prints activation status
#
# Once activated, all Hermes Agent calls route through free/local providers.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${YELLOW}🚀 FREE MODE Secret Agent Activation${NC}"
echo "────────────────────────────────────────"

# ── Check for .env file ─────────────────────────────────────────────────────
if [ ! -f "$REPO_ROOT/.env" ]; then
  echo -e "${RED}❌ Error: .env file not found at $REPO_ROOT/.env${NC}"
  echo "   Please create .env with required secrets"
  exit 1
fi

echo -e "${GREEN}✓ .env file detected${NC}"

# ── Check for required secrets ──────────────────────────────────────────────
source "$REPO_ROOT/.env" 2>/dev/null || true

HAS_SECRETS=0
if [ -n "${GROQ_API_KEY:-}" ] || [ -n "${GEMINI_API_KEY:-}" ] || \
   [ -n "${NVIDIA_NIM_API_KEY:-}" ] || [ -n "${OPENAI_API_KEY:-}" ] || \
   [ -n "${ANTHROPIC_API_KEY:-}" ]; then
  HAS_SECRETS=1
  echo -e "${GREEN}✓ Detected API keys${NC}"
else
  echo -e "${YELLOW}⚠ No recognized API keys found in .env${NC}"
  echo "   (You can still use local providers like Ollama)"
fi

# ── Start LiteLLM proxy ─────────────────────────────────────────────────────
echo ""
echo "Starting LiteLLM proxy..."

# Check if Docker is available
if command -v docker &> /dev/null; then
  if docker compose -f "$REPO_ROOT/docker-compose.free-mode.yml" ps litellm | grep -q "litellm"; then
    echo -e "${GREEN}✓ Proxy already running${NC}"
  else
    echo "  Starting proxy container..."
    cd "$REPO_ROOT"
    docker compose -f docker-compose.free-mode.yml up -d litellm 2>/dev/null || true
    sleep 2
  fi
else
  echo -e "${YELLOW}⚠ Docker not available, skipping proxy startup${NC}"
  echo "   Run manually: docker compose -f docker-compose.free-mode.yml up -d litellm"
fi

# ── Export FREE MODE environment variables ──────────────────────────────────
echo ""
echo "Configuring FREE MODE environment..."

export FREE_MODE=true
export FREE_MODE_PROXY_BASE_URL="http://127.0.0.1:4000"
export FREE_MODE_PROVIDER="${FREE_MODE_PROVIDER:-auto}"
export FREE_MODE_MODEL="${FREE_MODE_MODEL:-free-auto}"

# Set ANTHROPIC_BASE_URL to route through proxy
export ANTHROPIC_BASE_URL="http://127.0.0.1:4000"

# Use LITELLM_MASTER_KEY or prompt for it
if [ -n "${LITELLM_MASTER_KEY:-}" ]; then
  export ANTHROPIC_API_KEY="${LITELLM_MASTER_KEY}"
  echo -e "${GREEN}✓ Using LITELLM_MASTER_KEY from .env${NC}"
elif [ -n "${FREE_MODE_PROXY_MASTER_KEY:-}" ]; then
  export ANTHROPIC_API_KEY="${FREE_MODE_PROXY_MASTER_KEY}"
  echo -e "${GREEN}✓ Using FREE_MODE_PROXY_MASTER_KEY from .env${NC}"
else
  export ANTHROPIC_API_KEY="change-me-local-master-key"
  echo -e "${YELLOW}⚠ No master key configured, using default${NC}"
fi

echo -e "${GREEN}✓ Environment configured${NC}"

# ── Verify proxy health ─────────────────────────────────────────────────────
echo ""
echo "Verifying proxy health..."

# Wait for proxy to be ready
MAX_ATTEMPTS=10
ATTEMPT=0
PROXY_READY=0

while [ $ATTEMPT -lt $MAX_ATTEMPTS ]; do
  if curl -s "http://127.0.0.1:4000/health" > /dev/null 2>&1; then
    PROXY_READY=1
    break
  fi
  ATTEMPT=$((ATTEMPT + 1))
  sleep 1
done

if [ $PROXY_READY -eq 1 ]; then
  echo -e "${GREEN}✓ Proxy is healthy${NC}"
else
  echo -e "${YELLOW}⚠ Proxy health check timeout${NC}"
  echo "   This may be normal if Docker/proxy is still starting"
fi

# ── Activation complete ─────────────────────────────────────────────────────
echo ""
echo -e "${GREEN}✅ FREE MODE Secret Agent Activated!${NC}"
echo ""
echo "Environment status:"
echo "  FREE_MODE=${FREE_MODE}"
echo "  ANTHROPIC_BASE_URL=${ANTHROPIC_BASE_URL}"
echo "  Provider: ${FREE_MODE_PROVIDER}"
echo "  Model: ${FREE_MODE_MODEL}"
echo ""

if [ $HAS_SECRETS -eq 1 ]; then
  echo -e "${GREEN}✓ API keys loaded from .env${NC}"
  echo ""
  echo "All Hermes Agent calls will now route through:"
  echo "  1. Local providers (Ollama, LM Studio, llama.cpp, vLLM)"
  echo "  2. Free cloud providers (Groq, Gemini, OpenRouter, NVIDIA NIM)"
  echo "  3. Configured paid providers (OpenAI, Anthropic, Mistral, etc.)"
else
  echo -e "${YELLOW}⚠ Configure API keys in .env to enable cloud providers${NC}"
fi

echo ""
echo "To use FREE MODE:"
echo "  source <(bash free-mode/scripts/activate-secret-agent.sh)"
echo ""
echo "Or manually in your shell:"
echo "  export FREE_MODE=true"
echo "  export ANTHROPIC_BASE_URL=http://127.0.0.1:4000"
echo "  hermes \"your prompt here\""
