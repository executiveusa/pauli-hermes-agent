#!/usr/bin/env bash
set -euo pipefail
# Agent MAXX — Backup Postgres to timestamped SQL dump

BACKUP_DIR="${BACKUP_DIR:-/data/backups}"
CONTAINER="${1:-$(docker compose ps -q postgres)}"
DB="${POSTGRES_DB:-maxx_db}"
USER="${POSTGRES_USER:-maxx}"
TS=$(date +%Y%m%d_%H%M%S)
FILE="$BACKUP_DIR/maxx_${TS}.sql.gz"

mkdir -p "$BACKUP_DIR"
echo "🐕 Backing up $DB → $FILE"
docker exec "$CONTAINER" pg_dump -U "$USER" "$DB" | gzip > "$FILE"
echo "✅ Backup complete: $(du -h "$FILE" | cut -f1)"

# Prune backups older than 14 days
find "$BACKUP_DIR" -name "maxx_*.sql.gz" -mtime +14 -delete 2>/dev/null
echo "🗑️  Old backups pruned (14d retention)"
