# Stage 03: Deliver Results

**Input:** Structured data (JSON, CSV, etc from Stage 02)
**Output:** User report + downloadable files
**Duration:** <5 seconds

## Purpose

Summarize workflow results, report to user, provide download links and next-step suggestions.

## Process

1. **Verify outputs** — Check all Stage 02 files exist
2. **Generate report** — Summary with counts, stats, performance
3. **Create manifest** — List of all output files
4. **Suggest next steps** — Data loading, analysis, sharing options
5. **Report to user** — Delivery confirmation

## Verification Gates

| File | Must Exist | Check |
|------|------------|-------|
| videos.json | YES | Valid JSON, ≥1 video |
| by_playlist.json | YES | All playlists present |
| summary_stats.json | YES | Totals match video count |
| scrape_report.json | YES | Stage 01 metadata |

## Output Report Format

Print to user + save to `stages/03_deliver_results/output/REPORT.md`:

```markdown
# YouTube Scraper — Workflow Complete ✓

## Summary
- **Videos extracted:** 127
- **Playlists scraped:** 3
- **Channels:** 1 (@example)
- **Duplicates removed:** 2
- **Total runtime:** 5m 23s

## Output Files

| File | Type | Count | Size |
|------|------|-------|------|
| videos.json | Master dataset | 125 | 340 KB |
| by_playlist.json | Organized | 3 | 280 KB |
| summary_stats.json | Metadata | 1 | 2 KB |

Location: `youtube_scrapes/`

## Stats

- **Avg views per video:** 9,843
- **Total views:** 1,250,000
- **Newest video:** 2026-01-10
- **Oldest video:** 2020-01-15
- **Date range:** 6 years

## What's Next?

### Option 1: Load in Python
\`\`\`python
import json
with open('youtube_scrapes/videos.json') as f:
    videos = json.load(f)
print(f"{len(videos)} videos loaded")
\`\`\`

### Option 2: Load in Pandas
\`\`\`python
import pandas as pd
df = pd.read_json('youtube_scrapes/videos.json')
# Filter, aggregate, analyze
\`\`\`

### Option 3: Share
- Zip entire `youtube_scrapes/` directory
- Share via email or upload to cloud storage
- Import to Obsidian, Notion, or spreadsheet

## Need More?

- **Transcripts:** Use YouTube API or `yt-dlp --write-auto-subs`
- **Audio:** Extract with `yt-dlp -f bestaudio`
- **Sentiment analysis:** Process descriptions with Claude
- **Trending:** Sort by view_count, compare dates
```

## Files Generated

**In `youtube_scrapes/` directory:**
```
youtube_scrapes/
├── videos.json                    # Master dataset
├── by_playlist.json               # Organized by source
├── summary_stats.json             # Aggregate stats
├── playlist_*.json                # Individual playlist exports (if >10 playlists)
├── scrape_report.json             # Stage 01 report
└── MANIFEST.json                  # File inventory + checksums
```

## Manifest Format

`stages/03_deliver_results/output/MANIFEST.json`:

```json
{
  "workflow_id": "req_2026_001",
  "completed_at": "2026-01-15T10:35:50Z",
  "files": [
    {
      "path": "youtube_scrapes/videos.json",
      "type": "json",
      "size_bytes": 340000,
      "row_count": 125,
      "checksum_md5": "abc123..."
    },
    {
      "path": "youtube_scrapes/summary_stats.json",
      "type": "json",
      "size_bytes": 2000,
      "checksum_md5": "def456..."
    }
  ],
  "total_files": 3,
  "total_size_bytes": 342000,
  "next_steps": ["load_in_python", "share_via_zip", "import_to_notion"]
}
```

## On Gate Failure

If output validation fails:
- Error: "[File] missing or invalid"
- Gate: HOLD delivery, alert user
- Action: Re-run Stage 02, then Stage 03

Example:
```
ERROR: Stage 03 — Output validation failed

Missing file: youtube_scrapes/videos.json
Possible cause: Stage 02 crashed during JSON write
Action: Check Stage 02 logs, re-run: python stage_runner.py --stage 02
```

## Workflow Complete

All stages done. User has:
- ✅ Structured video dataset
- ✅ Playlist groupings
- ✅ Statistics and metadata
- ✅ Ready-to-use files (JSON, etc)
- ✅ Suggestions for next steps

No further stages.
