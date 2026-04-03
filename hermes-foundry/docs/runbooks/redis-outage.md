# Runbook: Redis Outage
1. Degrade to DB-backed polling mode.
2. Pause non-critical long workflows.
3. Recover Redis and replay pending events.
