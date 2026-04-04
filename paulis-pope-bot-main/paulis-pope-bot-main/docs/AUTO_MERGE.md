# Auto-Merge Controls

By default, job PRs that only modify files under `logs/` are automatically squash-merged. You can control this behavior with two **GitHub repository variables** (Settings → Secrets and variables → Actions → Variables tab).

---

## `AUTO_MERGE`

Kill switch for all auto-merging.

| Value | Behavior |
|-------|----------|
| *(unset or any value)* | Auto-merge enabled |
| `false` | Auto-merge disabled — all job PRs stay open for manual review |

---

## `ALLOWED_PATHS`

Comma-separated path prefixes that the agent is allowed to modify and still get auto-merged. If any changed file falls outside these prefixes, the PR stays open.

| Value | Behavior |
|-------|----------|
| *(unset)* | Defaults to `/logs` — only log files auto-merge |
| `/` | Everything allowed — all job PRs auto-merge |
| `/logs` | Only log changes auto-merge |

Path prefixes are matched from the repo root. A leading `/` is optional (`logs` and `/logs` are equivalent).

---

## Examples

Allow all agent changes to auto-merge (original behavior):
```
AUTO_MERGE = (unset)
ALLOWED_PATHS = /
```

Require manual review for everything:
```
AUTO_MERGE = false
```

Only auto-merge log changes:
```
ALLOWED_PATHS = /logs
```

If a PR is blocked, the workflow logs which files were outside the allowed paths so you can see exactly why.
