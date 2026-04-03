# System Overview

Hermes Foundry is a multi-tenant Agent Backend-as-a-Service.

## Permanent core (Rust-first)
- API + orchestration: `foundry-api`
- Policy: `foundry-policy`
- Routing: `foundry-router`
- Run ledger + usage: `foundry-runs`, `foundry-billing`
- Setup/onboarding: `foundry-setup`
- Tool contracts: `foundry-mcp`

## Temporary bridge
- Existing Python Hermes stack remains available as legacy bridge while Rust services mature.
