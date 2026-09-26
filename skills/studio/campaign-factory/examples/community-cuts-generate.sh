#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
OUTPUT="${1:-$ROOT/campaign-output/community-cuts-for-kids/qr}"

python "$ROOT/skills/studio/campaign-factory/scripts/generate_qr.py" \
  --url "https://asc3nd.org" \
  --campaign "Community Cuts for Kids" \
  --output "$OUTPUT"

printf 'Generated unverified QR package at: %s\n' "$OUTPUT"
printf 'Next gate: destination test, software decode, physical scan, final-composition scan.\n'
