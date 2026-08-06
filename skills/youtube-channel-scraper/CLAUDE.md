# YouTube Channel Scraper + Workflow Generator

Start here when using Claude Code with the scraper.

## Quick Path

1. Read `AGENTS.md` — agent identity and role
2. Read `CONTEXT.md` — active stage router
3. Identify the stage or mission (see CONTEXT router table)
4. Read only that stage's `CONTEXT.md`
5. Execute stage script or reasoning
6. Write outputs to `runs/<run-id>/`

## Stages Overview

| Stage | Purpose |
|-------|---------|
| `00_scraper_init` | Validate environment, set up Scrapling |
| `01_scrape_playlist` | Execute playlist scrape, collect metadata |
| `02_process_metadata` | Normalize timestamps, deduplicate, validate URLs |
| `03_analyze_patterns` | Find themes, upload trends, content patterns |
| `04_generate_workflow` | Convert learnings into A2A workflow spec |

## Run ID Format

```
YYYYMMDD-HHMMSS-<channel-slug>
```

All outputs go to: `runs/<run-id>/`

## Key Files

- `icm/methodology.md` — Interpretable Context Methodology
- `guardrails/scraping-safety.md` — Rate limits, consent, compliance
- `stages/*/CONTEXT.md` — Stage-specific contracts
