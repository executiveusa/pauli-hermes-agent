#!/usr/bin/env bash
# Create Stripe products for all 5 revenue streams
# Requires: stripe CLI (stripe.com/docs/stripe-cli)

set -euo pipefail

if ! command -v stripe &>/dev/null; then
  echo "Install Stripe CLI: https://stripe.com/docs/stripe-cli"
  exit 1
fi

echo "=== Setting Up Stripe Products ==="
echo ""

# --- Stream 1: PPaaS ---
echo "[Stream 1] PPaaS — Printing Press as a Service"

PPAAS_PROD=$(stripe products create \
  --name "PPaaS" \
  --description "Printing Press as a Service — auto-generate production Go CLIs + MCP servers" \
  --format json | jq -r '.id')

echo "  Product: $PPAAS_PROD"

# Pay-per-use: $49 one-time
stripe prices create \
  --product "$PPAAS_PROD" \
  --currency usd \
  --unit-amount 4900 \
  --nickname "Starter — Per Generation" \
  | jq -r '"  Price (starter): " + .id'

# Pro monthly: $199/mo
stripe prices create \
  --product "$PPAAS_PROD" \
  --currency usd \
  --unit-amount 19900 \
  --recurring-interval month \
  --nickname "Pro — Unlimited Generations" \
  | jq -r '"  Price (pro monthly): " + .id'

# Enterprise: $999/mo
stripe prices create \
  --product "$PPAAS_PROD" \
  --currency usd \
  --unit-amount 99900 \
  --recurring-interval month \
  --nickname "Enterprise — Private Deployment + SLA" \
  | jq -r '"  Price (enterprise): " + .id'

echo ""

# --- Stream 4: MCP Marketplace ---
echo "[Stream 4] MCP Server Marketplace"

for API in Salesforce HubSpot Notion Linear Stripe QuickBooks Shopify Jira; do
  case "$API" in
    Salesforce|QuickBooks) PRICE=4900 ;;
    Shopify) PRICE=3900 ;;
    *) PRICE=2900 ;;
  esac

  PROD=$(stripe products create \
    --name "$API MCP Server" \
    --description "$API MCP server — AI-native access for Claude, Cursor, and Codex" \
    --format json | jq -r '.id')

  stripe prices create \
    --product "$PROD" \
    --currency usd \
    --unit-amount "$PRICE" \
    --recurring-interval month \
    --nickname "$API MCP — Monthly" \
    | jq -r "\"  $API MCP: \" + .id"
done

echo ""
echo "=== Done ==="
echo "Copy price IDs above into your PPaaS .env and MCP marketplace config."
echo "Next: stripe listen --forward-to localhost:3000/api/webhook"
