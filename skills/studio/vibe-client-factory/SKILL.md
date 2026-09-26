---
name: vibe-client-factory
description: Run the complete Vibe Engineering client workflow from outcome framing through low-friction A/B decisions, bounded implementation, independent review, SHIP/HOLD, client proof, and safe improvement. Use for client projects, brownfield rescues, software-factory delivery, and client-owned phone apps.
version: 0.1.0
author: Bambú / Pauli Effect
license: MIT
tags: [vibe-engineering, icm, client-delivery, software-factory, decision-cards, governance, review]
platforms: [linux, macos, windows]
triggers:
  - start a client project
  - run vibe engineering
  - use the software factory
  - create a client decision app
  - prepare a controlled pilot
  - rescue this project
  - make this an A and B decision
  - finish and verify this project
  - vibe client factory
---

# Vibe Client Factory

## Purpose

Turn a client outcome into verified, client-owned work while hiding unnecessary technical complexity and preserving visible accountability.

The client should experience a simple phone-based decision surface. Hermes should operate the governed factory underneath it.

This is one workflow skill, not another orchestration platform.

## Native Hermes entry point

Invoke explicitly with:

```text
/vibe-client-factory <project request>
```

Hermes may also offer this skill when the request matches the frontmatter description or triggers. The full workflow is loaded only when invoked.

## Final outcome

A successful project ends with a client who can:

1. open a private branded app or secure project link on a phone;
2. make a small number of clear decisions without learning technical systems;
3. authorize only bounded work;
4. receive plain-language proof of what was completed;
5. own the repository, data, domain, assets, documentation, and credentials;
6. reverse a release when necessary;
7. continue without dependence on one model, agent, or developer.

## Core law

> Hide the machinery. Never hide the accountability.

## ICM operating model

Use the Interpretable Context Methodology:

- **Interpreter:** determine the real outcome, mode, audience, constraints, authority, proof, commercial value, and rollback.
- **Context:** load only the current stage, approved source material, explicit dependencies, and named prior-stage outputs.
- **Method:** execute one bounded stage, write durable artifacts, stop at human gates, and pass evidence to the next station.

The filesystem and structured project state are the source of truth. Chat history is supporting context, not the control plane.

## One production line

```text
OUTCOME
  -> INVESTIGATE
  -> DECIDE
  -> ARCHITECT
  -> BUILD
  -> VERIFY
  -> JUDGE
  -> RELEASE
  -> PROVE
  -> IMPROVE
```

Do not skip stations by calling a draft, CI run, merge request, or deployment request "done."

## Stage 0 — Outcome contract

Before changing code or client-facing material, record:

- MODE: greenfield or brownfield;
- OUTCOME: the smallest valuable human result;
- TARGET: exact repository, branch, deployment, or artifact;
- AUDIENCE: who must understand and use it;
- CONSTRAINTS: forbidden changes, legal/privacy limits, deadlines, budget, and ownership;
- PROOF: tests, review, visual evidence, live checks, and client comprehension evidence;
- COMMERCIAL VALUE: revenue created, cost reduced, risk avoided, adoption improved, or mission outcome supported;
- ROLLBACK: exact recovery path;
- HUMAN APPROVER: who may authorize consequential release.

If these are missing, remain in Intake. Do not infer consequential authority.

## Stage 1 — Investigate first, ask last

Inspect available evidence before questioning the client:

- current website and application;
- repositories, branches, issues, and pull requests;
- approved brand assets and copy;
- documents, notes, voice recordings, and prior decisions;
- hosting, database, analytics, domain, and ownership records;
- existing tests, deployment evidence, and known failures.

Classify findings into:

```text
CONFIRMED FACTS
REASONABLE INFERENCES
UNRESOLVED DECISIONS
MISSING CRITICAL INFORMATION
```

Ask the client only about unresolved decisions or missing critical information that blocks the next stage.

## Stage 2 — Client decision cards

Prefer one decision at a time. Never present more than three blocking decisions in one round.

Default to two real choices:

```text
DECISION
WHY IT MATTERS
OPTION A
OPTION B
RECOMMENDATION
CONSEQUENCE OF A
CONSEQUENCE OF B
SUPPORTING EVIDENCE
AUTHORITY REQUIRED
```

Use exactly two options unless an A/B framing would hide a genuine third path. Never manufacture false choices.

A decision card is not permission to deploy. It authorizes only the next bounded factory action recorded in the card.

Use `templates/decision-card.schema.json` for machine-readable records.

## Stage 3 — Architect directive

Convert the approved decision into a bounded build packet:

- exact scope;
- files or systems allowed to change;
- prohibited changes;
- acceptance criteria;
- native checks to run;
- review method;
- expected evidence;
- rollback;
- required approvals.

The Architect does not implement the work it specifies.

## Stage 4 — Builder execution

The Builder may:

- inspect approved context;
- create an isolated branch or worktree;
- draft and test reversible internal changes;
- write receipts and evidence.

The Builder may not:

- approve its own work;
- merge or deploy without the required gate;
- change billing, credentials, ownership, or production data without explicit authorization;
- expand scope because an adjacent improvement looks useful;
- conceal failed checks or missing evidence.

Reuse specialist skills rather than duplicating them.

## Stage 5 — Verification and Council

Run the target repository's native checks first. Then perform independent review.

