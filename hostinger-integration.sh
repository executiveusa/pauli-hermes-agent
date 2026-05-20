#!/usr/bin/env bash
# Hostinger API Integration for Hermes Agent
# Add Hostinger API key to environment and test connectivity
# Usage: bash hostinger-integration.sh

HOSTINGER_API_KEY="${HOSTINGER_API_KEY:-$(grep -E '^HOSTINGER_API_KEY=' ~/.hermes/.env 2>/dev/null | tail -n1 | cut -d= -f2-)}"
if [ -z "$HOSTINGER_API_KEY" ]; then
  echo "❌ HOSTINGER_API_KEY not set. Add it to ~/.hermes/.env or export it."
  exit 1
fi

echo "╔═══════════════════════════════════════════════════════════╗"
echo "║       Hostinger API Integration for Hermes Agent          ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""

# Step 1: Add to environment
echo "📝 Adding Hostinger API key to ~/.hermes/.env..."
if grep -q "HOSTINGER_API_KEY" ~/.hermes/.env; then
    sed -i "s/HOSTINGER_API_KEY=.*/HOSTINGER_API_KEY=$HOSTINGER_API_KEY/" ~/.hermes/.env
    echo "   ✅ Updated existing key"
else
    echo "" >> ~/.hermes/.env
    echo "# Hostinger API" >> ~/.hermes/.env
    echo "HOSTINGER_API_KEY=$HOSTINGER_API_KEY" >> ~/.hermes/.env
    echo "   ✅ Added new key"
fi

source ~/.hermes/.env

echo ""
echo "🔐 Testing Hostinger API..."

# Test 1: Get domains
echo ""
echo "Test 1: Fetching domains..."
DOMAINS_RESPONSE=$(curl -s -X GET \
  "https://api.hostinger.com/v1/domains" \
  -H "Authorization: Bearer $HOSTINGER_API_KEY" \
  -H "Content-Type: application/json")

if echo "$DOMAINS_RESPONSE" | grep -q "error\|invalid\|401\|403"; then
    echo "   ❌ API Key Error:"
    echo "   $DOMAINS_RESPONSE"
else
    echo "   ✅ Domains retrieved:"
    echo "$DOMAINS_RESPONSE" | jq '.' 2>/dev/null || echo "$DOMAINS_RESPONSE"
fi

# Test 2: Get VPS instances
echo ""
echo "Test 2: Fetching VPS instances..."
VPS_RESPONSE=$(curl -s -X GET \
  "https://api.hostinger.com/v1/vps" \
  -H "Authorization: Bearer $HOSTINGER_API_KEY" \
  -H "Content-Type: application/json")

if echo "$VPS_RESPONSE" | grep -q "error\|invalid\|401\|403"; then
    echo "   ❌ API Key Error:"
    echo "   $VPS_RESPONSE"
else
    echo "   ✅ VPS instances retrieved:"
    echo "$VPS_RESPONSE" | jq '.' 2>/dev/null || echo "$VPS_RESPONSE"
fi

# Test 3: Get account info
echo ""
echo "Test 3: Fetching account information..."
ACCOUNT_RESPONSE=$(curl -s -X GET \
  "https://api.hostinger.com/v1/account" \
  -H "Authorization: Bearer $HOSTINGER_API_KEY" \
  -H "Content-Type: application/json")

if echo "$ACCOUNT_RESPONSE" | grep -q "error\|invalid\|401\|403"; then
    echo "   ❌ API Key Error:"
    echo "   $ACCOUNT_RESPONSE"
else
    echo "   ✅ Account info retrieved:"
    echo "$ACCOUNT_RESPONSE" | jq '.' 2>/dev/null || echo "$ACCOUNT_RESPONSE"
fi

echo ""
echo "╔═══════════════════════════════════════════════════════════╗"
echo "║                    Integration Summary                    ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""
echo "✅ Hostinger API Key: CONFIGURED"
echo "📍 Location: ~/.hermes/.env"
echo ""
echo "Available Hostinger API Operations:"
echo "  • List VPS instances"
echo "  • Manage domains"
echo "  • Configure DNS records"
echo "  • Monitor account resources"
echo "  • Retrieve billing info"
echo ""
echo "Example: Get all VPS instances"
echo '  curl -X GET "https://api.hostinger.com/v1/vps" \'
echo '    -H "Authorization: Bearer $HOSTINGER_API_KEY"'
echo ""
