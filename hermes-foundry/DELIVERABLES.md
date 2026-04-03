# Hermes Foundry Deliverables Checklist

1. **Implementation plan** — added in `README.md` (Added)
2. **Normalized architecture diagram in prose** — `README.md` (Added)
3. **Rust workspace file tree** — represented by `Cargo.toml` members + crate dirs (Added)
4. **docker-compose.yml** — `docker-compose.yml` (Added)
5. **.env.example** — `.env.example` (Added)
6. **Migrations/schema definitions** — `migrations/001_init.sql` (Added)
7. **API routes** — `crates/foundry-api/src/main.rs` (Added)
8. **MCP/tool contracts** — `crates/foundry-mcp/src/lib.rs` (Added)
9. **Provider-router module** — `crates/foundry-router/src/lib.rs` (Added)
10. **Run ledger + billing meter** — `crates/foundry-runs`, `crates/foundry-billing` (Added)
11. **Coding worker scaffolding** — `crates/foundry-coder` (Added)
12. **Browser worker scaffolding** — `crates/foundry-browser` (Added)
13. **Dashboard integration plan using Pauli Vibe cockpit** — `apps/cockpit-web/README.md` (Added)
14. **Paperclip integration plan** — `crates/foundry-paperclip` + README architecture boundary (Added)
15. **Appwrite integration plan** — `crates/foundry-appwrite` + API provision route (Added)
16. **Missing-secrets flow** — `crates/foundry-setup` + `/missing-secrets` route (Added)
17. **Smoke tests** — `infra/scripts/smoke.sh` (Added)
18. **Startup instructions** — `README.md` (Added)
19. **Rollback plan** — `README.md` (Added)

## Existing implementation handling
- Kept existing Hermes Python stack untouched as bridge/runtime reference.
- Added Rust-native Foundry workspace alongside existing modules instead of duplicating existing Python features.
