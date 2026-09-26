---
name: proof-first-revenue-loop
description: Route active SELL/revenue work into the standalone Hermes ICM proof-first commercial workflow.
---

# Proof-First Revenue Loop — Runtime Adapter

This skill is a thin runtime adapter. It is not the policy source of truth.

## Load
1. `../../../icm/AGENTS.md`
2. `../../../icm/instructions/HERMES.md`
3. `../../../icm/instructions/PROOF_FIRST_REVENUE_LOOP.md`
4. `../../../icm/context/TASK_PROFILES.md`
5. only the current opportunity/test evidence required by the active state

## Execute
Follow the ICM pipeline declared in `../../../icm/workflows/proof-first-revenue-loop/CONTEXT.md`.

Do not skip receipts or human gates. Do not create a fourth workstream. Do not add software when the current bottleneck is selling, approval, testing, or measurement.

## Write
Persist run-specific outputs to the product/memory surfaces named by the ICM contracts. Never rewrite factory policy at runtime.

## Done
Return the completion contract from `../../../icm/instructions/PROOF_FIRST_REVENUE_LOOP.md`, including evidence and the exact next human decision if one exists.