Minimum review surfaces where relevant:

- correctness and acceptance criteria;
- security and privacy;
- accessibility and client comprehension;
- architecture and maintainability;
- deployment and rollback safety;
- commercial or mission consequence.

Use OpenCodeReview when installed. Code review does not replace deployment verification or client usability evidence.

Every finding must receive one disposition:

```text
FIXED
ACCEPTED_RISK
FALSE_POSITIVE_WITH_REASON
DEFERRED_WITH_OWNER_AND_DATE
BLOCKING
```

Critical or high findings cannot be silently deferred.

## Stage 6 — Judge

The Judge receives the directive, implementation evidence, test results, review findings, dispositions, rollback, and approval state.

The Judge returns only:

```text
SHIP
HOLD
```

Missing proof, unresolved blockers, exceeded authority, or absent approval produce `HOLD`.

## Stage 7 — Release

Release is a separate controlled action.

A client A/B decision must never directly trigger:

- merge to the production branch;
- production deployment;
- database migration;
- credential change;
- billing or purchase;
- destructive deletion;
- public publishing.

Those actions require the authority defined in `templates/authority-policy.json` and the exact project contract.

## Stage 8 — Proof returned to client

Return only useful client-facing information:

- what was decided;
- what changed;
- what was verified;
- what remains unverified;
- what the client owns;
- what action, if any, is required next.

Do not expose internal chain-of-thought, agent debates, raw infrastructure logs, or technical noise.

A client-facing completion claim requires plain-language proof and the professional technical term.

## Stage 9 — Improve safely

After the run, analyze:

- repeated client confusion;
- unnecessary questions;
- delayed decisions;
- review defects;
- failed releases;
- agent handoff failures;
- estimated versus actual effort;
- commercial or mission impact.

The improvement agent may propose workflow changes. It may not rewrite its own authority policy, bypass review, or self-approve the proposal.

System changes travel through the same factory.

## Client experience laws

1. Do not begin with a long intake form.
2. Investigate existing sources before asking questions.
3. Prefer one decision card at a time.
4. Use plain language first and the professional term second.
5. Show a recommendation, not a neutral wall of options.
6. Save progress automatically.
7. Default to a secure no-password experience: invitation link, device enrollment, then passkey or biometric unlock.
8. Make access revocable and require stronger confirmation for sensitive actions.
9. Adapt an existing client surface before creating another disconnected app.
10. The client must be able to export and receive their records.

## Authority classes

Use the authority policy template. Summary:

| Class | Typical action | Default |
|---|---|---|
| READ | inspect files, sites, docs | automatic |
| ANALYZE | compare, classify, recommend | automatic |
| DRAFT | prepare copy, plans, designs | automatic and labeled draft |
| REVERSIBLE_INTERNAL | branch changes, tests, local artifacts | allowed and logged |
| CLIENT_FACING | public copy or approved design changes | recorded client decision |
| RELEASE | merge, deploy, migration | review plus human approval |
| SENSITIVE | billing, credentials, ownership, destructive change | dual confirmation |
| PROHIBITED | hide evidence, bypass gates, self-approve | never |

The stricter project rule always wins.

## Required durable artifacts

Use the existing Vibe Engineering ICM workspace when available. Do not create a competing project-control hierarchy.

At minimum preserve:

- project contract;
- source audit;
- decision cards and receipts;
- architect directive;
- implementation record;
- native test evidence;
- independent review record;
- Judge verdict;
- release approval;
- live verification evidence;
- ownership and handoff record;
- rollback instructions;
- improvement proposal.

## Client app default

When a client-facing control surface is in scope, default to a branded mobile-first PWA with:

```text
TODAY
PROGRESS
PROOF
OWNERSHIP
HELP
```

The primary screen should show the next decision, recommendation, and two action buttons. Avoid a conventional project-management dashboard unless research proves it is necessary.

## Commercial classification

Classify the engagement as one of:

- VIBE AUDIT;
- VIBE RESCUE SPRINT;
- SOVEREIGN LAUNCH;
- MAXX OPERATIONS;
- INTERNAL FACTORY IMPROVEMENT.

Do not claim revenue, savings, adoption, or mission impact without evidence.

## Related skills

Load only when the current stage requires them:

- `campaign-factory` for campaign and QR delivery;
- `website-design` for strategy, wireframes, art direction, usability, and production design;
- `hardened-longrun-subagent-harness` for durable long-running execution;
- `local-footage-studio` for private footage understanding and edit planning;
- deployment skills for the approved host;
- OpenCodeReview or the configured review skill for independent code review.

## Supporting files

- `references/WORKFLOW.md` — full execution and client-app specification;
- `templates/decision-card.schema.json` — A/B decision record;
- `templates/authority-policy.json` — machine-readable authority classes;
- `templates/run-receipt.schema.json` — final evidence receipt;
- `scripts/doctor.py` — read-only installation and contract validation.

## Completion record

End every substantial run with:

```text
DECISION
CHANGES
PROOF
STATUS
COMMERCIAL IMPACT
RISKS
ROLLBACK
NEXT
HUMAN APPROVAL
```

Allowed overall states:

```text
PASS
PASS_WITH_DISPOSITIONS
HOLD
BLOCKED
NOT_RUN
```

`PASS` never means live production is healthy unless live target-environment evidence is included.
