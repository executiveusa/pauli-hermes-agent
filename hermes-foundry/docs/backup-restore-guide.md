# Backup / Restore Guide

## Backups
- DB: nightly dump (recommended cron: 02:00 UTC)
- Artifacts: daily snapshot with 30-day retention
- Policy files: versioned in git

## Restore order
1. Restore DB
2. Restore artifacts
3. Restore env secrets
4. Run health checks
5. Replay queued runs if needed

## Current non-HA limitations
- Single API instance
- Single Redis
- No automated cross-region failover yet
