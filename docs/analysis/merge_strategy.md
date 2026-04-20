# Merge Strategy

## Branch + remote snapshot
- Current branch: `work`
- Remotes: ['upstream\thttps://github.com/NousResearch/hermes-agent.git (fetch)', 'upstream\thttps://github.com/NousResearch/hermes-agent.git (push)']

## Upstream sync status
- Attempted: `git remote add upstream https://github.com/NousResearch/hermes-agent.git`
- Attempted: `git fetch upstream`
- Result: blocked by network policy (`CONNECT tunnel failed, response 403`).
- Divergence (`HEAD...upstream/main`): unavailable: Command '['git', 'rev-list', '--left-right', '--count', 'HEAD...upstream/main']' returned non-zero exit status 128.

## Strategy when access is restored
1. Fetch upstream and compute commit delta.
2. Create `upgrade/<date>-sync-upstream` integration branch.
3. Merge upstream `main` into upgrade branch, resolve conflicts with priority to local operator-specific logic.
4. Run full test matrix via `scripts/run_tests.sh` plus frontend checks.
5. Fast-forward/cherry-pick validated upgrade commits into feature branch.
