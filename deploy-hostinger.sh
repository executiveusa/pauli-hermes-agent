#!/usr/bin/env bash
# Hermes Agent Deployment for Hostinger VPS
# One-command setup for https://31.220.58.212
# Usage: bash deploy-hostinger.sh

set -e

REPO_URL="https://github.com/executiveusa/pauli-hermes-agent.git"
REPO_DIR="/opt/pauli-hermes-agent"
VPS_IP="31.220.58.212"

echo "╔═══════════════════════════════════════════════════════════╗"
echo "║     Hermes Agent - Hostinger VPS Deployment Script        ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""

# Step 1: Check prerequisites
echo "📋 Checking prerequisites..."
if ! command -v python3 &> /dev/null; then
    echo "❌ Python3 not found. Installing..."
    apt-get update && apt-get install -y python3 python3-pip python3-venv
fi

if ! command -v git &> /dev/null; then
    echo "❌ Git not found. Installing..."
    apt-get install -y git
fi

echo "✅ Prerequisites satisfied"
echo ""

# Step 2: Clone or update repository
echo "📦 Setting up repository..."
if [ ! -d "$REPO_DIR" ]; then
    echo "   Cloning repository..."
    git clone "$REPO_URL" "$REPO_DIR"
    cd "$REPO_DIR"
    git checkout main
else
    echo "   Repository exists, pulling latest..."
    cd "$REPO_DIR"
    git pull origin main
fi

echo "✅ Repository ready at $REPO_DIR"
echo ""

# Step 3: Create environment configuration
echo "⚙️  Setting up environment..."
mkdir -p ~/.hermes

if [ ! -f ~/.hermes/.env ]; then
    echo "   Creating ~/.hermes/.env template..."
    cat > ~/.hermes/.env << 'ENVEOF'
# Hermes Agent Environment Configuration
# ⚠️  UPDATE THESE WITH YOUR ACTUAL API KEYS

# ── Synthia Gateway (PRIMARY — fastest real AI via OpenAI/Groq) ───────────────
# Run synthia-gateway on this VPS (see services/synthia-gateway/docker-compose.yml)
SYNTHIA_GATEWAY_URL=http://localhost:3000
# Secret key to authenticate against the gateway (set same value in gateway .env)
SYNTHIA_GATEWAY_KEY=
# Model to use (gpt-4o-mini is fast + cheap; llama-3.3-70b-versatile for free Groq)
SYNTHIA_MODEL=gpt-4o-mini

# ── OpenAI (used by Synthia Gateway or directly) ─────────────────────────────
# API key from https://platform.openai.com/api-keys
OPENAI_API_KEY=

# ── Groq (blazing fast free inference — alternative to OpenAI) ───────────────
# Free key from https://console.groq.com — use model: llama-3.3-70b-versatile
GROQ_API_KEY=

# ── Mercury Inception Labs (diffusion model, fallback) ────────────────────────
MERCURY_API_KEY=

# ── NVIDIA NIM (free Claude inference proxy, last resort) ────────────────────
NVIDIA_NIM_API_KEY=

# ── Telegram Bot ──────────────────────────────────────────────────────────────
TELEGRAM_BOT_TOKEN=

# ── API Server ────────────────────────────────────────────────────────────────
API_SERVER_PORT=8642
API_SERVER_HOST=0.0.0.0

# ── Other (optional) ─────────────────────────────────────────────────────────
ELEVENLABS_API_KEY=
GOOGLE_API_KEY=
VERCEL_TOKEN=
GITHUB_TOKEN=
SUPABASE_URL=
ENVEOF
    chmod 600 ~/.hermes/.env
    echo "   ⚠️  Created ~/.hermes/.env template"
    echo "   📝 IMPORTANT: Edit ~/.hermes/.env and add your actual API keys"
    echo "   📍 Command: nano ~/.hermes/.env"
else
    echo "   ~/.hermes/.env already exists (keeping existing)"
fi

echo "✅ Environment configured"
echo ""

# Step 4: Set up Python virtual environment + install dependencies
echo "📚 Setting up Python virtual environment..."
VENV_DIR="/opt/hermes-venv"

apt-get install -y python3-venv python3-full 2>/dev/null || true

if [ ! -d "$VENV_DIR" ]; then
    python3 -m venv "$VENV_DIR"
fi

# Activate venv for this script
source "$VENV_DIR/bin/activate"

pip install --upgrade pip setuptools wheel
pip install fastapi uvicorn pydantic httpx python-dotenv aiolimiter

# Install NIM proxy dependencies
cd "$REPO_DIR/services/nim-proxy"
pip install -q -r ../nim-proxy-requirements.txt 2>/dev/null || pip install fastapi uvicorn httpx pydantic python-dotenv openai
cd "$REPO_DIR"

echo "✅ Virtual environment ready at $VENV_DIR"
echo ""

# Step 5a: Deploy Synthia Gateway via Docker (fast AI inference)
echo "🤖 Setting up Synthia Gateway..."
if command -v docker &> /dev/null; then
    cd "$REPO_DIR/services/synthia-gateway"
    # Source env for OPENAI_API_KEY / GROQ_API_KEY / SYNTHIA_GATEWAY_KEY
    set -a; source /root/.hermes/.env 2>/dev/null || true; set +a
    docker compose up -d --build 2>&1 | tail -5
    echo "✅ Synthia Gateway running on port 3000"
    cd "$REPO_DIR"
