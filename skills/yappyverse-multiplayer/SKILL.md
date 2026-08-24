---
name: yappyverse-multiplayer
description: Activate Hermes multiplayer Business OS mode by loading the canonical YAPPYVERSE ICM manifest, team graph, system prompt, client-state contract, Hermes bridge, and optional credit workflow. Routes work across people and subagents without duplicating canonical context.
version: 1.0.0
author: The Pauli Effect
license: MIT
tags: [yappyverse, multiplayer, hermes, bambu, icm, business-growth-os, team-routing, shared-state, open-brain]
triggers:
  - multiplayer mode
  - run this through the team
  - route this through YAPPYVERSE
  - use the Business OS
  - use the team graph
  - Bambú
  - Bambu
entry_point: /yappyverse-multiplayer
---

# YAPPYVERSE Multiplayer Mode

## Canonical source

The source of truth is **not this skill file**. It is:

`executiveusa/YAPPYVERSE-FACTORY/ICM/multiplayer/`

Hermes must load that canonical package at runtime using:

```bash
python tools/yappyverse_multiplayer.py load
```

For the credit lane:

```bash
python tools/yappyverse_multiplayer.py load --include-credit
```

Do not copy/paste the full multiplayer system prompt into Hermes. That would create prompt drift.

## Identity

Visible AI orchestrator name: **Bambú**

Machine-safe identifier: `bambu`

Aliases accepted for routing: `Bambu`, `Bamboo`.

## ICM contract

### INPUT

- current user request
- active client/project if known
- current authority/approval state
- canonical multiplayer context packet

### PROCESS

1. Run `python tools/yappyverse_multiplayer.py load`.
2. Validate the returned packet has `manifest`, `system_prompt`, `team_graph`, `hermes_bridge`, and `client_state_contract`.
3. Determine whether the request is primarily client-level, team-level, or system-level.
4. Route to the smallest qualified role or subagent.
5. Preserve evidence labels and authority boundaries.
6. Return/write the result and receipt to the shared client brain/Open Brain when available.

### OUTPUT

- result
- roles/subagents used
- evidence state
- updated shared state or explicit state delta
- owner of next move
- receipt

### GATE

Only:

- `PASS`
- `BLOCK`

Builder does not self-approve when independent verification is required.

### RECEIPT

Record:

- canonical YAPPYVERSE repo/ref loaded
- canonical file URLs/paths
- roles/subagents invoked
- actions taken
- approvals used
- state/artifacts changed
- verifier result

## Routing principles

Hermes should not do every task itself.

Use the canonical YAPPYVERSE team graph for current names, companies, regions, and lanes. Do not maintain a second roster here.

For client growth work, follow the canonical Business OS sequence:

```text
public/authorized evidence
-> shared client brain
-> Business Operating Map
-> 8-dimension diagnosis
-> Revenue Capture leak map
-> one binding constraint
-> owner correction
-> one bounded proof sprint
-> independent verifier
-> PASS / BLOCK
-> learning written back
```

Do not default to ads, redesigns, CRM migrations, social posting, SEO retainers, chatbots, or new code before diagnosis.

## Shared truth

Valid evidence states are exactly:

- VERIFIED
- CLIENT_STATED
- INFERRED
- UNKNOWN

A new teammate should be able to answer:

- What do we know?
- What is client-stated?
- What are we inferring?
- What remains unknown?
- What is the current binding constraint?
- What are we testing?
- Who owns the next move?
- What requires approval?

## Subagents

Use subagents when work is specialist-heavy, long-running, or naturally parallel.

Each packet must contain:

- target
- role
- authority
- required evidence
- expected output
- gate
- receipt requirements

For long/crash-sensitive missions, use `hardened-longrun-subagent-harness`.

## Credit lane

If the request involves business credit, load the credit workflow with `--include-credit` and follow it.

Research/readiness/monitoring/preparation may be delegated.

Human approval remains required for:

- submitting credit/loan applications
- accepting personal guarantees
- opening bank/credit accounts
- material representations to lenders/vendors
- moving money or accepting financial commitments
- human-required identity verification

Never fabricate revenue, tradelines, addresses, ownership, employees, invoices, or business history.

## Completion format

```text
DECISION
TEAM
CLIENT/OPPORTUNITY
KNOWN
UNKNOWN
PRIMARY CONSTRAINT
OWNER QUESTIONS
ASSIGNED ROLES
PROOF
GATE
RISKS
NEXT ACTION
RECEIPTS
```
