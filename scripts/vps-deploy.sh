#!/usr/bin/env bash
# ============================================================
# Hermes Agent — VPS Deployment Script
# ============================================================
# Installs and starts Hermes via Docker Compose on a fresh VPS.
# Tested on Ubuntu 22.04 / 24.04 (Hostinger, Contabo, Hetzner).
#
# Usage (run as root or sudo):
#   bash scripts/vps-deploy.sh
#
# Re-deploy after a code update:
#   bash scripts/vps-deploy.sh --update
# ============================================================

set -euo pipefail

GREEN='\033[0;32m'; YELLOW='\033[0;33m'; CYAN='\033[0;36m'
RED='\033[0;31m'; BOLD='\033[1m'; NC='\033[0m'

UPDATE_ONLY="${1:-}"

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_DIR"

info()  { echo -e "${CYAN}→${NC} $*"; }
ok()    { echo -e "${GREEN}✓${NC} $*"; }
warn()  { echo -e "${YELLOW}⚠${NC} $*"; }
die()   { echo -e "${RED}✗${NC} $*" >&2; exit 1; }
hr()    { echo -e "${BOLD}────────────────────────────────────────────${NC}"; }

hr
echo -e "${BOLD}  ⚡ Hermes Agent — VPS Deployment${NC}"
hr

# ── 1. Check .env ────────────────────────────────────────────
if [ ! -f ".env" ]; then
  if [ -f ".env.vps.example" ]; then
    cp .env.vps.example .env
    warn ".env created from .env.vps.example — fill in your API keys before continuing."
    echo ""
    echo "  Edit with: nano .env"
    echo "  Then re-run this script."
    exit 0
  else
    die ".env not found. Create it from .env.vps.example first."
  fi
fi
ok ".env found"

# ── 2. Install Docker (if missing) ──────────────────────────
if ! command -v docker &>/dev/null; then
  info "Installing Docker…"
  curl -fsSL https://get.docker.com | bash
  systemctl enable --now docker
  ok "Docker installed"
else
  ok "Docker $(docker --version | awk '{print $3}' | tr -d ',')"
fi

# ── 3. Install Docker Compose plugin (if missing) ────────────
if ! docker compose version &>/dev/null 2>&1; then
  info "Installing Docker Compose plugin…"
  apt-get -yq install docker-compose-plugin 2>/dev/null || \
    pip3 install docker-compose 2>/dev/null || \
    warn "Could not install docker compose — install manually: https://docs.docker.com/compose/install/"
else
  ok "Docker Compose $(docker compose version --short 2>/dev/null || echo 'OK')"
fi

# ── 4. Update nginx config for the domain ────────────────────
DOMAIN="$(grep '^DASHBOARD_ORIGIN=' .env | head -1 | sed 's|DASHBOARD_ORIGIN=||;s|https://||;s|http://||;s|/.*||' || true)"
if [ -n "$DOMAIN" ] && [ "$DOMAIN" != "YOUR_DOMAIN" ]; then
  if grep -q 'YOUR_DOMAIN' nginx/hermes.conf; then
    sed -i "s/YOUR_DOMAIN/$DOMAIN/g" nginx/hermes.conf
    ok "nginx config updated for $DOMAIN"
  fi
fi

# ── 5. Pull latest code (update mode only) ──────────────────
if [ "$UPDATE_ONLY" = "--update" ]; then
  info "Pulling latest code…"
  git pull --ff-only || warn "git pull failed — deploy may use stale code"
fi

# ── 6. Build image ───────────────────────────────────────────
info "Building Hermes image…"
docker compose build --no-cache hermes
ok "Build complete"

# ── 7. Start / restart services ─────────────────────────────
info "Starting services…"
docker compose up -d --remove-orphans
ok "Services started"

# ── 8. Wait for health check ─────────────────────────────────
info "Waiting for Hermes API server…"
for i in $(seq 1 20); do
  if curl -sf http://127.0.0.1:8642/health &>/dev/null; then
    ok "Hermes API server is up"
    break
  fi
  sleep 2
  if [ "$i" = "20" ]; then
    warn "API server health check timed out — check logs: docker compose logs hermes"
  fi
done

# ── 9. Systemd service (auto-start on reboot) ────────────────
SYSTEMD_UNIT="/etc/systemd/system/hermes.service"
if [ ! -f "$SYSTEMD_UNIT" ] && command -v systemctl &>/dev/null; then
  info "Creating systemd service…"
  cat > "$SYSTEMD_UNIT" <<EOF
[Unit]
Description=Hermes Agent
After=docker.service
Requires=docker.service

[Service]
Type=oneshot
RemainAfterExit=yes
WorkingDirectory=${REPO_DIR}
ExecStart=/usr/bin/docker compose up -d
ExecStop=/usr/bin/docker compose down
TimeoutStartSec=120

[Install]
WantedBy=multi-user.target
EOF
  systemctl daemon-reload
  systemctl enable hermes
  ok "Systemd service enabled (auto-starts on reboot)"
fi

# ── Done ──────────────────────────────────────────────────────
hr
echo -e "${GREEN}${BOLD}  ✓ Hermes deployed successfully!${NC}"
hr
echo ""
echo "  Dashboard:    http://31.220.58.212:9000"
echo "  API:          http://31.220.58.212:8642/v1"
echo "  Health:       http://31.220.58.212:8642/health"
echo "  Coolify:      http://31.220.58.212:8000"
echo "  Supabase:     http://31.220.58.212:3001"
echo ""
echo "  View logs:    docker compose logs -f hermes"
echo "  Stop:         docker compose down"
echo "  Update:       bash scripts/vps-deploy.sh --update"
echo ""

# Check if CORS is set for dashboard access
if grep -q 'YOUR_DOMAIN\|localhost' .env; then
  warn "Set DASHBOARD_ORIGIN in .env to your real dashboard URL for browser CORS to work."
fi
