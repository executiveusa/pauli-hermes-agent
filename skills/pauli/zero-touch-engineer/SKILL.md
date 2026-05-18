---
name: zero-touch-engineer
description: Zero-touch engineering execution loop for repo discovery, planning, implementation, testing, verification, and safe escalation.
version: 1.0.0
required_environment_variables: []
---

# Zero-Touch Engineer

## triggers

- repo scan
- full overview
- implement and verify
- fix and retest
- deploy prep

## when_to_use

Use for software delivery tasks that need discovery, planning, coding, testing, verification, and a clean handoff.

## when_not_to_use

Do not use for casual brainstorming, copy-only writing, or speculative architecture with no execution path.

## required_tools

- terminal
- file read/write tools
- git tools
- test runner

## required_env

- none

## context_budget

- default to 3 companion skills max
- add more only for complex repo tasks

## safety_gates

- never print secrets
- stop after 3 repeated failed fix loops
- require approval for production deploys or irreversible deletes

## workflow

1. Scan repo state, docs, and local conventions before edits.
2. Write a concrete plan artifact when the task is large.
3. Implement in small increments.
4. Run checks after each meaningful edit set.
5. Fix regressions before moving on.
6. Verify the claimed outcome with tests or explicit blockers.

## output_contract

- concise status
- files changed
- tests run
- blockers only when external

## tests

- repo build or compile step passes
- task-specific checks pass
