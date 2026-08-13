# _feeds/ — Live Design Intelligence Feed

This folder contains nightly-scraped design intelligence files.
Each file represents the top 5 sites for a niche as of the last scrape.

## File Format

- `{niche}-latest.md` — always the most recent scrape for that niche
- `{niche}-{YYYY-MM-DD}.md` — archived daily snapshot

## Contents per file

- Top 5 site URLs with screenshots (or screenshot paths)
- Stack detected (framework, CMS, notable libraries)
- 5–7 extracted visual mechanisms (checkable, not adjective-based)
- Judge commentary excerpts from Awwwards (paraphrased, not quoted verbatim)
- Blog post links from Awwwards blog covering this niche (last 30 days)
- UDEC axis signals (what axis this niche tends to score high/low on)

## Usage

Load only the feed file for the niche you're currently building for.
Do NOT load all feeds into context simultaneously.
