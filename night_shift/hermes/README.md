# Hermes Night Shift v1 Scaffold

This directory contains a clean-room MVP scaffold for **Hermes Night Shift v1**.

## 1) Implementation plan
1. Control plane (`hermes-api`) with typed routes, MCP mount, and workflow service boundaries.
2. Persistence with Postgres/Redis and SQLAlchemy + Alembic migration baseline.
3. Provider router abstraction (`/v1/llm/chat`) with pluggable adapters.
4. Repo catalog + PRD batch + approval loop.
5. Coding session queue contract and worker-coder execution skeleton.
6. Browser-run queue contract and worker-browser safety state machine.
7. Dashboard text-first shell with route panels.
8. Appwrite provisioning hooks (optional service).
9. Sub-agent lifecycle registry with strict scoped policy fields.
10. Smoke tests + runbook + rollback plan.

## 2) Exact file tree
```text
(see committed tree under night_shift/hermes)
```

## 3) docker-compose.yml
See `docker-compose.yml`.

## 4) .env.example
See `.env.example`.

## 5) Database schema / models
SQLAlchemy entities are defined in `apps/hermes-api/app/models/*.py` and mirrored in migration scaffold `migrations/versions/0001_initial.py`.

## 6) FastAPI routes
Implemented under `apps/hermes-api/app/routes/` plus `/deployment-manifest.json` in `app/main.py`.

## 7) MCP tool schemas
Mounted at `/mcp` via `app/mcp_mount.py` with handlers in `app/mcp_tools/`.

## 8) Provider-router interface and adapters
`POST /v1/llm/chat` in `apps/provider-router/app/routes.py`; adapters in `providers/`.

## 9) Coding worker job contracts
`apps/worker-coder/app/session_runner.py` defines required fields and state contract.

## 10) Browser worker job contracts
`apps/worker-browser/app/browser_session.py` and `app/state_machine.py` define inputs and states.

## 11) Appwrite provisioning module
`apps/hermes-api/app/services/appwrite_service.py` provides provisioning record flow.

## 12) Dashboard structure
Text-first Next.js panels under `apps/hermes-dashboard/src/app/*`.

## 13) Smoke tests
`infra/scripts/smoke-test.sh` checks API, provider-router, dashboard endpoints.

## 14) Startup instructions
```bash
cd night_shift/hermes
cp .env.example .env
make up
make smoke
```

## 15) Rollback plan
1. `docker compose down`
2. restore previous volume snapshots or `backup-*.sql`
3. revert to prior git commit/tag
4. `docker compose up -d`
