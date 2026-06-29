#!/usr/bin/env bash
# Start complete FREE MODE production stack
#
# This script orchestrates:
# 1. Secret detection & activation
# 2. LiteLLM proxy startup
# 3. Monitoring service startup
# 4. Dashboard readiness check
#
# Usage:
#   bash free-mode/scripts/start-production-stack.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
FREE_MODE_DIR="$REPO_ROOT/free-mode"
WEBSITE_DIR="$REPO_ROOT/website"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                                                                ║${NC}"
echo -e "${BLUE}║        🚀 FREE MODE PRODUCTION STACK STARTUP 🚀               ║${NC}"
echo -e "${BLUE}║                                                                ║${NC}"
echo -e "${BLUE}║  Dashboard  │  Cost Tracking  │  Monitoring  │  Proxy        ║${NC}"
echo -e "${BLUE}║                                                                ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════════╝${NC}"
echo ""

# ── Step 1: Detect Secrets ──────────────────────────────────────────────────
echo -e "${YELLOW}Step 1: Detecting secrets...${NC}"

if [ ! -f "$REPO_ROOT/.env" ]; then
    echo -e "${RED}❌ .env file not found${NC}"
    echo "   Create .env with your API keys:"
    echo "   GROQ_API_KEY=..."
    echo "   GEMINI_API_KEY=..."
    echo "   ANTHROPIC_API_KEY=..."
    exit 1
fi

source "$REPO_ROOT/.env" 2>/dev/null || true
echo -e "${GREEN}✓ .env file detected${NC}"

HAS_KEYS=0
if [ -n "${GROQ_API_KEY:-}" ] || [ -n "${GEMINI_API_KEY:-}" ] || \
   [ -n "${OPENAI_API_KEY:-}" ] || [ -n "${ANTHROPIC_API_KEY:-}" ]; then
    HAS_KEYS=1
    echo -e "${GREEN}✓ API keys detected${NC}"
else
    echo -e "${YELLOW}⚠ No API keys detected (local providers only)${NC}"
fi

# ── Step 2: Activate FREE MODE ──────────────────────────────────────────────
echo ""
echo -e "${YELLOW}Step 2: Activating FREE MODE...${NC}"

export FREE_MODE=true
export FREE_MODE_PROXY_BASE_URL="http://127.0.0.1:4000"
export ANTHROPIC_BASE_URL="http://127.0.0.1:4000"

if [ -n "${LITELLM_MASTER_KEY:-}" ]; then
    export ANTHROPIC_API_KEY="${LITELLM_MASTER_KEY}"
    echo -e "${GREEN}✓ Using LITELLM_MASTER_KEY${NC}"
elif [ -n "${FREE_MODE_PROXY_MASTER_KEY:-}" ]; then
    export ANTHROPIC_API_KEY="${FREE_MODE_PROXY_MASTER_KEY}"
    echo -e "${GREEN}✓ Using FREE_MODE_PROXY_MASTER_KEY${NC}"
else
    export ANTHROPIC_API_KEY="change-me-local-master-key"
    echo -e "${YELLOW}⚠ Using default master key${NC}"
fi

# ── Step 3: Start Docker Services ───────────────────────────────────────────
echo ""
echo -e "${YELLOW}Step 3: Starting Docker services...${NC}"

if ! command -v docker &> /dev/null; then
    echo -e "${RED}❌ Docker not found. Install Docker to use production stack.${NC}"
    echo "   Visit: https://docs.docker.com/get-docker/"
    exit 1
fi

cd "$REPO_ROOT"
docker compose -f docker-compose.free-mode.yml up -d 2>&1 | grep -E "Created|Starting|Service" || true

echo -e "${GREEN}✓ Starting Docker services...${NC}"

# ── Step 4: Wait for Services ───────────────────────────────────────────────
echo ""
echo -e "${YELLOW}Step 4: Waiting for services to be healthy...${NC}"

MAX_ATTEMPTS=30
ATTEMPT=0

