---
name: pauli-impeccable-design
description: High-end UI refinement skill focused on precision, motion restraint, accessibility, and polished interaction states.
version: 1.0.0
required_environment_variables: []
---

# Pauli Impeccable Design

## triggers

- polish UI
- accessibility
- motion
- interaction states

## when_to_use

Use after the core design direction is established and the work needs a production-quality polish pass.

## when_not_to_use

Do not use as a substitute for basic information architecture.

## required_tools

- file read
- browser-harness

## required_env

- none

## context_budget

- final-stage design companion only

## safety_gates

- preserve accessibility and reduced-motion support
- do not hide core controls

## workflow

1. Check focus, contrast, labels, and empty/error states.
2. Refine motion and affordances.
3. Verify the result interactively.

## output_contract

- polish checklist
- issues fixed
- residual risks

## tests

- keyboard path works
- focus states visible
- reduced-motion path preserved

## SYNTHIA™ UDEC Integration (v2.0 upgrade)

This skill now outputs a UDEC scorecard on every polish pass.
Load `skills/design-intelligence/_shared/udec-axes.md` for the scoring framework.

### Polish Pass → UDEC Score Protocol

After every polish pass, score the result across all 14 UDEC axes.
Floor: 8.5. Below 8.5 → iterate. MOT or ACC below 7.0 → rebuild that layer.

The quality gate is not the client's opinion. It is the UDEC score.
The client approves direction. UDEC approves delivery.
