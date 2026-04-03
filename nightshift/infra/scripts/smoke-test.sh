#!/usr/bin/env bash
set -euo pipefail
# Agent MAXX — Smoke test: verify all services are responding

PASS=0; FAIL=0
check() {
  local name=$1 url=$2
  if curl -sf "$url" >/dev/null 2>&1; then
    echo "  ✅ $name"
    ((PASS++))
  else
    echo "  ❌ $name ($url)"
    ((FAIL++))
  fi
}

echo "🐕 Agent MAXX — Smoke Test"
echo ""
check "hermes-api /healthz"      "http://localhost:8642/healthz"
check "hermes-api /readyz"       "http://localhost:8642/readyz"
check "hermes-api /version"      "http://localhost:8642/version"
check "dashboard  /healthz"      "http://localhost:3000/healthz"
check "dashboard  landing"       "http://localhost:3000/"
check "dashboard  cockpit"       "http://localhost:3000/dashboard"
check "API overview"             "http://localhost:8642/api/dashboard/overview"
check "API repos"                "http://localhost:8642/api/repos"
check "API runs"                 "http://localhost:8642/api/runs"
check "API providers"            "http://localhost:8642/api/providers"

echo ""
echo "Results: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ] && echo "🎉 All green!" || echo "⚠️  Some checks failed"
exit "$FAIL"
