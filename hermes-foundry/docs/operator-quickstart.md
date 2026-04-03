# Operator Quick Start

1. Open dashboard bootstrap payload: `GET /dashboard/bootstrap`.
2. Check setup blockers: `GET /missing-secrets`.
3. Onboard tenant: `POST /onboarding/tenant`.
4. Submit task: `POST /v1/tasks`.
5. Watch run: `GET /v1/runs/{id}`.
6. Resolve approvals: `POST /v1/approvals/{id}/resolve`.
7. Review usage: `GET /v1/usage/{tenant_id}`.
