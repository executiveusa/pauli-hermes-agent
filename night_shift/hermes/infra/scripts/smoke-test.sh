#!/usr/bin/env bash
set -euo pipefail
curl -fsS http://localhost:8000/healthz
curl -fsS http://localhost:8080/healthz
curl -fsS http://localhost:3000 >/dev/null
echo "smoke ok"
