---
name: pauli-vercel-ops
description: Vercel deployment triage and repair for build failures, 404s, routing issues, and environment drift.
version: 1.0.0
required_environment_variables:
  - VERCEL_TOKEN
---

# Pauli Vercel Ops

## triggers

- vercel
- 404
- build failed
- deployment failed

## when_to_use

Use for Vercel project inspection, deployment diagnosis, and routing/env repair plans.

## when_not_to_use

Do not mutate production settings without approval.

## required_tools

- terminal
- Vercel CLI or API

## required_env

- `VERCEL_TOKEN`

## context_budget

- 3 companion skills max

## safety_gates

- read/list first
- approval before production mutations

## workflow

1. Inspect projects and latest deployments.
2. Classify the failure mode.
3. Propose or execute the smallest safe fix.

## output_contract

- failing deployment summary
- likely root cause
- next safe fix

## tests

- deployment lookup works
- fix path references the correct project and route
