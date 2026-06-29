# Context Bloat Policy

## Rule

Installed does not mean loaded.

## Always-Load Policy Layer

- `zero-touch-engineer-prime-directive`
- `pauli-agent-operating-contract`
- `memory-routing-policy`
- `secret-safety-policy`
- `tool-use-budget-policy`

These are policy concepts, not blanket skill payloads.

## Lazy-Load Rules

- GitHub/repo tasks load GitHub skills only when the task text mentions repo, PR, CI, branch, commit, or issue work.
- Deploy tasks load deployment skills only when the task text points at deploy, rollback, Docker, Coolify, Hostinger, or VPS work.
- Design tasks load design skills only when the task actually needs frontend or UX work.
- Memory tasks use retrieval and search, never full-vault injection.
- Video tasks stay under a five-skill budget and do not default to paid generation.

## Budgets

- Default: max 3 skills.
- Complex repo task: max 6 skills and require `jcodemunch-summary` plus `codebase-inspection` before broad file ingestion.
- Video task: max 5 skills, paid generation off by default.
- Production deploy: explicit approval, secret scan, rollback plan.

## Browser Policy

- Browser tasks use `browser-harness` as the first browser-control surface.
- Critical UI actions should not be hidden behind unlabeled controls.
- Frontend verification should use browser automation or a documented blocker.
