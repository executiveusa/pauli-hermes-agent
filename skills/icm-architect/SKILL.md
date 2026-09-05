---
name: icm-architect
description: "Structure or restructure a workspace, repo, or described process into an ICM (Interpretable Context Methodology) agent-runnable folder pipeline — numbered stages, CONTEXT.md contracts, factory-vs-product separation. Auto-activates on structuring requests so it never needs manual invocation."
version: 1.0.0
author: Pauli Hermes Agent
license: MIT
platforms: [linux, macos, windows]
triggers:
  - make this an ICM
  - ICM this
  - ICM-ify
  - build me a workspace
  - structure this for agents
  - restructure this repo for agents
  - turn this into an agent-runnable pipeline
  - organize this into a folder pipeline
  - map this org as context
  - audit this repo for ICM
  - add this to the icm map
  - /icm-architect
entry_point: /icm-architect [build|restructure] [target path or description]
metadata:
  hermes:
    tags: [icm, workspace-structuring, folder-as-agent, methodology, restructure, scaffolding]
    related_skills: [icm-organizer, icm-engineering-governor]
    capabilities: [workspace-design, repo-restructure, contract-writing, walk-test-validation]
    activation_style: automatic-on-keyword
---

# ICM Architect — Runtime Adapter

Thin runtime adapter. Canonical policy — identity, the ten invariants,
Build mode, Restructure mode, the walk test, guardrails, and templates —
lives in `../../icm/instructions/ICM_ARCHITECT.md`. This file exists only
so Hermes recognizes the trigger phrases above and lazy-loads that policy
without the user having to invoke a skill by name.

## Auto-Activation (No Setup Required)

Hermes detects structuring/restructuring language — "make this an ICM",
"ICM this", "structure this for agents", "build me a workspace", "turn
this into an agent-runnable pipeline", "restructure this repo for
agents", "map this org as context", or an explicit `/icm-architect` — and
loads this skill automatically. No need to name the skill.

## Load
1. `../../icm/AGENTS.md`
2. `../../icm/instructions/ICM_ARCHITECT.md`

## Execute
Follow `ICM_ARCHITECT.md` exactly: pick Build or Restructure mode, run
its steps, and run the walk test before declaring anything done. In
Restructure mode, the migration step is a human gate — present the
target tree and migration map and get approval before moving files.

## Done
Report the walk-test result inline per bullet (pass/fail), not a general
assertion that the workspace works.
