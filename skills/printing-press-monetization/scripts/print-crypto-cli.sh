#!/usr/bin/env bash
# Print the crypto intelligence CLI using CLI Printing Press
# Wraps CoinGecko + altFINS + ChangeNOW APIs into one compound CLI

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SERVICE_DIR="$SCRIPT_DIR/../services/crypto-cli"
OUTPUT_DIR="${OUTPUT_DIR:-$SERVICE_DIR}"

echo "=== Crypto Intelligence CLI Builder ==="
echo "Prints CoinGecko + altFINS + ChangeNOW CLI via CLI Printing Press"
echo ""

# Check for CLI Printing Press
if ! command -v printing-press &>/dev/null && ! command -v go &>/dev/null; then
  echo "ERROR: printing-press or go not found. Install from github.com/mvanhorn/cli-printing-press"
  exit 1
fi

mkdir -p "$OUTPUT_DIR"

# Step 1: Print CoinGecko CLI (free public API)
echo "[1/4] Printing CoinGecko CLI..."
printing-press \
  --api "https://api.coingecko.com/api/v3" \
  --name "crypto-intel" \
  --output "$OUTPUT_DIR" \
  --lang go \
  --mcp \
  --skill \
  || echo "  Note: Run 'printing-press' manually if above fails"

echo ""

# Step 2: Extend with altFINS trading signals
echo "[2/4] Adding altFINS trading signal commands..."
if [ -f "$SERVICE_DIR/compound-commands.go" ]; then
  echo "  compound-commands.go found — will be merged at build time"
else
  echo "  Copy compound-commands.go to $SERVICE_DIR manually"
fi

# Step 3: Add license key auth
echo "[3/4] Adding LemonSqueezy license key auth..."
if [ -f "$SERVICE_DIR/auth.go" ]; then
  echo "  auth.go found — license key gating ready"
else
  echo "  Copy auth.go to $SERVICE_DIR manually"
fi

# Step 4: Build and release
echo "[4/4] Building crypto-intel binary..."

if command -v goreleaser &>/dev/null && [ -f "$SERVICE_DIR/goreleaser.yaml" ]; then
  cd "$SERVICE_DIR"
  goreleaser release --clean --snapshot
  echo "  Built: $SERVICE_DIR/dist/"
elif command -v go &>/dev/null && [ -d "$OUTPUT_DIR" ]; then
  cd "$OUTPUT_DIR"
  go build -o crypto-intel . 2>/dev/null || echo "  Source not generated yet — run printing-press first"
fi

echo ""
echo "=== Next Steps ==="
echo "1. Test: ./crypto-intel prices --top 10"
echo "2. Add free API keys:"
echo "   export COINGECKO_API_KEY=''  # free, no key needed"
echo "   export ALTFINS_API_KEY='your-key'  # free tier: 1000 credits/mo"
echo "   export CHANGENOW_API_KEY='your-key'  # free, no limits"
echo "3. List on LemonSqueezy: app.lemonsqueezy.com"
echo "4. Price: \$49/month, \$399/year"
