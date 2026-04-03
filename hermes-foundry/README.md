# Hermes Foundry (Rust-native product platform)

Hermes Foundry is a multi-tenant Agent Backend-as-a-Service for revenue-first AI delivery.

## Phase 2 gap-focused productization
- Kept existing Rust workspace scaffold.
- Added product layer: tenant config, onboarding, white-label metadata, usage/billing snapshots, support/admin endpoints, policy files, deployment manifest, versioned API, and operator runbooks.

## Architecture (normalized)
- **Governance**: Paperclip bridge (`foundry-paperclip`)
- **Execution OS**: API/orchestration/policy/runs/billing/subagents
- **Memory**: LightRAG adapter boundary (`foundry-memory`)
- **Tool Surface**: MCP contracts (`foundry-mcp`)
- **Worker Plane**: coding/browser scaffolds
- **Product Backend**: Appwrite binding layer
- **Interfaces**: dashboard bootstrap + channel abstractions

## API foundations
- Health: `/healthz`, `/readyz`, `/version`
- Deploy safety: `/deployment-manifest.json`, `/openapi.json`
- Versioned operations: `/v1/...`
- Operator/admin: `/onboarding/tenant`, `/tenants`, `/admin/...`

## Startup
```bash
cp .env.example .env
cargo run -p foundry-api
```

## Smoke test
```bash
bash infra/scripts/smoke.sh
```

## Rollback plan
1. Roll back image tag.
2. Restore latest verified DB snapshot.
3. Re-run smoke checks.
4. Replay safe queued runs only.
