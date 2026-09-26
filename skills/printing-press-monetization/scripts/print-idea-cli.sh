#!/usr/bin/env bash
# Print the IdeaBrowser CLI using CLI Printing Press (browser-sniff mode)
# Wraps ideabrowser.com's API for startup idea research

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SERVICE_DIR="$SCRIPT_DIR/../services/idea-cli"
OUTPUT_DIR="${OUTPUT_DIR:-$SERVICE_DIR}"

echo "=== IdeaBrowser CLI Builder ==="
echo "Sniffs ideabrowser.com API and generates a research CLI"
echo ""

mkdir -p "$OUTPUT_DIR"

# Browser-sniff mode: discover the API from the live site
echo "[1/3] Sniffing IdeaBrowser API..."
printing-press \
  --sniff "https://ideabrowser.com" \
  --name "idea" \
  --output "$OUTPUT_DIR" \
  --lang go \
  --mcp \
  --skill \
  || echo "  Note: Run 'printing-press --sniff https://ideabrowser.com' manually"

echo ""

# Add compound commands
echo "[2/3] Adding compound research commands..."
if [ -f "$SERVICE_DIR/compound-commands.go" ]; then
  echo "  compound-commands.go found — ready to build"
else
  echo "  Add compound-commands.go to $SERVICE_DIR"
fi

# Build
echo "[3/3] Building idea binary..."
if command -v go &>/dev/null && [ -d "$OUTPUT_DIR" ]; then
  cd "$OUTPUT_DIR"
  go build -o idea . 2>/dev/null || echo "  Source not generated yet"
fi

echo ""
echo "=== Next Steps ==="
echo "1. Test: ./idea trending --today"
echo "2. Add auth: ./idea auth --key YOUR_LEMONSQUEEZY_KEY"
echo "3. List on LemonSqueezy at \$29/month"
echo "4. Landing page: ideabrowser-cli.dev"
