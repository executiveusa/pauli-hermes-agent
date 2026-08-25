# Web Intelligence Skill

## Purpose

ICM governed web research and extraction layer for Hermes.

## Providers

- Agent Reach: discovery and source expansion.
- ScrapeGraphAI: structured AI extraction.
- Scrapling: browser and difficult-page extraction fallback.

## Workflow

1. Parse research objective.
2. Discover sources.
3. Extract information using the best provider.
4. Verify provenance.
5. Store evidence artifacts in ICM.
6. Return evidence-backed results.

## Rules

Never store scraped information directly into memory without verification.

Every artifact must contain:
- source URL
- timestamp
- provider
- extraction method
- confidence

Do not bypass authentication, paywalls, or private systems.
