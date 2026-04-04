#!/bin/bash
# Quick Infisical Integration Test Script
# Run this on your VPS after docker compose up -d

set -e

API_URL="${1:-http://localhost:8642}"
INFISICAL_TOKEN="${INFISICAL_TOKEN}"

echo "==============================================="
echo "Hermes Agent - Infisical Integration Tests"
echo "==============================================="
echo ""
echo "API URL: $API_URL"
echo "Token: ${INFISICAL_TOKEN:0:20}..."
echo ""

# Test 1: Health check
echo "[1/5] Testing API health..."
if curl -s "$API_URL/health" | grep -q "ok"; then
    echo "✅ API is running"
else
    echo "❌ API health check failed"
    exit 1
fi
echo ""

# Test 2: Fetch a secret (anthropic-api-key)
echo "[2/5] Fetching 'anthropic-api-key' from Infisical..."
SECRET_RESPONSE=$(curl -s "$API_URL/v1/secrets/anthropic-api-key" \
    -H "Authorization: Bearer $INFISICAL_TOKEN")
echo "Response:"
echo "  $SECRET_RESPONSE" | jq '.' 2>/dev/null || echo "  $SECRET_RESPONSE"
echo ""

# Test 3: List secrets with prefix
echo "[3/5] Listing secrets with 'supabase' prefix..."
LIST_RESPONSE=$(curl -s "$API_URL/v1/secrets?prefix=supabase" \
    -H "Authorization: Bearer $INFISICAL_TOKEN")
echo "Response:"
echo "  $LIST_RESPONSE" | jq '.' 2>/dev/null || echo "  $LIST_RESPONSE"
echo ""

# Test 4: Check cache status
echo "[4/5] Checking local cache status..."
CACHE_RESPONSE=$(curl -s "$API_URL/v1/cache-status")
echo "Response:"
echo "  $CACHE_RESPONSE" | jq '.' 2>/dev/null || echo "  $CACHE_RESPONSE"
echo ""

# Test 5: Rotate secrets (clear cache)
echo "[5/5] Rotating secrets (clearing cache)..."
ROTATE_RESPONSE=$(curl -s -X POST "$API_URL/v1/rotate-secrets")
echo "Response:"
echo "  $ROTATE_RESPONSE" | jq '.' 2>/dev/null || echo "  $ROTATE_RESPONSE"
echo ""

echo "==============================================="
echo "All tests completed! ✅"
echo "==============================================="
