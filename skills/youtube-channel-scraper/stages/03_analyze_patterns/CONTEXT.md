# Stage 03: Analyze Patterns

## Input

Required:
- `runs/<run-id>/processed.json` (normalized metadata from stage 02)

## Process

1. **Upload frequency analysis**
   - Count videos per month (from `upload_date_iso`)
   - Find average days between uploads
   - Identify upload trends (accelerating? slowing down?)

2. **View distribution**
   - Min, max, median, mean view counts
   - Percentiles (p25, p50, p75, p90)
   - Videos underperforming (<50th percentile)
   - Videos overperforming (>90th percentile)

3. **Content themes (by title)**
   - Extract common keywords from video titles
   - Group by theme (e.g., "tutorial", "review", "live", etc.)
   - Count theme frequency

4. **Playlist patterns**
   - Videos per playlist (avg, min, max)
   - Playlist size distribution
   - Overlap between playlists (same video in multiple)

5. **Time trends**
   - Plot upload dates on timeline
   - Identify content gaps or bursts
   - Seasonal patterns (if data spans 6+ months)

## Output

File to write:
```
runs/<run-id>/analysis.md
# YouTube Channel Analysis

## Upload Frequency
- **Average gap:** 14 days between uploads
- **Trend:** Accelerating (last 3 months: 10-day average)
- **Last upload:** 3 days ago
- **Upload consistency:** High (std dev: 2.1 days)

## View Performance
- **Median views:** 2,500
- **Mean views:** 5,200
- **Range:** 45 - 125,000
- **Top performer:** "Title of Video" (125K views)
- **Underperformers:** 5 videos <500 views

## Content Themes
- Tutorial (40%) - 18 videos
- Review (30%) - 14 videos
- Live Stream (20%) - 9 videos
- Other (10%) - 5 videos

## Playlist Insights
- Avg videos per playlist: 15.7
- Largest playlist: "Tutorials" (32 videos)
- Content overlap: 3 videos in multiple playlists

## Recommendations
1. Tutorial content resonates best → consider increasing frequency
2. Upload time is consistent → good for subscriber notifications
3. Live streams underperform → consider shorter/more focused content
```

And structured data:
```
runs/<run-id>/analysis.json
{
  "upload_frequency": {
    "average_days_between": 14.2,
    "trend": "accelerating",
    "last_upload_days_ago": 3,
    "consistency_stddev": 2.1
  },
  "view_stats": {
    "median": 2500,
    "mean": 5200,
    "min": 45,
    "max": 125000,
    "p25": 800,
    "p75": 8500,
    "p90": 18000
  },
  "content_themes": {
    "Tutorial": {"count": 18, "pct": 40},
    "Review": {"count": 14, "pct": 30},
    "Live Stream": {"count": 9, "pct": 20},
    "Other": {"count": 5, "pct": 10}
  },
  "playlist_stats": {
    "avg_videos": 15.7,
    "min_videos": 5,
    "max_videos": 32,
    "total_playlists": 3,
    "overlap_count": 3
  },
  "analyzed_at": "2026-08-06T23:45:12Z"
}
```

## Exit gates

| Gate | Result |
|------|--------|
| ✓ Analysis complete | → Stage 04: Generate workflow |
| ✓ Patterns identified | → Stage 04 |

## Next stage

→ `stages/04_generate_workflow/CONTEXT.md`

---

**Note:** This stage informs workflow design. Strong patterns = automation opportunities.
