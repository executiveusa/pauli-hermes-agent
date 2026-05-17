---
name: pauli-coolify-ops
description: Coolify and Hostinger VPS deployment operations with staging-first safety gates and rollback planning.
version: 1.0.0
required_environment_variables:
  - COOLIFY_BASE_URL
  - COOLIFY_API_KEY
---

# Pauli Coolify Ops

## triggers

- deploy
- coolify
- hostinger
- rollback
- vps

## when_to_use

Use for Coolify service discovery, deploy preparation, staging rollout, health checks, and rollback planning.

## when_not_to_use

Do not use for first production deploys without explicit approval.

## required_tools

- terminal
- web or API client
- docker or compose validation
- browser-harness

## required_env

- `COOLIFY_BASE_URL`
- `COOLIFY_API_KEY`

## context_budget

- 4 skills max including deployment companions

## safety_gates

- production approval required
- no DNS mutation
- no destructive service deletion
- use dashboard/browser control only when API access is missing or incomplete

## workflow

1. Validate secret presence without printing values.
2. Discover service and environment state.
3. Validate compose and healthcheck definitions.
4. Prepare staging deploy path and rollback notes.

## output_contract

- redacted service status
- deploy command
- rollback command
- blockers

## tests

- service discovery succeeds or reports exact missing credential
- healthcheck path resolves
