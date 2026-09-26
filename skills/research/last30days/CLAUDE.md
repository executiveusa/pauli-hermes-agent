# last30days — Real-Time Social & News Research

## Quick Start

```bash
/last30days <topic>
# Or: "what's trending in AI agents?"
# Or: "what are people saying about X on Reddit and X?"
```

Hermes auto-activates the workflow. No setup required beyond the first-run wizard (installs yt-dlp, Digg CLI, browser cookie extraction — runs once).

## What This Does

Pulls posts, comments, and engagement from Reddit, X/Twitter, YouTube, TikTok, Hacker News, Polymarket, GitHub, Bluesky, Truth Social, and the general web for any topic within the last 30 days, then synthesizes a citation-backed, community-voice-driven report under a strict output contract (see `SKILL.md` LAWS 1-11).

Two modes:
- **Topic mode** — research a named topic, person, product, or comparison
- **Discovery mode** — sweep for what's trending globally or in a named domain, judged by the hosting model (three-command protocol)

## Authority

- **Automatic:** Read-only research against public social/web data on explicit or keyword-triggered request
- **Gated:** Optional credentials (X/Twitter cookies, Bluesky app password, ScrapeCreators/OpenAI/xAI/Perplexity keys) — all opt-in via the first-run wizard, never required for baseline operation
- **Gated:** Publishing a saved research library externally (`ht-ml.app`) requires explicit user consent before `--publish` is used

## Output Location

- Reports print to stdout / the user-facing response
- Raw research artifacts save to `LAST30DAYS_MEMORY_DIR` (default `~/Documents/Last30Days`)
- A local library/feed (`index.html`, `feed.xml`) can be built from saved research for content-calendar reuse

## Triggers (Auto-activation)

Hermes detects and auto-activates on:
- "/last30days [topic]"
- "what's trending [in domain]?"
- "what are people saying about [topic]"
- "content angles / podcast angle / X article angle for [topic]"

See `HERMES_ACTIVATION.md` for the full trigger list and detection pattern.

## Related Skills

- `skills/research/` siblings (arxiv, polymarket, blogwatcher) — narrower single-source research
- `skills/social-media/` — posting/distribution once content angles are drafted here
- `skills/youtube-channel-scraper/`, `optional-skills/research/scrapling/` — targeted single-platform scraping when last30days' multi-source sweep is unnecessary