else
    echo "⚠️  Docker not found — installing..."
    curl -fsSL https://get.docker.com | sh
    systemctl enable --now docker
    cd "$REPO_DIR/services/synthia-gateway"
    set -a; source /root/.hermes/.env 2>/dev/null || true; set +a
    docker compose up -d --build 2>&1 | tail -5
    echo "✅ Synthia Gateway running on port 3000"
    cd "$REPO_DIR"
fi
echo ""

# Step 5b: Create systemd services for auto-restart
echo "🔧 Setting up systemd services..."

# API Server service
sudo tee /etc/systemd/system/hermes-api.service > /dev/null << 'SVCEOF'
[Unit]
Description=Hermes Agent API Server (Port 8642)
After=network.target
StartLimitIntervalSec=60
StartLimitBurst=3

[Service]
Type=simple
User=root
WorkingDirectory=/opt/pauli-hermes-agent
EnvironmentFile=/root/.hermes/.env
ExecStart=/opt/hermes-venv/bin/python3 /opt/pauli-hermes-agent/api_server.py
Restart=on-failure
RestartSec=10
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
SVCEOF

# NIM Proxy service
sudo tee /etc/systemd/system/hermes-nim-proxy.service > /dev/null << 'SVCEOF'
[Unit]
Description=NVIDIA NIM Free Inference Proxy (Port 8082)
After=network.target
StartLimitIntervalSec=60
StartLimitBurst=3

[Service]
Type=simple
User=root
WorkingDirectory=/opt/pauli-hermes-agent/services/nim-proxy
EnvironmentFile=/root/.hermes/.env
ExecStart=/opt/hermes-venv/bin/python3 -m uvicorn server:app --host 0.0.0.0 --port 8082 --log-level warning
Restart=on-failure
RestartSec=10
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
SVCEOF

sudo systemctl daemon-reload
sudo systemctl enable hermes-api hermes-nim-proxy
echo "✅ Systemd services configured"
echo ""

# Step 6: Start services
echo "🚀 Starting services..."
sudo systemctl start hermes-api hermes-nim-proxy
sleep 2

# Check if services are running
if sudo systemctl is-active --quiet hermes-api; then
    echo "✅ API Server running on port 8642"
else
    echo "❌ API Server failed to start - check: sudo journalctl -u hermes-api -n 20"
fi

if sudo systemctl is-active --quiet hermes-nim-proxy; then
    echo "✅ NIM Proxy running on port 8082"
else
    echo "❌ NIM Proxy failed to start - check: sudo journalctl -u hermes-nim-proxy -n 20"
fi

echo ""

# Step 7: Deploy AionUI (full-featured mobile agent interface)
echo "📱 Setting up AionUI mobile interface (port 3001)..."
if bash "$REPO_DIR/services/aion-ui/setup.sh" 2>&1; then
    echo "✅ AionUI deployed on port 3001"
else
    echo "⚠️  AionUI setup had issues — run manually: bash $REPO_DIR/services/aion-ui/setup.sh"
fi
echo ""

echo "╔═══════════════════════════════════════════════════════════╗"
echo "║                  ✅ DEPLOYMENT COMPLETE                   ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""
echo "📱 Agent Interfaces:"
echo "   🎤 Voice Agent:  https://pauli-hermes-agent.vercel.app/agent"
echo "   💬 Full Chat UI: http://$VPS_IP:3001  (AionUI — open on phone)"
echo ""
echo "💻 API Endpoints (Your VPS):"
echo "   🤖 API Server: http://$VPS_IP:8642"
echo "   🚀 NIM Proxy:  http://$VPS_IP:8082"
echo "   🔗 Health:     curl http://$VPS_IP:8642/health"
echo ""
echo "📝 Test Commands:"
echo "   # Test API"
echo "   curl -X POST http://$VPS_IP:8642/api/chat \\"
echo "     -H 'Content-Type: application/json' \\"
echo "     -d '{\"message\": \"remember Alice is a PM\", \"providers\": {\"nvidia\": true, \"mercury\": true}}'"
echo ""
echo "   # Use free Claude Code"
echo "   ANTHROPIC_BASE_URL=http://$VPS_IP:8082 ANTHROPIC_API_KEY=dummy claude"
echo ""
echo "📋 Service Management:"
echo "   # Check status"
echo "   sudo systemctl status hermes-api hermes-nim-proxy"
echo ""
echo "   # View logs"
echo "   sudo journalctl -u hermes-api -f"
echo "   sudo journalctl -u hermes-nim-proxy -f"
echo ""
echo "   # Restart services"
echo "   sudo systemctl restart hermes-api hermes-nim-proxy"
echo ""
echo "🔐 Firewall (if needed):"
echo "   sudo ufw allow 8642  # API Server"
echo "   sudo ufw allow 8082  # NIM Proxy"
echo ""
echo "📚 Documentation:"
echo "   - VPS Setup:    https://github.com/executiveusa/pauli-hermes-agent/blob/main/VPS_SETUP.md"
echo "   - Claude Docs:  https://github.com/executiveusa/pauli-hermes-agent/blob/main/CLAUDE.md"
echo "   - GitHub Repo:  https://github.com/executiveusa/pauli-hermes-agent"
echo ""
echo "🎉 Ready to use! Open the voice agent on your phone."
echo ""
