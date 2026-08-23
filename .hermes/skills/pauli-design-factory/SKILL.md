---
name: pauli-design-factory
description: Route brand, interface, product UI, motion, visual-system, brand-book, and design-review work into the canonical Pauli Design Factory through MCP or its REST contract instead of recreating design logic inside Hermes.
---

# Pauli Design Factory

## Canonical authority
Repository: `executiveusa/brand-kit-builder-`

The Design Factory owns brand/design workflow law, ICM stages, design skills, quality gates, guardians, and design deliverables. Hermes orchestrates and calls it; Hermes does not fork its design rules into a competing local system.

## When to invoke
Use this skill when the user asks to:
- create or revise a brand identity or brand system;
- design a website, product UI, component, interactive experience, style guide, brand book, social template, or visual campaign;
- improve interface taste, typography, color, motion, interaction, responsive behavior, or accessibility;
- run Collins-level review, Design Guardian, animation review, design proof, or the Design Gauntlet;
- choose among design directions or prototype alternatives.

## Walk test first
Before producing design work, discover factory capabilities and identify:
`PURPOSE → OWNER → STAGE → READ SCOPE → WRITE SCOPE → REQUIRED SKILLS → OUTPUT → PASS BAR → PROOF → NEXT HANDOFF`.

With MCP, call `design_factory_capabilities` first on a cold start or when routing is uncertain.

## Preferred transport: local MCP
If `PAULI_BRAND_STUDIO_ROOT` points to a checkout containing `interfaces/mcp/server.mjs`, use the baked `brand-studio` MCP tool surface.

Bootstrap command:
```bash
bash .hermes/mcp/bootstrap.sh
```

The bootstrap conditionally registers the MCP server through mcp2cli when the Design Factory checkout is present.

Core MCP tools:
- `design_factory_capabilities` — discover ICM stages, gates, skills, laws and live surfaces.
- `design_factory_normalize` — compile the request to canonical `interface-request.v1` plus idempotency key.
- `design_factory_run_reference` — prove local ICM orchestration and receipts through G4, stopping at G5 human approval. Never present reference-worker output as finished creative work.

## REST fallback
The Design Factory defines a model-agnostic REST contract in `interfaces/rest/API.md`. Use it only when a deployed base URL has been explicitly configured and verified (for example `PAULI_BRAND_STUDIO_URL`). Do not invent a URL or claim REST is live from documentation alone.

Expected contract families include:
- `/v1/projects`
- `/v1/work-orders`
- `/v1/projects/{id}/manifest`
- `/v1/projects/{id}/render/*`
- `/v1/context/compile`
- `/v1/guardians/run`
- `/v1/knowledge/*`

## Routing law
1. Keep canonical design truth in Design Factory ICM/manifests.
2. Ask the factory which skills apply; do not preload every design skill.
3. Surface useful options to the user in plain language. Example: "Prototype three directions", "Apply Apple-style interaction principles", "Audit motion", "Run Collins-level critique".
4. Strategy precedes styling.
5. Motion is opt-in when it improves hierarchy, feedback, continuity, or comprehension; do not animate by default.
6. Builder and critic must be independent for final validation.
7. G4 requires proof; G5 requires human approval before publish.
8. Never duplicate client brand truth, secrets, or approved manifests into Hermes memory as a second source of truth.

## User-facing recommendation behavior
When a project is classified, tell the user the 2–5 most relevant capabilities and why. Default to the smallest sufficient workflow.

Examples:
- Brand/marketing site → Brand Discovery → Collins-Level → Prototype where material → Design Engineering → Guardian → Gauntlet → Proof.
- Product UI → Krug/clarity → Apple Design → Pick UI Library → Prototype critical interaction → Design Engineering → Guardian → Gauntlet.
- Existing motion-heavy UI → Improve Animations audit → prioritized repair → Review Animations → Guardian → Gauntlet.
- Component → Pick UI Library first; prototype only if the interaction is consequential; animate only when justified.

## Failure behavior
- If MCP is not registered, check `PAULI_BRAND_STUDIO_ROOT` and rerun `.hermes/mcp/bootstrap.sh`.
- If the repo checkout is absent, use GitHub only for inspection and tell the operator the local MCP cannot run until a checkout is available.
- If a REST URL is configured but not verified, fail closed instead of claiming a callable API.
- Never bypass Design Factory approval/provenance gates because another harness can write files directly.
