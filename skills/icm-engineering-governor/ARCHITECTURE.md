# ICM Engineering Governor Architecture

## Scaffold

```text
skills/
└── icm-engineering-governor/
    ├── SKILL.md
    ├── manifest.json
    ├── ATTRIBUTION.md
    ├── ARCHITECTURE.md
    ├── references/
    │   ├── engineering.md
    │   ├── productivity.md
    │   ├── misc.md
    │   └── beta.md
    └── workflows/
        └── code-review/
            └── PROCESS.md

cron/
└── icm-code-review.json

tests/
└── skills/
    ├── test_icm_engineering_governor.py
    └── test_icm_code_review_gate.py
```

## Runtime model

```mermaid
flowchart TD
    U[Owner / Bambu] --> H[Hermes / Cosmos]
    H --> I[ICM Engineering Governor]

    I -->|lazy route| E[Engineering references]
    I -->|lazy route| P[Productivity references]
    I -->|lazy route| M[Misc references]
    I -->|explicit/bounded| B[Beta references]

    E --> S[Spec / Tickets]
    S --> X[Implementation]
    X --> T[Tests / Proof]
    T --> C{Project completion boundary}

    C -->|mandatory wake-up| R[Code Review Process]
    R --> SR[Standards Reviewer subagent]
    R --> PR[Spec Reviewer subagent]
    SR --> F[Findings reconciliation]
    PR --> F
    F -->|material fixes| X2[Builder fixes]
    X2 --> RR[Fresh re-review]
    RR --> F
    F --> G[Target proof + rollback]
    G --> HR[Hermes asks owner to review code/diff]
    HR -->|Approved| DONE[DONE / SHIP allowed]
    HR -->|Changes requested| X
    HR -->|Declined review| DONE
    HR -->|Pending| HOLD[HOLD]

    CRON[Nightly cron review] --> R
    CRON -->|no merge/deploy authority| REPORT[Durable review receipt]
    R --> REPORT
```

## Invariants

- Capability details remain lazy-loaded except the completion trigger.
- Completion review always wakes when a project reaches its closing boundary.
- The builder cannot be the final approver.
- Cron review may inspect, test, and dispatch reviewer subagents, but cannot merge or deploy.
- A human code-review prompt must be surfaced before project closure.
- Pending human review keeps the project on HOLD.
- Declining to inspect the diff does not waive automated proof or independent review.
