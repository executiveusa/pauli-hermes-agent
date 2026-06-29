---
name: pauli-flywheel-orchestrator
description: Flywheel and Ralphy orchestration lane for planning, bead decomposition, execution loops, and review checkpoints.
version: 1.0.0
required_environment_variables: []
---

# Pauli Flywheel Orchestrator

## triggers

- flywheel
- beads
- task swarm
- PRD
- ralphy
- orchestrate

## when_to_use

Use when the task needs structured decomposition, repeated execute-review loops, or a PRD-driven work queue.

## when_not_to_use

Do not use for one-off trivial edits that do not benefit from bead/task decomposition.

## required_tools

- terminal
- git
- planning docs

## required_env

- none

## context_budget

- pair with one domain skill plus one debugging or review skill when needed

## safety_gates

- preserve repo boundaries
- approval before destructive commands
- do not pretend Ralphy is active if only the vendor repo is present

## workflow

1. Check `scripts/pauli/flywheel-status.ps1` to verify Ralphy readiness.
2. Break the work into beads or PRD tasks.
3. Execute in small loops with verification after each stage.
4. Record blockers as environment, credential, or platform issues instead of inventing fake success.

## output_contract

- orchestration mode
- bead/task list
- readiness/blockers
- next safe command

## tests

- flywheel status command returns valid JSON
- readiness clearly distinguishes installed, partial, and missing states
