#!/usr/bin/env bash
set -euo pipefail
BASE="${1:-http://localhost:8788}"

curl -fsS "$BASE/healthz" >/tmp/foundry-healthz.json
curl -fsS "$BASE/dashboard/bootstrap" >/tmp/foundry-bootstrap.json
curl -fsS "$BASE/missing-secrets" >/tmp/foundry-secrets.json
curl -fsS "$BASE/provider-profiles" >/tmp/foundry-profiles.json

curl -fsS -X POST "$BASE/tasks" -H 'content-type: application/json' -d '{
  "tenant_id":"tenant_demo",
  "owner":"ops",
  "objective":"Generate sprint prep",
  "input":{"offer":"AI Systems Sprint"},
  "approval_gate":"standard"
}' >/tmp/foundry-task.json

echo "Smoke passed"
