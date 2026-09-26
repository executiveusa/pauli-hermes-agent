# ICM Portable Persona Factory

## Purpose

Allow Hermes to spawn isolated Pi-based sub-agents from portable persona templates without sharing owner memory, private workflows, customer data, credentials, or unrelated context.

## Separation model

```text
Hermes Governor
  └─ Persona Factory
      ├─ Fanny instance — Tenant A
      ├─ Fanny instance — Tenant B
      └─ Future persona — Tenant C
```

Hermes owns the factory contract and governance. Each spawned agent owns only its tenant-scoped runtime state.

## ICM spawn sequence

### Interpreter
Validate tenant, measurable job, approved sources, constraints, proof, commercial classification, authority, retention, rollback, and human owner.

### Context
Load the selected persona package plus the new tenant configuration. Do not load Hermes owner memory or another agent instance.

### Method

1. Validate the portable persona manifest.
2. Generate a unique tenant and instance namespace.
3. Provision isolated memory, artifacts, secrets references, and durable task queue.
4. Bind only approved tools and data sources.
5. Run leakage, authority, and acceptance tests.
6. Start in sandbox mode.
7. Require human approval for assisted or production mode.
8. Record a spawn receipt and export manifest.

## Non-negotiable invariants

- deny by default;
- tenant filter before retrieval;
- one instance cannot query another instance;
- persona templates contain no runtime secrets or customer records;
- learned information remains instance-scoped unless deliberately sanitized and promoted through review;
- spawning and training are different operations;
- builders cannot approve production activation;
- every side effect has an idempotency key and receipt;
- suspension and export must remain possible.

## Commercial modes

- **Template license:** portable persona and guided setup materials.
- **Assisted configuration:** paid workflow mapping, categories, examples, tests, and onboarding.
- **Managed training:** paid review of corrections, lesson proposals, regression checks, and version upgrades.
- **MAXX Operations:** monitoring, support, integrations, evidence, and continuous governed improvement.

## Export classes

1. **Template export:** reusable persona only.
2. **Customer configuration export:** categories, branding, authority, and approved integrations.
3. **Customer memory export:** customer-owned training examples and lessons.
4. **Audit export:** receipts, approvals, and version history.

These exports remain separate so the generic persona never absorbs private tenant information.
