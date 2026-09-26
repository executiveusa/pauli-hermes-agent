#!/usr/bin/env bash
# Create LemonSqueezy products for CLI subscriptions
# Uses LemonSqueezy REST API: https://docs.lemonsqueezy.com/api

set -euo pipefail

LS_API_KEY="${LEMONSQUEEZY_API_KEY:-}"
LS_STORE_ID="${LEMONSQUEEZY_STORE_ID:-}"

if [ -z "$LS_API_KEY" ] || [ -z "$LS_STORE_ID" ]; then
  echo "Set LEMONSQUEEZY_API_KEY and LEMONSQUEEZY_STORE_ID first"
  echo "Get them at: app.lemonsqueezy.com/settings/api"
  exit 1
fi

LS_API="https://api.lemonsqueezy.com/v1"
AUTH="Authorization: Bearer $LS_API_KEY"

create_product() {
  local name="$1" desc="$2" price_monthly="$3" price_yearly="$4"

  echo "Creating: $name"

  PRODUCT=$(curl -sf -X POST "$LS_API/products" \
    -H "$AUTH" -H "Content-Type: application/vnd.api+json" \
    -d "{
      \"data\": {
        \"type\": \"products\",
        \"attributes\": {
          \"store_id\": $LS_STORE_ID,
          \"name\": \"$name\",
          \"description\": \"$desc\",
          \"status\": \"draft\"
        }
      }
    }")

  PRODUCT_ID=$(echo "$PRODUCT" | jq -r '.data.id')
  echo "  Product ID: $PRODUCT_ID"

  # Monthly variant
  curl -sf -X POST "$LS_API/variants" \
    -H "$AUTH" -H "Content-Type: application/vnd.api+json" \
    -d "{
      \"data\": {
        \"type\": \"variants\",
        \"attributes\": {
          \"product_id\": $PRODUCT_ID,
          \"name\": \"Monthly\",
          \"price\": $price_monthly,
          \"is_subscription\": true,
          \"interval\": \"month\",
          \"interval_count\": 1,
          \"has_license_keys\": true,
          \"license_activation_limit\": 3
        }
      }
    }" | jq -r '"  Monthly variant: " + .data.id'

  # Yearly variant (2 months free)
  curl -sf -X POST "$LS_API/variants" \
    -H "$AUTH" -H "Content-Type: application/vnd.api+json" \
    -d "{
      \"data\": {
        \"type\": \"variants\",
        \"attributes\": {
          \"product_id\": $PRODUCT_ID,
          \"name\": \"Annual (2 months free)\",
          \"price\": $price_yearly,
          \"is_subscription\": true,
          \"interval\": \"year\",
          \"interval_count\": 1,
          \"has_license_keys\": true,
          \"license_activation_limit\": 3
        }
      }
    }" | jq -r '"  Annual variant: " + .data.id'

  echo ""
}

echo "=== Setting Up LemonSqueezy Products ==="
echo ""

# Crypto Intelligence CLI
create_product \
  "Crypto Intelligence CLI" \
  "Cross-exchange arbitrage, trading signals, portfolio P&L — all in your terminal. Powered by CoinGecko + altFINS + ChangeNOW." \
  4900 \
  39900

# IdeaBrowser CLI
create_product \
  "IdeaBrowser CLI" \
  "Research startup ideas from the command line. 1000+ pre-validated ideas with Reddit + search trend data." \
  2900 \
  23900

echo "=== Done ==="
echo "Activate products at: app.lemonsqueezy.com/products"
echo "Set LEMONSQUEEZY_SIGNING_SECRET in your .env for webhook validation"