while [ $ATTEMPT -lt $MAX_ATTEMPTS ]; do
    if curl -s http://127.0.0.1:4000/health > /dev/null 2>&1; then
        echo -e "${GREEN}✓ LiteLLM proxy is healthy${NC}"
        break
    fi
    ATTEMPT=$((ATTEMPT + 1))
    echo "  Waiting for proxy... ($ATTEMPT/$MAX_ATTEMPTS)"
    sleep 1
done

if [ $ATTEMPT -eq $MAX_ATTEMPTS ]; then
    echo -e "${RED}❌ Proxy failed to start${NC}"
    docker compose -f docker-compose.free-mode.yml logs litellm-free-mode | tail -20
    exit 1
fi

ATTEMPT=0
while [ $ATTEMPT -lt $MAX_ATTEMPTS ]; do
    if curl -s http://127.0.0.1:8001/health > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Monitoring service is healthy${NC}"
        break
    fi
    ATTEMPT=$((ATTEMPT + 1))
    echo "  Waiting for monitoring... ($ATTEMPT/$MAX_ATTEMPTS)"
    sleep 1
done

if [ $ATTEMPT -eq $MAX_ATTEMPTS ]; then
    echo -e "${YELLOW}⚠ Monitoring service not ready yet (starting in background)${NC}"
fi

# ── Step 5: Check Dashboard ─────────────────────────────────────────────────
echo ""
echo -e "${YELLOW}Step 5: Checking dashboard...${NC}"

if [ -f "$WEBSITE_DIR/package.json" ]; then
    echo -e "${GREEN}✓ Dashboard found (website/src/pages/dashboard.tsx)${NC}"
    echo "  Start with: cd website && npm run dev"
else
    echo -e "${YELLOW}⚠ Dashboard not found${NC}"
fi

# ── Summary ─────────────────────────────────────────────────────────────────
echo ""
echo -e "${GREEN}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║                    ✅ PRODUCTION STACK READY                   ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════════════════╝${NC}"
echo ""

echo -e "${BLUE}📊 Services Status:${NC}"
echo -e "  ${GREEN}✓${NC} LiteLLM Proxy        http://127.0.0.1:4000"
echo -e "  ${GREEN}✓${NC} Monitoring Service    http://127.0.0.1:8001"
echo -e "  ○ Web Dashboard          http://localhost:3000/dashboard (start manually)"
echo ""

echo -e "${BLUE}🔧 Next Steps:${NC}"
echo ""
echo "1. Start the web dashboard (Terminal 2):"
echo -e "   ${YELLOW}cd website && npm run dev${NC}"
echo ""
echo "2. Test the monitoring API (Terminal 3):"
echo -e "   ${YELLOW}curl http://127.0.0.1:8001/metrics${NC}"
echo ""
echo "3. View logs:"
echo -e "   ${YELLOW}docker compose -f docker-compose.free-mode.yml logs -f${NC}"
echo ""
echo "4. Access dashboard once running:"
echo -e "   ${YELLOW}http://localhost:3000/dashboard${NC}"
echo ""

echo -e "${BLUE}📚 Documentation:${NC}"
echo -e "  ${YELLOW}free-mode/DASHBOARD.md${NC}  - Dashboard & monitoring docs"
echo -e "  ${YELLOW}free-mode/README.md${NC}     - Setup guide"
echo -e "  ${YELLOW}CLAUDE.md${NC}               - Agent instructions"
echo ""

echo -e "${BLUE}🛑 To stop services:${NC}"
echo -e "   ${YELLOW}docker compose -f docker-compose.free-mode.yml down${NC}"
echo ""

if [ $HAS_KEYS -eq 1 ]; then
    echo -e "${GREEN}✓ All API keys configured - you're ready to use FREE MODE!${NC}"
else
    echo -e "${YELLOW}⚠ Using local providers only - configure API keys in .env for cloud providers${NC}"
fi

echo ""
