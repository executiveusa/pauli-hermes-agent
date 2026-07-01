---
name: browser-smoke-check
description: Verify that a Vercel deployment URL displays a real page instead of a 404, auth wall, blank shell, or error page.
---

# Browser Smoke Check

## Use when

- A deployment URL has been created.
- A site shows 404 or blank page.
- The agent must prove the hero or first meaningful content is visible.

## Script

```bash
node scripts/verify-url.mjs <url>
```

## Output

```json
{
  "visible": true,
  "status": 200,
  "title": "...",
  "h1": "...",
  "signals": [],
  "blockers": []
}
```
