# ICM Workflows

This folder composes existing Hermes skills, tools, scripts, and governance into end-to-end operating procedures.

A skill answers **how to perform a capability**. A workflow answers **how to deliver a complete human outcome safely across multiple capabilities**.

## ICM contract

Every workflow follows the Interpretable Context Methodology:

1. **Interpreter** — establish mode, outcome, target, authority, constraints, proof, commercial value, and rollback.
2. **Context** — load only approved systems, credentials metadata, runbooks, current telemetry, and named prior outputs.
3. **Method** — execute a bounded sequence, preserve receipts, stop at gates, independently verify, and return a human review packet.

## Required lifecycle

```text
HUMAN INTAKE
  -> READ-ONLY DISCOVERY
  -> RISK CLASSIFICATION
  -> PLAN + ROLLBACK
  -> HUMAN AUTHORIZATION
  -> BOUNDED EXECUTION
  -> INDEPENDENT VERIFICATION
  -> HUMAN REVIEW
  -> LEARNING / RUNBOOK UPDATE
```

No workflow may let the same agent propose, execute, and approve a consequential production change.

## Initial workflow set

- `sovereign-vps-operator/` — Hostinger VPS operations control plane.
- `sovereign-vps-operator/workflows/01-onboard-and-baseline.md` — inventory, ownership, exposure, and recovery baseline.
- `sovereign-vps-operator/workflows/02-monitor-and-maintain.md` — scheduled health, patch, capacity, certificate, and cost checks.
- `sovereign-vps-operator/workflows/03-backup-and-restore.md` — provider-independent encrypted backups and restore drills.
- `sovereign-vps-operator/workflows/04-deploy-and-rollback.md` — controlled deployment with live verification and rollback.
- `sovereign-vps-operator/workflows/05-incident-response.md` — detect, contain, recover, verify, and review incidents.
- `sovereign-vps-operator/workflows/06-client-data-lifecycle.md` — isolate, retain, export, revoke, and delete client data safely.

## Shared governance

Use these existing repository capabilities rather than duplicating them:

- `scripts/hostinger_bridge.py` for read-only Hostinger inventory.
- `hardened-longrun-subagent-harness` for durable missions, checkpoints, bounded concurrency, and side-effect receipts.
- `vibe-client-factory` for outcome contracts, authority classes, independent review, Judge verdicts, ownership, and client proof.
- Hermes cron for recurring checks and delivery.
- FREE MODE / local inference for low-risk monitoring and summarization, with external models only when policy permits.
- Existing deployment skills for Vercel or other approved targets; do not treat a deployment request as production proof.

## Workflow completion record

Every run ends with:

```text
DECISION
CHANGES
PROOF
STATUS
COMMERCIAL IMPACT
RISKS
ROLLBACK
NEXT
HUMAN APPROVAL
```

Allowed states: `PASS`, `PASS_WITH_DISPOSITIONS`, `HOLD`, `BLOCKED`, `NOT_RUN`.
