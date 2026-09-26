# Stage 02: Process Metadata

## Input

Required:
- `runs/<run-id>/metadata.json` (raw output from stage 01)

## Process

1. **Normalize timestamps**
   - Parse "3 months ago" → ISO8601 date (estimated)
   - Parse "2 weeks ago" → ISO8601 date
   - Parse exact dates if present
   - Fallback: use today's date if unparseable

2. **Normalize view counts**
   - "12K views" → 12000
   - "1.5M views" → 1500000
   - "45 views" → 45
   - Extract numeric value, discard "views" suffix

3. **Deduplicate**
   - Group by `video_url`
   - Keep only latest entry if same video appears in multiple playlists
   - Log duplicates found

4. **Validate URLs**
   - Ensure all `video_url` start with `https://www.youtube.com/watch?v=`
   - Ensure `playlist_url` contains valid `list=` parameter
   - Flag invalid URLs (log but keep record)

5. **Add computed fields**
   ```json
   {
     "...": "...",
     "video_id": "extracted from URL",
     "playlist_slug": "extracted from URL",
     "upload_date_iso": "2026-05-06",
     "view_count_numeric": 12000,
     "scrape_source": "youtube-channel-scraper",
     "processed_at": "2026-08-06T23:45:12Z"
   }
   ```

## Output

File to write:
```
runs/<run-id>/processed.json
[
  {
    "playlist_name": "...",
    "playlist_url": "...",
    "playlist_slug": "PL_example1",
    "video_title": "...",
    "video_url": "...",
    "video_id": "dQw4w9WgXcQ",
    "upload_date": "3 months ago",
    "upload_date_iso": "2026-05-06",
    "view_count": "12K views",
    "view_count_numeric": 12000,
    "description": "",
    "scrape_source": "youtube-channel-scraper",
    "processed_at": "2026-08-06T23:45:12Z"
  }
]
```

And quality report:
```
runs/<run-id>/processing_report.json
{
  "total_records": 47,
  "deduplicated": 3,
  "final_count": 44,
  "timestamps_normalized": 44,
  "timestamps_unparseable": 0,
  "view_counts_normalized": 44,
  "urls_validated": 44,
  "urls_invalid": 0,
  "timestamp": "2026-08-06T23:45:12Z"
}
```

## Exit gates

| Gate | Result |
|------|--------|
| ✓ 100% processed | → Stage 03: Analyze patterns |
| ✓ 90%+ processed (minor failures OK) | → Stage 03 |
| ✗ <90% success rate | → Log warnings, ask user to review metadata quality |

## Next stage

→ `stages/03_analyze_patterns/CONTEXT.md`

---

**Note:** This stage is safe to re-run (idempotent). Useful if stage 03 patterns reveal data issues.
