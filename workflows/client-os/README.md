# Client OS Workflow

Purpose: reusable operating system for embedded growth partnerships.

## Operating Architecture

Hermes is the client operating brain and durable intelligence layer.

Every client is represented inside Hermes using an ICM workspace. Do not create a standalone client repository merely to hold research, audits, decisions, reports, or memory.

Use a separate GitHub repository only when the client has deployable software, a website/app we own or maintain, custom integrations, or other code that requires independent version control and deployment.

## Required ICM Workspace

```text
clients/<client-slug>/
├── icm/
│   ├── context/
│   ├── intelligence/
│   ├── decisions/
│   ├── execution/
│   └── proof/
├── intake/
├── audits/
├── assets/
├── reports/
└── memory/
```

### ICM Context

`business.md`, `owner.md`, `customers.md`, `offers.md`, `brand.md`, `market.md`, `competitors.md`, `constraints.md`

### ICM Intelligence

`seo/`, `competitor-analysis/`, `customer-language/`, `market-research/`, `reviews/`, `social-analysis/`

### ICM Decisions

`hypotheses.md`, `priorities.md`, `tradeoffs.md`, `experiments.md`

### ICM Execution

`campaigns/`, `content/`, `automation/`, `sales/`, `operations/`

## ICM Meaning Contract

Do not store facts without meaning. Every material recommendation should follow:

```text
Observation
→ Context
→ Interpretation / Hypothesis
→ Decision
→ Expected outcome
→ Measurement
→ Result
```

Proof records should include:

```text
Claim:
Evidence:
Source:
Date:
Confidence:
Result:
```

## Flow

1. Intake
2. Populate ICM context
3. Collect evidence into ICM intelligence
4. Business diagnosis
5. Opportunity scoring
6. Record decision and hypothesis
7. Quick-win execution
8. Measurement and proof
9. Update memory
10. Monthly growth loop

## Principles

- Hermes is the operating layer.
- ICM folders are the canonical client memory structure.
- Evidence before recommendations.
- Revenue impact before activity metrics.
- Capture client knowledge as durable context.
- Separate every client workspace.
- Avoid generic AI output.
- Do not duplicate client intelligence across application repositories.
