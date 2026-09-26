---
name: hyperagent-browser-operator
description: ICM-governed browser execution layer for Hermes/Cosmos using HyperAgent as an external runtime. Use for unknown or multi-step browser work that benefits from natural-language execution, action caching, deterministic replay, structured extraction, or repair of a previously learned browser routine. Prefer existing deterministic browser tools when they are sufficient.
version: 0.1.0
author: Bambu / Pauli Effect
license: MIT
tags: [icm, browser, hyperagent, playwright, action-cache, routine-compiler, replay, subagent, automation]
triggers:
  - use HyperAgent
  - learn this browser workflow
  - turn this browser task into a routine
  - replay this browser routine
  - repair this browser automation
  - automate this website in the browser
entry_point: /hyperagent-browser-operator
---

# HyperAgent Browser Operator

## Purpose

Give Hermes/Cosmos a repeatable browser-execution worker without replacing Hermes' existing browser stack or decision authority.

HyperAgent is an external AGPL-3.0 dependency. Hermes owns the adapter, policy, receipts, routine registry, approvals, and proof. Do not copy HyperAgent source code into Hermes.

## Core routing rule

Use the cheapest reliable path:

1. Existing deterministic Hermes browser/CDP tool when the target and action are already known.
2. HyperAgent `perform` for one granular browser action.
3. HyperAgent `ai` for unknown or multi-step UI work.
4. HyperAgent `extract` for structured/semantic page reading.
5. HyperAgent `replay` when a verified routine already exists.

Do not invoke AI browser reasoning when a verified replay or deterministic Hermes primitive can do the same job.

## ICM contract

Before browser mutation record:

- OUTCOME: exact business or technical result.
- TARGET: exact site/account/page/environment.
- AUTHORITY: read-only, bounded write, or human-approval-required actions.
- CONSTRAINTS: what must not change.
- PROOF: what observable state proves completion.
- ROLLBACK: how to reverse material changes.
- ROUTINE POLICY: record new routine, replay existing routine, or one-off execution.

## Authority boundaries

### Automatic read/analyze

Hermes may:

- navigate public or already-authorized pages;
- inspect DOM/page state;
- extract non-secret information;
- compare current state to an approved target;
- test a previously approved routine in a non-destructive path.

### Bounded automatic actions

Only when already authorized by the active task and reversible, Hermes may:

- fill forms with non-secret approved values;
- click through a known workflow;
- create temporary/staging objects;
- save an action cache after successful proof;
- replay a routine whose target identity still matches.

### Human approval required

Unless the current user instruction already explicitly authorizes it:

- purchases, paid-plan changes, or financial commitments;
- production deletion or destructive restore;
- DNS/domain ownership/credential/security changes;
- publishing or deployment that materially changes a live customer system;
- sending messages, submitting applications, contracts, or legal/medical representations;
- creating or rotating credentials;
- any browser action whose target identity is ambiguous.

## Routine lifecycle

### Learn

Use HyperAgent `ai` only when the workflow is not already deterministic.

A routine may be saved only after:

1. target identity is proven;
2. the browser run succeeds;
3. the intended result is independently verified;
4. the cache contains no credentials, tokens, session cookies, or sensitive form values that should not persist;
5. the routine receives a stable descriptive name.

### Replay

Before replay:

1. prove the current target is the same service/account/workflow class;
2. check that the requested authority matches the routine's authority;
3. load only the named routine;
4. replay with bounded XPath retries;
5. inspect whether fallback AI was required;
6. verify the final target state.

If replay drifts or fails, stop escalating blindly. Move to repair mode.

### Repair

When a routine breaks:

1. capture the failing step and current page state;
2. run the smallest HyperAgent `ai` repair needed;
3. verify the repaired workflow;
4. save a new routine version or replace the old cache only after proof;
5. preserve the prior cache until the replacement passes.

## Runtime

Adapter directory:

`integrations/hyperagent/`

Install on first explicit use:

```bash
cd integrations/hyperagent
npm install --ignore-scripts --no-audit --no-fund
npm run self-test
```

The top-level package is exact-pinned to `@hyperbrowser/agent@1.1.2`. The upstream package itself uses its own dependency graph; do not pretend Hermes controls those transitives without a lockfile and supply-chain review.

Runtime environment variables:

```text
OPENAI_API_KEY / provider-specific key
HYPERAGENT_LLM_PROVIDER   # default: openai
HYPERAGENT_LLM_MODEL      # default: gpt-4o
HYPERAGENT_BROWSER_PROVIDER=Hyperbrowser  # optional cloud mode
HYPERBROWSER_API_KEY      # only for Hyperbrowser cloud mode
```

Never place secret values in task JSON, saved routines, receipts, Git commits, or skill files.

## Invocation examples

### One granular action

```bash
printf '%s' '{"action":"perform","url":"https://example.com","instruction":"click the Sign in button"}' \
  | node integrations/hyperagent/runner.cjs
```

### Learn and save a routine

```bash
printf '%s' '{"action":"ai","url":"https://example.com","instruction":"open settings and navigate to billing history without changing anything","routine_name":"example-billing-history-v1"}' \
  | node integrations/hyperagent/runner.cjs
```

### Replay

```bash
printf '%s' '{"action":"replay","routine_name":"example-billing-history-v1"}' \
  | node integrations/hyperagent/runner.cjs
```

## Completion receipt

Return:

```text
DECISION
TARGET
MODE: deterministic | perform | ai | extract | replay | repair
ROUTINE: none | learned | replayed | repaired
CHANGES
PROOF
FALLBACK_USED
RISKS
ROLLBACK
STATUS
HUMAN APPROVAL
```

Never call a browser task complete from `ok: true` alone. Prove the actual requested target state.

## Max boundary

This skill is currently a Bambu/Cosmos pilot. Do not copy, enable, or route it into Agent Max until the owner explicitly approves promotion after runtime proof.
