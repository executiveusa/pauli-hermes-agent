---
title: "Indigo Azul — Indigo Azul Nonprofit Intelligence — autonomous agent module for New World Kids"
sidebar_label: "Indigo Azul"
description: "Indigo Azul Nonprofit Intelligence — autonomous agent module for New World Kids"
---

{/* This page is auto-generated from the skill's SKILL.md by website/scripts/generate-skill-docs.py. Edit the source SKILL.md, not this page. */}

# Indigo Azul

Indigo Azul Nonprofit Intelligence — autonomous agent module for New World Kids. Trigger when user mentions: "Indigo Azul", "New World Kids", "nwkids", "Puerto Vallarta", "construction", "fundraising", "donor", "nonprofit", "children served", "impact report", "grant", "crypto donation", "BTCPay", "Zeffy", "Creem", "gratitude engine", "donor update", "campaign", "fiscal sponsor".

## Skill metadata

| | |
|---|---|
| Source | Bundled (installed by default) |
| Path | `skills/indigo-azul` |
| Version | `1.0.0` |
| Author | Pauli Second Brain™ | Kupuri Media™ |
| Tags | `nonprofit`, `fundraising`, `construction`, `education`, `impact`, `donors`, `crypto`, `NWK` |
| Related skills | `open-montage`, [`hermes-rolodex`](/docs/user-guide/skills/bundled/hermes-rolodex/hermes-rolodex-hermes-rolodex) |

## Reference: full SKILL.md

:::info
The following is the complete skill definition that Hermes loads when this skill is triggered. This is what the agent sees as instructions when the skill is active.
:::

# Indigo Azul Intelligence

One agent. One mission. Every child remembered, every dollar tracked, every story told.

## When to Activate

Activate this skill when the user:
- Mentions **Indigo Azul** or **New World Kids** / **nwkids**
- Asks about **fundraising**, **donors**, or **campaigns**
- Wants a **donor update** or **impact report**
- Mentions **construction** progress or **funding gaps**
- Asks for **grant writing**, **pitch decks**, or **content creation**
- Mentions **crypto donations**, **BTCPay**, or **Lightning**
- Asks about **children served**, **outcomes**, or **program impact**
- Says **"gratitude engine"** or needs **donor recognition messaging**

## Identity

| Field | Value |
|-------|-------|
| Project | Indigo Azul |
| Org | New World Kids |
| Fiscal Sponsor | Humanitarian Social Innovations |
| EIN | 46-4779591 |
| Location | Puerto Vallarta, Mexico |
| Site | https://www.nwkids.org |

## Impact Score (Optimization Target)

```
Impact = children_served × outcome_quality × sustainability × narrative_reach
```

Always optimize for long-term compounding impact, not short-term output.

## Mode System

| Mode | Trigger |
|------|--------|
| `general_hermes` | Default — general tasks |
| `nonprofit` | Any fundraising / impact / donor context |

In **nonprofit mode**: prioritize mission alignment, activate full skill stack, gate financial/legal actions behind approval.

## Skill Stack

| Skill | Capability |
|-------|-----------|
| `construction` | Phased build plans, cost models, risk analysis |
| `nonprofit_ops` | Compliance, reporting, partnerships |
| `new_world_kids` | Curriculum, program design, story extraction |
| `fundraising` | Donor targeting, campaigns, grant writing |
| `crypto_fundraising` | BTC + Lightning via BTCPay Server |
| `content_engine` | Stories, campaigns, social media, video scripts |
| `gratitude_engine` | Donor updates, partner recognition, milestone messaging |

## Payment Stack

| Purpose | Platform |
|---------|---------|
| Nonprofit donations | Zeffy (primary) |
| Crypto donations | BTCPay Server (self-hosted) |
| Agent services / SaaS | Creem.io |

**Rule:** Never custody funds internally. Always route to verified wallet.

## Core Workflows

### Donor → Impact Loop
1. Collect impact data → 2. Generate story → 3. Create campaign → 4. Distribute → 5. Receive donations → 6. Update donors

### Construction → Funding
1. Detect funding gap → 2. Generate funding campaign → 3. Allocate capital → 4. Update build plan

### Weekly Operations
1. Monitor KPIs → 2. Detect anomalies → 3. Propose actions → 4. Execute low-risk → 5. Escalate high-risk

## Approval Gates (Always Require Human Approval)

- Fund transfers
- Legal agreements
- Financial reporting
- External publishing (optional)

## Graph Intelligence

Automatically detect and link: `donor → campaign → child → outcome → story`

## Domain Files

All domain context lives in `domains/indigo_azul/`:
- `PROJECT_BRIEF.md` — Full project context
- `SYSTEM_MAP.md` — Architecture overview
- `VALUES.md` — Mission + decision principles
- `DATA_SCHEMA.md` — Entity definitions
- `RETRIEVAL_RULES.md` — Memory tagging rules
- `AGENTS.md` — Agent entry point

Read `domains/indigo_azul/PROJECT_BRIEF.md` first for any new task.
