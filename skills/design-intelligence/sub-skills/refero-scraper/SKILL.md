---
name: refero-scraper
description: >
  Nightly scraper for styles.refero.design. Fetches the top 5 design references
  for a given niche, extracts design token signals (colors, typography, spacing),
  and writes structured data into the _feeds/ layer alongside awwwards data.
  Triggers via the nightly cron job or on "scrape refero", "get refero data for [niche]".
version: 1.0.0
author: Kupuri Media™
---

# Refero Scraper

## Scrape Target

```
https://styles.refero.design/
```

Refero organizes designs by category. Use the niche-to-category mapping in
`_shared/niches.md` to find the relevant section.

## What to Extract

For each of the top 5 designs in the niche:
1. Site name and URL
2. Color palette (primary, secondary, accent hex values)
3. Typography signals (primary font family, weight range, scale ratio if detectable)
4. Spacing grid (base unit if detectable)
5. Notable design tokens visible in the design system panel

## Output Format

Append to the existing `_feeds/{niche}-latest.md` file under a new section:

```markdown
## Refero Design Token Signals

### Site: {name}
- Primary bg: {hex}
- Primary text: {hex}
- Accent: {hex}
- Font: {family} | Weights: {list}
- Base spacing unit: {px or rem if detectable}

[repeat for top 5]

## Cross-Reference Notes

Patterns appearing in 3+ of the top 5 sites are structural signals, not coincidence:
- {pattern}: found in {site1}, {site2}, {site3}
```

## Error Handling

Same as awwwards-scraper: back off, retry once, write error file, log to ops/reports.
Do NOT overwrite `{niche}-latest.md` on partial scrape failure.
