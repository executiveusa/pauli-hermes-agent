#!/bin/bash
set -e

PACKAGE_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DEV_DIR="${1:-/tmp/thepopebot.local}"
ENV_BACKUP="/tmp/env.$(uuidgen)"

HAS_ENV=false
if [ -f "$DEV_DIR/.env" ]; then
  mv "$DEV_DIR/.env" "$ENV_BACKUP"
  HAS_ENV=true
fi

rm -rf "$DEV_DIR"
mkdir -p "$DEV_DIR"
cd "$DEV_DIR"

node "$PACKAGE_DIR/bin/cli.js" init

sed -i '' "s|\"thepopebot\": \".*\"|\"thepopebot\": \"file:$PACKAGE_DIR\"|" package.json

rm -rf node_modules package-lock.json
npm install --install-links


if [ "$HAS_ENV" = true ]; then
  mv "$ENV_BACKUP" .env
  echo "Restored .env from previous build"
else
  npm run setup
fi
