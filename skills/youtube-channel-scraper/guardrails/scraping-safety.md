# Scraping Safety Guardrails

All stages must respect these limits and rules.

## Rate Limits

- **40 requests/minute** (NVIDIA NIM free tier soft limit)
- **1 second minimum delay** between playlist fetches
- **Batch size:** Max 5 playlists per run
- If rate limited → exponential backoff (2s, 4s, 8s, 16s), then notify user

## YouTube Compliance

- ✓ **Allow:** Public playlists, official channel playlists
- ✗ **Skip:** Private, unlisted, or age-restricted playlists
- ✗ **Never:** Download actual video files or bypass sign-in walls
- ✓ **OK:** Extracting public metadata (titles, dates, view counts)

## Data Privacy

- No user comments, subscriber data, or private channel info
- Anonymize in workflow output (use channel slug, not channel name)
- No storage of personally identifiable viewer data
- All outputs to `runs/<run-id>/` (ephemeral, not backed up)

## Error Handling

- **Video-level failure:** Log and continue (don't break the run)
- **Playlist-level failure:** Mark as failed, record reason, continue to next
- **Scraper crash:** Catch exception, save partial results, exit gracefully
- **Network timeout:** Retry once with 2s delay; if fails again, mark playlist as failed

## Exit Conditions

- ✓ Normal exit: All playlists processed (success + failures logged)
- ✗ Abnormal exit: Uncaught exception or max retries exceeded
- Always write `summary_<timestamp>.json` before exit (even on failure)

## Logging

Every run logs to: `runs/<run-id>/scraper.log`
```
[TIMESTAMP] [STAGE] [LEVEL] message
[2026-08-06 23:45:12] [01_scrape] [INFO] Starting playlist: https://...
[2026-08-06 23:45:15] [01_scrape] [WARN] Video parse failed (index 3): selector mismatch
[2026-08-06 23:46:02] [01_scrape] [ERROR] Playlist timeout after 45s
```
