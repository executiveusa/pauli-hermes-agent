# Public API Contract (v1)

Base endpoints:
- `POST /v1/tasks`
- `GET /v1/runs/{id}`
- `POST /v1/approvals/{id}/resolve`
- `GET /v1/usage/{tenant_id}`
- `GET /v1/mcp/contracts`
- `POST /v1/llm/chat`

Compatibility:
- breaking changes require `/v2` namespace
