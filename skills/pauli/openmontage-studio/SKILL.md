---
name: pauli-openmontage-studio
description: OpenMontage-oriented studio orchestration for briefs, scripts, shot lists, asset planning, and dry-run render workflows.
version: 1.0.0
required_environment_variables: []
---

# Pauli OpenMontage Studio

## triggers

- montage
- shot list
- creative brief
- video studio
- render plan

## when_to_use

Use for creative planning and dry-run orchestration around OpenMontage-style pipelines.

## when_not_to_use

Do not claim a real render completed unless the render path actually ran.

## required_tools

- file read/write
- terminal

## required_env

- optional provider keys if render providers are enabled

## context_budget

- 4 studio skills max

## safety_gates

- dry-run by default
- paid render approval required

## workflow

1. Build brief, script, and shot list.
2. Select assets and provider path.
3. Produce a dry-run render plan and export metadata.

## output_contract

- brief
- shot list
- provider plan
- blockers

## tests

- dry-run metadata export succeeds
- no paid render on default path
