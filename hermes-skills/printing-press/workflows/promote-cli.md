# Workflow: Promote CLI lifecycle state

## Goal
Move a CLI from candidate to production through evidence-based gates.

## State model
`candidate -> installed -> dogfooded -> production -> deprecated`

## Promotion criteria
- `candidate -> installed`: package installs and basic read-only commands work.
- `installed -> dogfooded`: three real tasks completed successfully.
- `dogfooded -> production`: audit passed, approval gates configured, regression checks stable.
- `production -> deprecated`: replaced, broken, or no longer compliant.

## Required artifacts
- Workflow card (`templates/workflow-card.md`).
- Registry entry update.
- Audit report.

## Exit criteria
- Registry `status` reflects current lifecycle stage.
- Promotion rationale and validation evidence captured.
