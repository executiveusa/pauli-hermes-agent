# Stage 01: Scrape Target

**Input:** Targets list (JSON from Stage 00)
**Output:** Raw video entries (JSONL)
**Duration:** 1-5 minutes (depends on playlist count/size)

## Purpose

Use Scrapling + Playwright to stealth-fetch video metadata from each target.

## Process

1. **Initialize fetcher** — Create PlayWrightFetcher with stealth=True
2. **For each target:**
   - If channel → auto-discover playlists
   - For each playlist → scrape_playlist()
   - Rate-limit between requests
3. **Extract video data** — Title, URL, view count, upload date
4. **Handle errors** — Log failures, continue to next playlist
5. **Output raw entries** — One JSON object per video

## Key Logic

From `skills/youtube-intelligence-pipeline/scrape.py`:

- `discover_playlists()` — Extract all /playlists page links
- `scrape_playlist()` — Extract ytd-playlist-video-renderer entries
- `fetch_description()` — Optional: Get full description from video page

## Input Validation

```
Stage 00 output present?
  YES → Parse targets.json
  NO  → ERROR: "Stage 00 not complete, run first"
```

## Rate Limiting

- 1 second sleep between playlist fetches
- YouTube soft limit: ~40 requests/min (Scrapling adapts)
- If 429 (Too Many Requests): wait 60s, retry

## Output Format

**File:** `stages/01_scrape_target/output/raw_videos.jsonl`

One JSON object per line (JSONL format):

```json
{"channel_name": "Example Channel", "channel_url": "https://www.youtube.com/@example", "playlist_name": "Uploads", "playlist_url": "https://www.youtube.com/playlist?list=PL123", "video_title": "How to Do X", "video_url": "https://www.youtube.com/watch?v=abc123", "upload_date": "3 months ago", "view_count": "12K views", "description": ""}
```

## Also Output

**File:** `stages/01_scrape_target/output/scrape_report.json`

```json
{
  "stage": "01_scrape_target",
  "started_at": "2026-01-15T10:30:00Z",
  "completed_at": "2026-01-15T10:35:42Z",
  "targets_processed": 2,
  "playlists_attempted": 3,
  "videos_extracted": 127,
  "videos_failed": 2,
  "rate_limit_hits": 0,
  "errors": []
}
```

## On Error

If scrape fails on a playlist:
- Log error with playlist URL
- Continue to next playlist (do NOT stop)
- Report summary with failed count

**Critical error:** Network unreachable
- Gate: BLOCK Stage 02
- Action: Check internet, retry Stage 01

**Recoverable error:** YouTube rate limit
- Gate: HOLD 60 seconds, retry automatically
- Log: "Rate limit hit on [playlist], retrying..."

## Next Stage

→ Stage 02: Structure Output (clean, dedupe, enrich)
