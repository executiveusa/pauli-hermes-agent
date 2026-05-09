# Workflow: Audit CLI for autonomous safety

## Goal
Assess whether a CLI can be safely used by autonomous agents.

## Checklist
1. Auth scopes are minimal for required tasks.
2. Mutating operations require explicit opt-in flags.
3. Financial/legal/destructive commands are flagged and approval-gated.
4. Read-only commands are safe for autonomous execution.
5. Output is deterministic, compact, and machine-parsable.

## Procedure
1. Review command surface and risk tags.
2. Classify each command: `read_only`, `write`, `financial`, `legal`.
3. Map risk class to approval policy using `templates/approval-gate.md`.
4. Verify `--json --compact` output on representative commands.
5. Save summary and update registry risk fields.

## Exit criteria
- Risk classification complete.
- Approval policy documented.
- Registry risk and status fields updated.
