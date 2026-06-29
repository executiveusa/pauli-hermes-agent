# Rollback Runbook

1. Keep immutable image tags per release.
2. Roll back Coolify service to last known good image tag.
3. Restore compatible env bundle and restart service.
4. Validate health endpoints and key integrations.
