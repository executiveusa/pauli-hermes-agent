#!/usr/bin/env bash
set -euo pipefail

BASE_URL="${1:-http://localhost:8787}"

echo "🔎 Running smoke checks against ${BASE_URL}"
curl -fsS "${BASE_URL}/health" > /tmp/nightshift-health.json
curl -fsS "${BASE_URL}/setup/missing-secrets" > /tmp/nightshift-secrets.json
curl -fsS -X POST "${BASE_URL}/repos/scan" \
  -H 'content-type: application/json' \
  -d '{"repo_url":"https://github.com/example/repo","branch":"main"}' > /tmp/nightshift-repo.json
curl -fsS -X POST "${BASE_URL}/subagents" \
  -H 'content-type: application/json' \
  -d '{"name":"lint-agent","mission":"run checks","scope":["tests","lint"]}' > /tmp/nightshift-subagent.json

python - <<'PY'
import json
from pathlib import Path

health = json.loads(Path('/tmp/nightshift-health.json').read_text())
assert health["status"] == "ok"
repo = json.loads(Path('/tmp/nightshift-repo.json').read_text())
assert repo["status"] == "cataloged"
print("✅ Smoke checks passed")
PY
