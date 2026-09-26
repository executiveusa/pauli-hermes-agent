# New World Kids fundraising — workflow contract

Outcome: a compliant, evidence-backed fundraising and grant engine that reduces repetitive work without bypassing HSI or platform rules.

| Stage | Job | Human gate |
|---|---|---|
| `01_fundrazr` | Build/maintain campaign | Owner approves launch/settings |
| `02_grant-research` | Qualify opportunities | Owner selects opportunities |
| `03_grant-draft` | Draft from evidence | Owner verifies facts/budget |
| `04_hsi-review` | Obtain sponsor review | HSI approval required |
| `05_distribution` | Drive traffic | Owner approves outbound sequence/spend |
| `06_reporting` | Reconcile and learn | Owner verifies totals |

Stable references live in `references/`. Run-specific work belongs in each stage `output/` folder. A stage is not complete until its output exists and its gate is satisfied.
