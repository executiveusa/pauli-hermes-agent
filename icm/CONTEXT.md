# ICM — Standalone Hermes Control Plane

## Purpose

This `icm/` tree is the model-agnostic operating layer embedded in the Hermes runtime repo. It does not replace Hermes source code or the upstream development guide. It tells an agent what operating contract to load and where evidence/state belongs.

PAULIS-PLACE remains the cross-system business source of truth when connected. Local ICM files make standalone Hermes portable and safe when that wider context is unavailable.

## Portable spine

- `instructions/` — WHAT to do: Hermes role and executable operating workflows.
- `context/` — WHERE/HOW boundaries work: repo map, task profiles, event/envelope contracts.
- `memory/` — WHAT happened: write-once run evidence, decisions and promoted patterns.

`.hermes/skills/` contains runtime adapters. A skill should route into ICM rather than duplicate canonical policy.

## Load protocol

1. Read this file.
2. Read the relevant folder `CONTEXT.md`.
3. Load the single instruction for the current task.
4. Load only context files that instruction names.
5. Load prior memory only when history is needed.
6. Execute using existing Hermes tools/plugins/skills.
7. Write run evidence to memory/product surfaces; never mutate factory policy at runtime.

For proof-first revenue work:

1. `instructions/HERMES.md`
2. `instructions/PROOF_FIRST_REVENUE_LOOP.md`
3. `context/TASK_PROFILES.md`
4. `context/ENVELOPES.md`
5. only current prospect/test evidence

## Factory vs product

**Factory:** instructions, context, stable skill adapters.

**Product:** opportunity state, proof artifacts, approval packets, test results, receipts, decisions and promoted patterns.

## Human gate

Research/drafts/internal reversible work may run inside approved autonomy. External sending, publishing, spending, production changes, destructive/admin actions and commitments remain gated unless an explicit standing policy authorizes a narrower bounded action.

## Walk test

A cold agent must identify owner, instruction, required context, output location, proof condition and human gate from this file plus at most two additional reads. If it needs the full repo, the ICM layer has failed.