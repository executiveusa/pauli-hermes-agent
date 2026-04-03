#!/usr/bin/env bash
set -euo pipefail
BASE="${1:-http://localhost:8788}"

curl -fsS "$BASE/healthz" >/tmp/foundry-healthz.json
curl -fsS "$BASE/readyz" >/tmp/foundry-readyz.json
curl -fsS "$BASE/version" >/tmp/foundry-version.json
curl -fsS "$BASE/deployment-manifest.json" >/tmp/foundry-deploy.json
curl -fsS "$BASE/openapi.json" >/tmp/foundry-openapi.json
curl -fsS "$BASE/dashboard/bootstrap" >/tmp/foundry-bootstrap.json
curl -fsS "$BASE/missing-secrets" >/tmp/foundry-secrets.json
curl -fsS "$BASE/provider-profiles" >/tmp/foundry-profiles.json

curl -fsS -X POST "$BASE/onboarding/tenant" -H 'content-type: application/json' -d '{
  "tenant_id":"tenant_demo",
  "org_name":"Demo Org",
  "mode":"internal",
  "app_name":"Demo Foundry",
  "support_email":"ops@example.com"
}' >/tmp/foundry-onboard.json

curl -fsS -X POST "$BASE/v1/tasks" -H 'content-type: application/json' -d '{
  "tenant_id":"tenant_demo",
  "owner":"ops",
  "objective":"Generate sprint prep",
  "input":{"offer":"AI Systems Sprint"},
  "approval_gate":"standard"
}' >/tmp/foundry-task.json

echo "Smoke passed"
