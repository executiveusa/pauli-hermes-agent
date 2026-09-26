# Stage 01: Scrape Playlist

## Input

Required:
- Channel URL (for context/naming)
- Playlist URLs (1-5 URLs, space or newline separated)

Example:
```
channel: https://www.youtube.com/@mychannel
playlists:
  https://www.youtube.com/playlist?list=PL_example1
  https://www.youtube.com/playlist?list=PL_example2
```

## Process

1. **Validate URLs**
   - Must start with `https://www.youtube.com/playlist?list=`
   - Extract slug from `list=` parameter
   - Skip if invalid → log warning, continue

2. **Execute scrape_youtube.py**
   ```bash
   python scrape_youtube.py \
     "https://www.youtube.com/@channel" \
     "https://www.youtube.com/playlist?list=PL1" \
     "https://www.youtube.com/playlist?list=PL2"
   ```

3. **Collect outputs**
   - `youtube_scrapes/<slug>_YYYYMMDD_HHMMSS.json` (raw metadata per playlist)
   - `youtube_scrapes/summary_YYYYMMDD_HHMMSS.json` (succeeded/failed counts)

4. **Move to run directory**
   ```bash
   mkdir -p runs/<run-id>/raw_scrapes
   cp youtube_scrapes/* runs/<run-id>/raw_scrapes/
   ```

5. **Log execution**
   ```
   runs/<run-id>/scrape.log
   - Timestamp, playlist URL, video count, elapsed time
   - Failed videos (if any) with reason
   ```

## Output

File to write:
```
runs/<run-id>/metadata.json
[
  {
    "playlist_name": "...",
    "playlist_url": "...",
    "video_title": "...",
    "video_url": "...",
    "upload_date": "3 months ago",
    "view_count": "12K views",
    "description": ""
  }
]
```

And summary:
```
runs/<run-id>/scrape_summary.json
{
  "channel_url": "...",
  "playlists_attempted": 2,
  "playlists_succeeded": 2,
  "total_videos": 47,
  "timestamp": "2026-08-06T23:45:12Z"
}
```

## Exit gates

| Gate | Result |
|------|--------|
| ✓ All playlists scraped | → Stage 02: Process metadata |
| ✓ Partial success (1+ playlists) | → Stage 02 (continue with what we have) |
| ✗ All playlists failed | → Escalate to user (network issue?) |
| ✗ Rate limited | → Wait 30s, retry once |

## Next stage

→ `stages/02_process_metadata/CONTEXT.md`

---

**Note:** This stage respects `guardrails/scraping-safety.md`:
- 1s delay between playlists
- Stealth mode enabled
- Failures logged but don't block run
