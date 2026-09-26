# Stage 02_repo_inventory: Repo Inventory


## Inputs

- Normalized event or inventory mission request
- GitHub token availability
- `skills/repo-inventory/SKILL.md`

## Process

1. List target repo or target owner repos.
2. Capture default branch, clone URL, pushed timestamp, archived/fork/private flags.
3. Detect likely app framework from package files after clone only when needed.
4. Emit JSON.

## Outputs

- `runs/<run-id>/github-repos.json`
- `stages/02_repo_inventory/output/repos.json`

## Gate

Proceed when at least one repo is eligible and not archived.

