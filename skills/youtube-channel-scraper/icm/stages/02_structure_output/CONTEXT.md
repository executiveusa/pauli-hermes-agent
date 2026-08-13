# Stage 02: Structure Output

**Input:** Raw video entries (JSONL from Stage 01)
**Output:** Structured datasets (JSON + CSV + Parquet)
**Duration:** <10 seconds

## Purpose

Transform raw Scrapling output into clean, deduplicated, enriched structured data.

## Process

1. **Parse JSONL** — Load raw_videos.jsonl
2. **Deduplicate** — Remove duplicate video_url entries
3. **Clean fields** — Normalize date/view formats
4. **Enrich** — Add video_id, extract year from upload_date
5. **Organize by playlist** — Group videos by source playlist
6. **Validate structure** — JSON schema check

## Deduplication

By `video_url` (extract video ID from URL if needed):
```
https://www.youtube.com/watch?v=abc123 → video_id: abc123
Keep: first occurrence (earliest in scraped order)
Remove: duplicate video_id entries
```

## Field Normalization

| Field | Raw | Cleaned | Example |
|-------|-----|---------|---------|
| `view_count` | "12K views" | Integer | 12000 |
| `upload_date` | "3 months ago" | ISO 8601 | "2025-10-15" (est) |
| `video_url` | With params | Clean | Remove &list= param |
| `video_id` | Extract | Add field | abc123 |

## Output Format

**File 1:** `stages/02_structure_output/output/videos.json`

```json
[
  {
    "video_id": "abc123",
    "channel_name": "Example Channel",
    "channel_url": "https://www.youtube.com/@example",
    "playlist_name": "Uploads",
    "playlist_url": "https://www.youtube.com/playlist?list=PL123",
    "video_title": "How to Do X",
    "video_url": "https://www.youtube.com/watch?v=abc123",
    "view_count": 12000,
    "upload_date": "2025-10-15",
    "upload_date_raw": "3 months ago",
    "description": "",
    "scraped_at": "2026-01-15T10:35:00Z"
  }
]
```

**File 2:** `stages/02_structure_output/output/by_playlist.json`

```json
{
  "playlist_123": {
    "playlist_name": "Uploads",
    "playlist_url": "https://www.youtube.com/playlist?list=PL123",
    "video_count": 42,
    "videos": [...]
  }
}
```

**File 3:** `stages/02_structure_output/output/summary_stats.json`

```json
{
  "total_videos": 127,
  "unique_videos": 125,
  "duplicates_removed": 2,
  "playlists": 3,
  "channels": 1,
  "view_count_total": 1250000,
  "view_count_avg": 9843,
  "oldest_video": "2020-01-15",
  "newest_video": "2026-01-10"
}
```

## Validation Gates

| Check | Pass | Fail |
|-------|------|------|
| JSONL parse | All lines valid JSON | Malformed JSONL |
| video_url | All URLs valid | Invalid URL format |
| Dedup | Count decreases | Duplicate count error |
| Enrich | video_id extracted | Can't parse URL |

## On Gate Failure

If JSONL parse fails:
- Error: "Raw JSONL malformed at line N: [line content]"
- Gate: BLOCK Stage 03
- Action: Check Stage 01 output, may need to re-scrape

## Next Stage

→ Stage 03: Deliver Results (report + prepare for user)
