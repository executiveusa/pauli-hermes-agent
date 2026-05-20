#!/usr/bin/env bash
# AionUI Web Setup for Hermes VPS (Hostinger)
# Full-featured AI chat interface accessible on your phone
# Usage: bash services/aion-ui/setup.sh
# Access: http://31.220.58.212:3001

set -e

AION_DIR="/opt/aion-ui"
AION_REPO="https://github.com/iOfficeAI/AionUi.git"
AION_PORT=3001

echo "╔════════════════════════════════════════════════╗"
echo "║   AionUI — Mobile Agent Interface on VPS      ║"
echo "╚════════════════════════════════════════════════╝"

# --- Node.js 20 ---------------------------------------------------------------
if ! node --version 2>/dev/null | grep -q "v2[0-9]"; then
  echo "Installing Node.js 20..."
  curl -fsSL https://deb.nodesource.com/setup_20.x | bash -
  apt-get install -y nodejs
fi
echo "✅ Node $(node --version)"

# --- Bun ----------------------------------------------------------------------
if ! command -v bun &>/dev/null; then
  echo "Installing Bun..."
  curl -fsSL https://bun.sh/install | bash
fi
export PATH="$HOME/.bun/bin:$PATH"
echo "✅ Bun $(bun --version)"

# --- Clone / update AionUI ---------------------------------------------------
if [ ! -d "$AION_DIR/.git" ]; then
  echo "Cloning AionUI..."
  git clone "$AION_REPO" "$AION_DIR" --depth 1
else
  echo "Updating AionUI..."
  git -C "$AION_DIR" pull --ff-only 2>/dev/null || true
fi

cd "$AION_DIR"

# --- Configure environment ---------------------------------------------------
cat > "$AION_DIR/.env" << ENVEOF
PORT=$AION_PORT
NODE_ENV=production

# Route all AI through the free NIM proxy running on this VPS
# All Claude model names map to moonshotai/kimi-k2-thinking (free NVIDIA tier)
ANTHROPIC_BASE_URL=http://localhost:8082
ANTHROPIC_API_KEY=dummy

# If you have a Groq key for faster inference, uncomment:
# GROQ_API_KEY=gsk_...
# OPENAI_API_KEY=sk-...
ENVEOF

# Load additional keys from Hermes env if present
if [ -f /root/.hermes/.env ]; then
  set -a
  source /root/.hermes/.env 2>/dev/null || true
  set +a
  # Write actual keys into AionUI env
  [ -n "$GROQ_API_KEY" ] && sed -i "s|# GROQ_API_KEY=.*|GROQ_API_KEY=$GROQ_API_KEY|" "$AION_DIR/.env"
  [ -n "$OPENAI_API_KEY" ] && sed -i "s|# OPENAI_API_KEY=.*|OPENAI_API_KEY=$OPENAI_API_KEY|" "$AION_DIR/.env"
fi
echo "✅ Environment configured"

# --- Install dependencies ----------------------------------------------------
echo "Installing dependencies (this takes a minute first time)..."
bun install --frozen-lockfile 2>/dev/null || bun install
echo "✅ Dependencies installed"

# --- Build web renderer -------------------------------------------------------
echo "Building web UI..."
bun run build:renderer:web 2>/dev/null \
  || npx --yes vite build --config vite.renderer.config.ts
echo "✅ Web UI built"

# --- Systemd service ----------------------------------------------------------
cat > /etc/systemd/system/hermes-aionui.service << SVCEOF
[Unit]
Description=AionUI Agent Interface (Port $AION_PORT)
After=network.target
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
echo "✅ Systemd service enabled"

# --- Firewall -----------------------------------------------------------------
if command -v ufw &>/dev/null && ufw status | grep -q "Status: active"; then
  ufw allow "$AION_PORT/tcp" comment "AionUI" 2>/dev/null || true
  echo "✅ Firewall opened for port $AION_PORT"
fi

# --- Nginx reverse proxy (optional — if nginx is installed) -------------------
if command -v nginx &>/dev/null; then
  VPS_IP=$(curl -s ifconfig.me 2>/dev/null || echo "31.220.58.212")
  cat > /etc/nginx/sites-available/aionui << NGINXEOF
server {
    listen 3002;
    server_name _;

    location / {
        proxy_pass http://localhost:$AION_PORT;
        proxy_http_version 1.1;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_read_timeout 86400s;
    }
}
NGINXEOF
  ln -sf /etc/nginx/sites-available/aionui /etc/nginx/sites-enabled/aionui 2>/dev/null || true
  nginx -t 2>/dev/null && systemctl reload nginx 2>/dev/null || true
fi

# --- Verify -------------------------------------------------------------------
sleep 4

if systemctl is-active --quiet hermes-aionui; then
  VPS_IP=$(curl -s ifconfig.me 2>/dev/null || echo "31.220.58.212")
  echo ""
  echo "╔════════════════════════════════════════════════╗"
  echo "║           ✅ AionUI is LIVE                    ║"
  echo "╚════════════════════════════════════════════════╝"
  echo ""
  echo "📱 Open on your phone:"
  echo "   http://$VPS_IP:$AION_PORT"
  echo ""
  echo "💡 First-time login: set a password in AionUI's settings."
  echo ""
  echo "📋 Manage:"
  echo "   sudo systemctl status hermes-aionui"
  echo "   sudo journalctl -u hermes-aionui -f"
  echo "   sudo systemctl restart hermes-aionui"
else
  echo ""
  echo "❌ AionUI failed to start"
  echo "   sudo journalctl -u hermes-aionui -n 40"
fi
