#!/usr/bin/env bash
# AionUI Web Setup for Hermes VPS
# Deploys AionUI (https://github.com/iOfficeAI/AionUi) on port 3001
# Routes AI through the NIM proxy (free inference) or Synthia Gateway
# Usage: bash services/aion-ui/setup.sh

set -e

AION_DIR="/opt/aion-ui"
AION_REPO="https://github.com/iOfficeAI/AionUi.git"
PORT=3001

echo "╔════════════════════════════════════════════╗"
echo "║   AionUI — Mobile Agent Interface Setup    ║"
echo "╚════════════════════════════════════════════╝"

# Ensure Bun is installed (AionUI uses Bun)
if ! command -v bun &>/dev/null; then
  echo "Installing Bun..."
  curl -fsSL https://bun.sh/install | bash
  export PATH="$HOME/.bun/bin:$PATH"
fi

# Ensure Node 20+ for Vite/React
if ! command -v node &>/dev/null || [[ $(node -e "process.exit(parseInt(process.versions.node)<20?1:0)" 2>&1; echo $?) == "1" ]]; then
  echo "Installing Node.js 20..."
  curl -fsSL https://deb.nodesource.com/setup_20.x | bash -
  apt-get install -y nodejs
fi

# Clone or update AionUI
if [ ! -d "$AION_DIR" ]; then
  echo "Cloning AionUI..."
  git clone "$AION_REPO" "$AION_DIR" --depth 1
else
  echo "Updating AionUI..."
  git -C "$AION_DIR" pull --ff-only
fi

cd "$AION_DIR"

# Write AionUI environment config
cat > "$AION_DIR/.env" << 'ENVEOF'
# AionUI — Hermes VPS Configuration
PORT=3001
NODE_ENV=production

# Route AI through free NIM proxy (no API key needed)
ANTHROPIC_BASE_URL=http://localhost:8082
ANTHROPIC_API_KEY=dummy

# Or use Synthia Gateway (set these to use OpenAI/Groq instead)
# OPENAI_BASE_URL=http://localhost:3000/v1
# OPENAI_API_KEY=<your-synthia-gateway-key>
ENVEOF

echo "Installing dependencies..."
bun install --frozen-lockfile 2>/dev/null || bun install

echo "Building web UI..."
bun run build:renderer:web 2>/dev/null || npx vite build --config vite.renderer.config.ts

# Create systemd service
cat > /etc/systemd/system/hermes-aionui.service << SVCEOF
[Unit]
Description=AionUI Agent Interface (Port $PORT)
After=network.target hermes-nim-proxy.service
Wants=hermes-nim-proxy.service

[Service]
Type=simple
User=root
WorkingDirectory=$AION_DIR
EnvironmentFile=$AION_DIR/.env
ExecStart=$(which bun) run server:start
Restart=on-failure
RestartSec=10
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
SVCEOF

systemctl daemon-reload
systemctl enable hermes-aionui
systemctl restart hermes-aionui

sleep 3

if systemctl is-active --quiet hermes-aionui; then
  VPS_IP=$(curl -s ifconfig.me 2>/dev/null || echo "31.220.58.212")
  echo ""
  echo "✅ AionUI running!"
  echo ""
  echo "📱 Open on your phone:"
  echo "   http://$VPS_IP:$PORT"
  echo ""
  echo "🔥 Firewall (if blocked):"
  echo "   sudo ufw allow $PORT"
else
  echo "❌ AionUI failed to start"
  echo "   Check: sudo journalctl -u hermes-aionui -n 30"
fi
