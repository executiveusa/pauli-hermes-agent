---
name: icm-engineering-governor
description: Lazy ICM engineering workflow router adapted from Matt Pocock's public engineering/productivity skills. Use when work needs grilling, domain modeling, research, specs, dependency-aware tickets, prototypes, TDD, diagnosis, implementation, review, merge-conflict resolution, handoff, teaching, questionnaires, agent-writing, git guardrails, pre-commit setup, or long-horizon planning. Load only the minimum referenced capability needed for the current phase.
version: 1.0.0
license: MIT
metadata:
  hermes:
    tags: [icm, engineering, progressive-disclosure, lazy-load, software-factory, matt-pocock]
    related_skills: [gauntlet-loop, hardened-longrun-subagent-harness]
---

# ICM Engineering Governor

A lightweight router for a dormant engineering capability library. The router stays small; detailed capability instructions live in `references/` and MUST be loaded only when their trigger is present.

## ICM contract

Before substantial work establish:

- MODE: greenfield or brownfield.
- OUTCOME: measurable result.
- TARGET: customer, user, or system.
- CONSTRAINTS: what must not change.
- PROOF: evidence required.
- COMMERCIAL VALUE: revenue, savings, retention, or validated learning.

For brownfield work: inspect before changing, record a baseline, identify blast radius and rollback, preserve existing conventions, and make the smallest isolated change that can be verified.

## Lazy-loading law

1. Do not preload the reference library.
2. Classify the current need from the routing table below.
3. Load exactly one reference file first with `skill_view`.
4. Within that file, use only the named capability needed for the current phase.
5. Load a second reference file only if the task crosses a real phase boundary.
6. When a capability is no longer needed, stop carrying its detailed procedure forward; retain only outcomes, evidence, and unresolved decisions.
7. Never invoke beta capabilities silently for production-critical work.

## Routing table

### Engineering — `references/engineering.md`
Use for: ask-matt/router, grill-with-docs, triage, improve-codebase-architecture, setup-matt-pocock-skills, to-spec, to-tickets, implement, wayfinder, prototype, diagnosing-bugs, research, tdd, domain-modeling, codebase-design, code-review, resolving-merge-conflicts, wizard.

### Productivity — `references/productivity.md`
Use for: grill-me, grilling, handoff, teach, to-questionnaire, wait-what, writing-for-agents.

### Misc — `references/misc.md`
Use for: git-guardrails-claude-code, migrate-to-shoehorn, scaffold-exercises, setup-pre-commit.

### Beta — `references/beta.md`
Use only when explicitly beneficial and bounded: loop-me, writing-beats, writing-fragments, writing-shape, claude-handoff, setup-ts-deep-modules.

## Default software-factory chain

Use the smallest applicable prefix/slice of this chain; do not run every stage mechanically:

`GRILL -> MODEL -> RESEARCH -> SPEC -> TICKETS -> PROTOTYPE (if uncertainty) -> IMPLEMENT -> TDD -> REVIEW -> GAUNTLET -> PROOF`

For huge work, insert `WAYFINDER` before tickets. For bugs, replace the build path with `DIAGNOSE -> REPRODUCE -> MINIMIZE -> HYPOTHESIZE -> INSTRUMENT -> FIX -> REGRESSION TEST -> REVIEW`.

## Separation of duties

The builder cannot approve itself. Implementation and review must be logically separate passes. For material releases, combine this skill with the existing Gauntlet or another independent reviewer before claiming completion.

## Completion receipt

For major work return:

- DECISION
- CHANGES
- PROOF
- STATUS
- COMMERCIAL IMPACT
- RISKS
- ROLLBACK
- NEXT
- HUMAN APPROVAL

## Provenance

This is an ICM-native adaptation of concepts from `mattpocock/skills` (MIT licensed). It intentionally does not mirror upstream verbatim; Hermes-specific governance, progressive disclosure, owner control, rollback, and evidence requirements are added here. See `ATTRIBUTION.md`.