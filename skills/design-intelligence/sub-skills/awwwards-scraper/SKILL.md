---
name: awwwards-scraper
description: >
  Nightly scraper for Awwwards.com. Fetches the top 5 sites in a given niche
  category, extracts the tech stack, scrapes the Awwwards jury comments and blog
  posts, and writes a structured _feeds/{niche}-latest.md file. Runs on cron.
  Triggers on "scrape awwwards", "update the design feed", "get top sites for [niche]",
  or via the nightly cron job defined in cron/design-intelligence.json.
version: 1.0.0
author: Kupuri Media™
---

# Awwwards Scraper

## Scrape Targets

### Site of the Day / Week by Category

```
https://www.awwwards.com/websites/{category}/
```

Map niche to category using `_shared/niches.md`.

### Jury Comments
Each Awwwards listing page includes jury scores and comments.
Scrape these per site. Do NOT reproduce verbatim — paraphrase into mechanisms.

### Blog Posts

```
https://www.awwwards.com/blog/
```

Filter to posts from the last 30 days. Match to the current niche keyword.
Store the URL and a one-line summary. Do not reproduce article content.

## Output Format

Write to `skills/design-intelligence/_feeds/{niche}-latest.md`:

```markdown
# {Niche} Design Feed — {YYYY-MM-DD}

## Top 5 Sites

### 1. {Site Name}
- URL: {url}
- Stack: {detected framework, CMS, notable libraries}
- Awwwards Score: {score}/10 (if available)
- Jury Signals: {paraphrased, ≤2 sentences}

### 2–5: [same format]

## Visual Mechanisms Extracted

These are checkable by a critic looking at rendered output — not adjectives.

1. {mechanism}
2. {mechanism}
3. {mechanism}
4. {mechanism}
5. {mechanism}

## UDEC Axis Signals for This Niche

| Axis | Signal | Notes |
|------|--------|-------|
| TYP | {high/low/variable} | {one line} |
| MOT | {high/low/variable} | {one line} |
| IMG | {high/low/variable} | {one line} |

## Awwwards Blog Posts (Last 30 Days)

- [{title}]({url}) — {one-line summary}

## Scrape Metadata
- Scraped: {ISO timestamp}
- Niche: {niche}
- Awwwards Category: {category}
- Sites Found: {count}
```

## Error Handling

If Awwwards blocks or rate-limits:

1. Back off 60 seconds, retry once
2. On second failure, write a `_feeds/{niche}-error.md` with the HTTP status and timestamp
3. Do NOT write partial data to `{niche}-latest.md`
4. Log the failure to `ops/reports/design-intelligence-errors.json`

## Anti-Patterns

* Never store raw HTML in the feed files
* Never reproduce jury quotes verbatim (copyright — paraphrase always)
* Never load more than 3 feed files into context at once
* Never write directly to `popular-web-designs` templates (that's a separate, static skill)
