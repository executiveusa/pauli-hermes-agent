# Indigo Azul — Project Brief

## Organization
- **Nonprofit:** New World Kids
- **Website:** https://www.nwkids.org
- **Fiscal Sponsor:** Humanitarian Social Innovations
- **EIN:** 46-4779591
- **Location:** Puerto Vallarta, Mexico

## Project
- **Name:** Indigo Azul
- **Type:** Construction + Education + Community Infrastructure

## Mission
Transform the lives of children in Puerto Vallarta through education, community infrastructure, and sustained human connection. Every dollar raised builds something permanent. Every child served becomes a story that funds the next one.

## Impact Score Formula
```
Impact = children_served × outcome_quality × sustainability × narrative_reach
```

The agent optimizes for this score continuously — not for short-term output metrics.

## Current Phase
See `SYSTEM_MAP.md` for current construction and program phase.

## Funding Channels
- **Zeffy** — Primary nonprofit donation platform
- **BTCPay Server** — Self-hosted crypto (BTC + Lightning)
- **Creem.io** — Agent service payments / SaaS subscriptions
- **Grants** — Active grant writing via Fundraising Engine

## Key Relationships
- Children → Programs → Outcomes → Stories → Donors → Campaigns → Funding → Build
- Fiscal sponsor (HSI) holds legal/financial responsibility
- New World Kids executes programs on the ground

## Deployment
- **Runtime:** Hermes Core (pauli-hermes-agent)
- **Memory:** Supabase + pgvector
- **Control Plane:** Paperclip
- **VPS:** Hostinger Docker deployment
- **Domain Module:** `/domains/indigo_azul/`
