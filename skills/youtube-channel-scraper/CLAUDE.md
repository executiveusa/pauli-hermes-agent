# YouTube Channel Scraper — ICM Workflow

## Quick Start

```bash
/youtube-channel-scraper
# Or: "scrape YouTube channel [URL]"
# Or: "extract videos from [playlist URL]"
```

Hermes auto-activates the workflow. No setup required.

## What This Does

1. **Parse** — Extract channel/playlist URLs from your request
2. **Scrape** — Stealth browser automation via Scrapling + Playwright
3. **Structure** — JSON output with video metadata
4. **Deliver** — Files ready to use + summary report

## Workflow Stages

| Stage | Input | Process | Output |
|-------|-------|---------|--------|
| 00 | User request | Parse URLs, validate | Structured target list |
| 01 | Target list | Run Scrapling fetcher | Raw video entries |
| 02 | Raw entries | Clean, dedupe, enrich | Structured JSON |
| 03 | JSON files | Report summary, next steps | Ready-to-use dataset |

## Authority

- **Automatic:** Scraping any publicly accessible YouTube content
- **Gated:** Only on explicit user request (not autonomous)

## Output Location

All results go to `youtube_scrapes/` directory with timestamp:
- `playlist_*.json` — Video metadata per playlist
- `summary_*.json` — Scrape run report

## Triggers (Auto-activation)

Hermes detects and auto-activates on:
- "scrape YouTube channel [URL]"
- "extract videos from [playlist]"
- "download YouTube data [request]"
- "/youtube-channel-scraper"

See `AGENTS.md` for full trigger list.
