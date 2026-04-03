# Hermes Foundry (Rust-native core)

Hermes Foundry is a multi-tenant execution OS for revenue-first AI service delivery.

## Implementation plan (v1)
1. Rust API gateway + stable routes
2. Tenant-aware run ledger + billing meter
3. Approval and policy engine
4. Provider router (`/v1/llm/chat`)
5. MCP capability contracts
6. Coding/browser worker scaffolds
7. Dashboard bootstrap payload (Pauli Vibe compatible)

## Normalized architecture (prose)
- **Layer A Governance**: `foundry-paperclip` (goals, budgets, approvals bridge)
- **Layer B Execution OS**: `foundry-core`, `foundry-runs`, `foundry-policy`, `foundry-billing`, `foundry-subagents`
- **Layer C Memory**: `foundry-memory` (LightRAG adapter boundary)
- **Layer D Tool Surface**: `foundry-mcp` contracts + API parity
- **Layer E Worker Plane**: `foundry-coder`, `foundry-browser`
- **Layer F Product Backend**: `foundry-appwrite` bindings
- **Layer G Interfaces**: API bootstrap for Pauli Vibe cockpit + Telegram bridge readiness

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
- Deploy with versioned containers/tags.
- Keep previous image as `foundry-api:previous`.
- On regression: route traffic back to previous tag, keep schema backward-compatible, and replay queued approvals/runs from ledger.
