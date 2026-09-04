# Project Registry — PARÉ-centered operating map

This file is the minimum cross-repo map for the owner's personal Hermes. It is not a substitute for reading each repository's local ICM/HiCM state before acting.

## Identity boundaries

### Personal Hermes
- Canonical repo: `executiveusa/pauli-hermes-agent`
- Role: owner's permanent orchestrator/governor across projects.
- Must not inherit client identity, credentials, memory, or runtime state from client agents.

### MACS Digital Media / Agent Max
- Company name: **MACS Digital Media**.
- Canonical portal repo: `executiveusa/macs-agent-portal`.
- Agent identity: **Agent Max**.
- Its Hermes/runtime is client/business-specific and is separate from the owner's personal Hermes.
- Never merge personal-Hermes memory, credentials, or authority into Agent Max by convenience.

## Core product spine

### PARÉ
- Canonical repo: `executiveusa/PARE`.
- Role: sovereign AI studio, project system, runtime-neutral execution surface, owner-controlled files/data/infrastructure.
- Current operating path: Studio -> sovereign runtime -> provider/agent -> SSE/artifact -> verification.
- PARÉ is the primary product. New frameworks are admitted only when they strengthen PARÉ rather than create a competing control plane.

### Loop Engineering / Gauntlet
- Role: build/verify/release discipline, not a separate customer product.
- Repository-local run state is authoritative for active work.
- Evidence outranks builder claims.

### ICM / HiCM
- Role: portable project intelligence and routing contract.
- Every managed repo must expose enough local ICM/HiCM state for a cold agent to identify owner, task, boundaries, proof, output location, and human gate without reading the full repository.

## Connected execution systems

### Pauli Orca / cloud coding runtime
- Role: disposable or cloud execution factory for bounded coding/build/test work.
- Hermes decides what should happen; Orca/workers execute bounded work; GitHub remains source of truth.

### Buffer Blaster
- Role: social/content production and distribution engine.
- Integrates with PARÉ as a capability/product surface, not as a competing orchestrator.

### Jarvis
- Role: owner-facing second-brain/control interface where useful.
- Must consume canonical project intelligence rather than become a second source of truth.

### Pauli Studio Control Plane
- Repo: `executiveusa/pauli-studio-control-plane`.
- Role: control-plane/governance reference. Reuse contracts where they strengthen Hermes/PARÉ; avoid duplicating authority.

## Cross-repo synchronization rule

Before consequential work in any registered repo:
1. Read that repo's local ICM/HiCM router/state.
2. Run the mandatory repository walk test.
3. Check recent commits/open PRs in PARÉ, personal Hermes, and any directly dependent repo named in the task.
4. Record only material dependency changes; do not sweep unrelated repos for novelty.
5. If another repo changed a contract this task depends on, update the active plan/evidence before building.

## Admission filter

A new project/framework belongs on the active path only if it materially improves at least one of:
- PARÉ capability or reliability;
- revenue/distribution;
- reusable project intelligence;
- owner sovereignty/security;
- Loop/Gauntlet verification quality.

Otherwise classify it PARK and do not let it interrupt the active release.