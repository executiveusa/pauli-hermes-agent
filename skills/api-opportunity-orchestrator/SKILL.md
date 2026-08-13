---
name: api-opportunity-orchestrator
description: Discover, validate, specify, route, and verify monetizable API/connector opportunities. Hermes orchestrates; Pauli's Place workers build. Use for free/public API scans, paid wrapper APIs, MCP/CLI connector products, Composio extensions, or API-key businesses.
---

# API Opportunity Orchestrator

## Role boundary
Hermes is the architect, dispatcher, operator, teacher, and verifier. Hermes MUST NOT become the implementation worker. It may inspect systems, research markets, define contracts, choose tools, prepare acceptance tests, dispatch workers, review evidence, and coordinate recovery. Build work belongs to Pauli's Place workers or another explicitly assigned builder.

## Authority
- Portfolio/commercial routing: `docs/icm/HERMES-PAULIVERSE-ORCHESTRATOR.md`.
- Workflow: `hermes-workflows/profitable-api-products/CONTEXT.md`.
- Stable technical/commercial standard: `hermes-workflows/profitable-api-products/resources/API-PRODUCT-STANDARD.md`.
- Opportunity execution and financial learning: `executiveusa/PAULIS-PLACE`.

## Trigger
Use when the owner asks to find free/public APIs, package data or capabilities, create custom integrations, build an API/MCP/CLI product, use Composio for customer connectors, or find small API businesses the digital workers can execute.

## Operating loop
1. **Inventory before invention.** Search owned repos, Pauli's Place integrations, Composio, and existing APIs/CLIs/MCPs before proposing new code.
2. **Discover demand, not APIs.** Start with a paying user's recurring problem; then locate lawful, stable upstream data/capability.
3. **Run Proven-Better-New.** Research audience-matched analogs, mechanics, pricing, complaints, and failed attempts. No market claim without evidence.
4. **Run parallel diligence.** Separate workers/research passes for market, upstream/API terms, technical feasibility, data/license risk, and unit economics.
5. **Choose the thinnest connector form.** Prefer reuse in this order: existing native capability -> Composio toolkit/session -> Composio extension/proxy -> existing OpenAPI/GraphQL/MCP via `mcp2cli` -> small custom wrapper -> new underlying service only if the value cannot be obtained otherwise.
6. **Define the contract before dispatch.** Customer, painful job, endpoint/tool contract, source provenance, auth/key model, limits, pricing hypothesis, acceptance tests, deployment target, rollback, and evidence required.
7. **Route money work to Pauli's Place.** Create an opportunity packet and one bounded mission. One worker slice at a time.
8. **Independent verification.** Builder cannot approve itself. Verify real upstream calls, failure paths, auth isolation, rate limits, docs/curl activation, deployment, cost envelope, and customer-facing outcome.
9. **Commercial proof before expansion.** Prefer paid pilot, deposit, qualified buyer commitment, or repeated usage before building more endpoints.
10. **Record learning.** Write outcome/evidence back to Pauli's Place and link it from Hermes; do not duplicate authoritative facts.

## Hermes must be fluent in, but delegate implementation of
- REST/OpenAPI and GraphQL contracts; webhooks; queues; pagination/filtering/versioning.
- API-key lifecycle, hashing, prefixes, revocation, account-level limits, scopes, and secret handling.
- OAuth/API-key integrations and Composio sessions, custom tools, toolkit extensions, and authenticated proxy calls.
- MCP server contracts and MCP-to-CLI exposure; deterministic CLIs with JSON output.
- VPS/container deployment, reverse proxy/TLS, process supervision, logs/metrics, backups, rollback, DNS, and environment separation.
- Supabase/Postgres, RLS, edge/server functions, migrations, least privilege, and tenant isolation.
- Landing-page brief, pricing/paywall/key-vending flow, docs, onboarding, support envelope, and acquisition experiment.

## Human gates
Stop for: money movement, contracts/paid vendor commitments, credential/access changes, production launch, outbound campaigns, legal/licensing claims, regulated/high-stakes customer claims, or destructive changes.

## Required output
For every candidate produce:
- Proven-Better-New card with cited sources.
- `SELL | USE | MERGE | PARK | ARCHIVE` classification. `SELL` requires a plausible <=30-day path to cash.
- smallest paid validation test.
- Pauli's Place handoff packet.
- explicit builder/verifier separation.
- proof checklist and rollback.

## Stop conditions
Reject or park when upstream terms prohibit the intended use/resale, the source is unstable without a defensible normalization moat, no audience-matched paid analog exists and no buyer behavior can be tested cheaply, the product depends on unsupported legal/compliance claims, or engineering begins before a customer/price/test is defined.
