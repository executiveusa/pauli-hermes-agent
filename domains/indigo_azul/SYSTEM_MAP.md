# Indigo Azul — System Map

## Architecture Layers

```
L1 — MEMORY (Second Brain)
  Supabase + pgvector
  Semantic + relational memory
  Required tags: project, org, domain

L2 — HERMES CORE
  Reasoning → Planning → Execution → Self-improvement

L3 — INDIGO AZUL DOMAIN
  /domains/indigo_azul/
  Skills, workflows, data schema
```

## Memory Tags (Required on Every Entry)

```yaml
project: indigo_azul
org: new_world_kids
domain: construction | education | fundraising | ops
```

## Skill Stack

```
skills/
  construction/       — build plans, cost models, risk
  nonprofit_ops/      — compliance, reporting, partnerships
  fundraising/        — campaigns, grants, pitch decks
  crypto_fundraising/ — BTC, Lightning, BTCPay
  new_world_kids/     — curriculum, programs, story extraction
  content_engine/     — narrative, social, video scripts
  gratitude_engine/   — donor updates, partner recognition
```

## Workflows

```
workflows/
  donor_update.md          — monthly donor communication
  fundraising_campaign.md  — campaign creation pipeline
  weekly_impact_report.md  — KPI monitoring + reporting
  construction_review.md   — build progress + funding gap detection
```

## Data Flow

```
Impact Data
    ↓
Story Generation (content_engine)
    ↓
Campaign Creation (fundraising)
    ↓
Distribution (social + email)
    ↓
Donations (Zeffy / BTCPay)
    ↓
Donor Update (gratitude_engine)
    ↓
Build Allocation (construction)
    ↓
Impact Data (loop)
```

## External Integrations

| System | Purpose |
|--------|--------|
| Supabase | Memory + vector search |
| BTCPay Server | Crypto donation processing |
| Zeffy | USD donation processing |
| Creem.io | Agent service billing |
| Paperclip | Orchestration + execution visibility |
| Hostinger VPS | Docker deployment host |

## Deployment Stack

```
Docker
  └── FastAPI (agent API)
  └── Supabase client (memory)
  └── Redis (optional caching)
  └── BTCPay Server (crypto)
```

## Environment Variables

```
PROJECT=indigo_azul
ORG=new_world_kids
SUPABASE_URL=
SUPABASE_KEY=
BTC_WALLET=
BTPAY_URL=
CREEM_API_KEY=
PAPERCLIP_ENDPOINT=
```
