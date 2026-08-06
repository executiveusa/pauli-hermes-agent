# Scraper Workflow Router

## User intent → Stage mapping

| User asks / Intent | Stage to open | Entry point |
|---|---|---|
| "Set up the scraper" / "Check dependencies" | `stages/00_scraper_init/CONTEXT.md` | Verify Scrapling + Playwright installed |
| "Scrape this channel/playlist" + URLs provided | `stages/01_scrape_playlist/CONTEXT.md` | Execute scrape_youtube.py |
| "Normalize/deduplicate the data" | `stages/02_process_metadata/CONTEXT.md` | Load JSON, clean timestamps, validate |
| "What patterns are in this data?" / "Analyze trends" | `stages/03_analyze_patterns/CONTEXT.md` | Time series, content themes, view distribution |
| "Turn this into a workflow" / "Create automation" | `stages/04_generate_workflow/CONTEXT.md` | Call agent-workflow-builder skill |

## Stable references (guardrails)

- `guardrails/scraping-safety.md` — Rate limits, consent, error handling
- `icm/methodology.md` — Folder-as-agent methodology
- `resources/scrapling-config.py` — Reusable Scrapling configuration

## Run convention

Every run gets an ID:
```
YYYYMMDD-HHMMSS-<channel-slug>
```

Artifacts go to:
```
runs/<run-id>/
├── metadata.json        # raw scraped data
├── processed.json       # normalized + deduplicated
├── analysis.md          # pattern findings
└── workflow_spec.json   # A2A protocol workflow (if stage 04 completed)
```

## Handoff expectations

Between stages, output always includes:
- What succeeded / what failed
- Next stage recommendation
- Required inputs for next stage
- Any blockers or manual decisions needed

---

**When in doubt:** Read the stage's CONTEXT.md. Each stage is self-contained.
