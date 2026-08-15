# Hermes Phase 01 — Upstream Truth and Baseline

## Objective

Establish a verified brownfield baseline before changing Hermes runtime behavior. Preserve Pauli-specific orchestration while identifying the exact upstream delta from `NousResearch/hermes-agent`.

## Baseline

- Fork: `executiveusa/pauli-hermes-agent`
- Upstream: `NousResearch/hermes-agent`
- Baseline branch: `main`
- Baseline SHA: `950aabef00059cbbf8a8a735da0bc215acd2d483`
- Upstream head observed during audit: `4aa9f738cedbe8a69fbd08595d0fb67f812ce2d3`
- Merge base observed during audit: `c79e3fd0baf41c0adda616b73153eeaa8a4b8231`
- GitHub cross-fork comparison: diverged; Pauli fork 163 commits ahead and 11,864 commits behind upstream at audit time.

## Existing capability already present in Pauli Hermes

The current `toolsets.py` already registers core capabilities that earlier planning treated as missing:

- web search/extract
- terminal + process management
- files
- vision + image generation
- skills list/view/manage
- browser automation including CDP, console, vision, dialogs
- text-to-speech
- todo + persistent memory
- session search
- code execution
- subagent delegation
- cron
- messaging
- Home Assistant
- Kanban worker coordination
- computer use (macOS via `cua-driver`)
- VPS SSH + Ralphy controls

This means the sprint must reconcile and harden existing substrate rather than duplicate it.

## Pauli-specific authority that must survive upstream reconciliation

- Hermes = business/cross-portfolio orchestrator
- Pi = personal/Human OS lane
- BARS = computer/media/operator worker
- Jarvis = presence/voice/communications
- Lightning = evaluator/watchdog/memory curator
- Pauli's Place = canonical mission/evidence state where applicable
- ICM routing and existing Pauliverse orchestration contracts
- Gauntlet/independent-review behavior
- minimum-necessary cross-lane context
- human approval for consequential actions

## Acceptance criteria

1. Current main and upstream relationship are recorded with exact SHAs.
2. Existing Hermes capabilities are inventoried from source, not assumptions.
3. Upstream reconciliation uses `PORT | ADAPT | KEEP | PARK` instead of wholesale merge.
4. No runtime behavior, production settings, secrets, domains, databases, or deployment targets change in this phase.
5. Rollback is deletion/revert of documentation-only phase files.
6. Phase 02 may begin only from this baseline and must compare actual capability implementations, not marketing/docs claims.

## Verification

- GitHub repository metadata confirms this repository is a fork of `NousResearch/hermes-agent`.
- Cross-fork compare API establishes exact merge base and ahead/behind counts.
- `toolsets.py` establishes currently registered tool capability.
- `AGENTS.md` establishes current code architecture and test/runtime entry points.

## Risk

LOW — documentation/audit only.
