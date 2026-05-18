# Merge Strategy (Snapshot: 2026-04-23)

## Repository State
- Current branch: `work`.
- Working tree: modified analysis docs only (this PR scope).
- Merge conflict markers: none detected.

## Upstream Status
- `git remote -v` returns **no remotes configured** in this environment.
- Because no upstream remote exists locally, divergence checks (`HEAD...upstream/main`) are currently blocked.

## Safe Upgrade Procedure Once Remote Access Is Available
1. Add canonical upstream remote.
2. Fetch upstream refs.
3. Compute divergence (`git rev-list --left-right --count HEAD...upstream/main`).
4. Create integration branch `upgrade/<date>-upstream-sync`.
5. Merge/rebase with conflict resolution policy:
   - preserve local operator/business logic,
   - adopt safe kernel/runtime/security improvements from upstream,
   - remove duplicate or regressed code.
6. Run full validation (`scripts/run_tests.sh` + frontend builds + container build).
7. Only then merge upgrade branch into feature branch.

## Conflict-Resolution Policy
- Prefer compatibility and explicit adapters over invasive edits to core Hermes loop.
- Record every non-trivial conflict decision in this file during execution.
