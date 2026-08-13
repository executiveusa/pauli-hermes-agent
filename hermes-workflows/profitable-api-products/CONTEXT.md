# Profitable API products — workflow contract

Outcome: discover, validate, hand off, and independently verify narrow API and connector products with a credible path to paid proof.

Hermes orchestrates. Pauli's Place assigns implementation workers and owns commercial execution. Hermes does not implement the product.

| Stage | Job | Gate |
|---|---|---|
| `00_inventory` | Search owned capabilities, Composio, upstream APIs, MCP/CLI options, and collisions | reuse checked before build |
| `01_discover` | Find recurring user jobs with lawful free or low-cost upstream inputs | named customer + painful job |
| `02_validate` | Run Proven-Better-New and analog/graveyard research | market evidence + source provenance |
| `03_specify` | Define smallest sellable contract, economics, security, interfaces, tests, and rollback | no code before spec |
| `04_handoff` | Route one bounded mission to Pauli's Place | explicit builder assignment |
| `05_verify` | Fresh-context verifier checks behavior, security, deployment, docs, cost, and evidence | builder cannot self-approve |
| `06_commercial_test` | Test the offer with real target buyers | proof before expansion |
| `07_learn` | Record demand, costs, failures, support burden, and reusable assets | one authoritative home per fact |

## Claude Cookbook patterns adapted
- Orchestrator-workers: Hermes decomposes work and assigns context-specific specialist passes.
- Prompt chaining: discovery -> validation -> specification -> handoff -> verification -> commercial test.
- Parallelization: market, technical, source/terms, security, and economics diligence run independently before synthesis.
- Routing: choose native, Composio, mcp2cli, or custom-wrapper paths by contract rather than habit.
- Evaluator-optimizer: a fresh verifier returns the largest evidence-backed gap; the builder repairs; verifier rechecks.
- Plan big, execute small: architecture may cover API + MCP + CLI + landing + operations, but workers receive one reversible slice at a time.
- Human gates: production, paid commitments, credentials, public claims, and outbound remain explicit approvals.
- Version and rollback: stable contracts are versioned and production changes require rollback evidence.

Source inspiration: `anthropics/claude-cookbooks`, especially `patterns/agents/orchestrator_workers.ipynb`, `patterns/agents/basic_workflows.ipynb`, `patterns/agents/evaluator_optimizer.ipynb`, and the managed-agent coordination, verification, gating, and rollback recipes. Adapt the patterns; do not copy the notebooks into Hermes.

## Collision rules
- Do not replace `mcp2cli`; use it for existing MCP/OpenAPI/GraphQL surfaces.
- Do not replace Pauli's Place Mission Control, Integrations Bus, Factory Kernel, or Pauli Signal.
- Do not create a second commercial authority in Hermes.
- Do not build a generic connector platform when Composio or an existing native integration already covers the needed action.

Stable references live in `resources/`. Run-specific research belongs in `runs/`. Each validated opportunity becomes its own Pauli's Place opportunity and mission.
