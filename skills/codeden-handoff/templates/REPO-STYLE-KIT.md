# One-time per-repository style-kit PR

Do not blindly overwrite project rules. Inspect the existing AGENTS.md/CLAUDE.md, repository policy and active PRs; choose the canonical existing file and propose only an additive reviewed change.

## AGENTS.md or CLAUDE.md (adapt to this repo)
Project name, owner and production boundaries:
Stack, architecture, existing patterns and design references:
Install/bootstrap commands and supported environments:
Build, unit, integration, lint, security and browser checks (exact commands):
Naming, formatting, API contracts, files/paths to preserve:
Approved skills (names, pinned source revisions, licenses, copied-in scoped text paths):
Definition of done: verified acceptance criteria, security, accessibility/mobile, screenshots for UI changes, independently checked PR on exact SHA, rollback:
No direct pushes to main, no secret leakage, no self-review, no automatic deployment or spend.

## .codeden/VISION.md
Product purpose, target customer and verified current baseline:
Commercial outcome and success measurement (hypotheses clearly marked):
One bounded implementation slice:
Non-goals and protected UX/brand/data/infrastructure:
Acceptance proof and source-of-truth docs:
Owner approvals for releases / external contacts / spend:
Rollback path:

## PR receipt
Base and head SHA, files changed, evidence of existing rules preserved, independent reviewer, test results, why this is a one-time kit rather than a duplicate:
