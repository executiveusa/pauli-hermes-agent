#!/usr/bin/env bash
set -euo pipefail

ENV_FILE="${1:-/mnt/e/THE PAULI FILES/master.env}"
HERMES_HOME_DIR="${HERMES_HOME:-$HOME/.hermes}"
PROFILE_NAME="${HERMES_PROFILE:-pauli}"

if [[ ! -f "$ENV_FILE" ]]; then
  echo "Env file not found: $ENV_FILE" >&2
  exit 1
fi

PROFILE_DIR="$HERMES_HOME_DIR/profiles/$PROFILE_NAME"
PROFILE_ENV="$PROFILE_DIR/.env"
ROOT_ENV="$HERMES_HOME_DIR/.env"

mkdir -p "$PROFILE_DIR"

CONTENT="$(cat "$ENV_FILE")"
for REQUIRED in "API_SERVER_ENABLED=true" "API_SERVER_CORS_ORIGINS=*"; do
  KEY="${REQUIRED%%=*}"
  if ! grep -q "^${KEY}=" <<<"$CONTENT"; then
    CONTENT="${CONTENT}"$'\n'"${REQUIRED}"
  fi
done

printf '%s' "$CONTENT" > "$PROFILE_ENV"
printf '%s' "$CONTENT" > "$ROOT_ENV"

echo "Synced redacted env source into $PROFILE_ENV and $ROOT_ENV"
