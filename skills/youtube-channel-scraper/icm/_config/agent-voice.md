# Agent Voice — Scraper Workflow

Use direct, machine-readable operational language.

## Prefer

- **Tables** for structured data
- **JSON** for outputs (no markdown tables for data)
- **Explicit gates** (PASS/FAIL, not "should try")
- **Imperative instructions** (do X, then Y)
- **Error codes** with recovery steps

## Avoid

- Motivational language ("Let's try to scrape")
- Vague hedging ("might", "could", "should try")
- Implicit assumptions (state them)
- Large context dumps (reference files instead)

## Example: Good Stage Report

```
Stage 01: Scrape Target [PASS]

Scraped:
- Playlist 1: 42 videos
- Playlist 2: 18 videos
- Total: 60 videos

Next: Stage 02 (Structure Output)
```

## Example: Bad Stage Report

```
Successfully scraped some videos from the playlists. The results 
look good and we're ready to move on. Let's structure the output next!
```

## URLs in Output

Always include full URLs, one per line in lists:
```
https://www.youtube.com/watch?v=abc123
https://www.youtube.com/watch?v=def456
```

NOT: "123 videos extracted" (say WHICH videos).

## Errors

Always provide:
- What failed (exact URL or stage)
- Why (rate limit / DOM change / network)
- What to do (retry / check selector / wait N seconds)

Example:
```
ERROR: Stage 01 — Playlist fetch failed

Playlist: https://www.youtube.com/playlist?list=PL123
Reason: YouTube rate limit (429 Too Many Requests)
Action: Wait 60 seconds, then retry: python stage_runner.py --stage 01 --resume
```
