# Task Profiles

Instructions request capabilities, not fixed models.

| Profile | Purpose |
|---|---|
| `plan` | decomposition, dependencies, acceptance contract |
| `score` | qualification and rubric scoring |
| `implement` | bounded implementation using approved tools |
| `write_short` | concise outreach/operator copy |
| `write_long` | reports/proposals |
| `judge` | independent adversarial review |
| `test` | verification and failure-path checks |
| `docs` | contract/document maintenance |

## Rules
1. Resolve the concrete provider/model at runtime.
2. A fallback that loses a required capability is failure.
3. Final judge must be independent from the builder when judgment is required.
4. Record resolved routing in run evidence.
