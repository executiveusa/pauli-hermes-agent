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

# NVIDIA NIM (Free Claude Code Inference)
# Get key from: https://build.nvidia.com
NVIDIA_NIM_API_KEY=nvapi-your-key-here

# Mercury Inception Labs
# Get key from: https://mercury.ai
MERCURY_API_KEY=sk_your-key-here

# Telegram Bot
# Create bot at: https://t.me/BotFather
TELEGRAM_BOT_TOKEN=your-bot-token-here

# API Server
API_SERVER_PORT=8642
API_SERVER_HOST=0.0.0.0
API_SERVER_CORS_ORIGINS=https://pauli-hermes-agent.vercel.app,http://localhost:3000,http://localhost:8642

# Other APIs (optional, add as needed)
OPENAI_API_KEY=
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

# Step 4: Install Python dependencies
echo "📚 Installing Python dependencies..."
pip install --upgrade pip setuptools wheel

# Install API server dependencies
pip install fastapi uvicorn pydantic httpx

# Install NIM proxy dependencies
cd "$REPO_DIR/services/nim-proxy"
pip install -q -r ../nim-proxy-requirements.txt
cd "$REPO_DIR"

echo "✅ Dependencies installed"
echo ""

# Step 5: Create systemd services for auto-restart
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
ExecStart=/usr/bin/python3 /opt/pauli-hermes-agent/api_server.py
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
ExecStart=/usr/bin/python3 -m uvicorn server:app --host 0.0.0.0 --port 8082 --log-level warning
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
echo "╔═══════════════════════════════════════════════════════════╗"
echo "║                  ✅ DEPLOYMENT COMPLETE                   ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""
echo "🎤 Voice Agent (Web UI):"
echo "   📱 Phone: https://pauli-hermes-agent.vercel.app/agent"
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
