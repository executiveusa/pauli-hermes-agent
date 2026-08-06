# YouTube Scraper — Quick Start

## 🚀 One-Minute Setup

```bash
# 1. Install dependencies (if needed)
pip install scrapling[all]
playwright install chromium

# 2. Run the scraper
python -m skills.youtube_channel_scraper.stage_runner \
  "scrape https://www.youtube.com/@examplechannel"

# 3. Results appear in youtube_scrapes/
ls youtube_scrapes/
# videos.json ✓
# summary_stats.json ✓
# DELIVERY_REPORT.md ✓
```

## Natural Language (Easiest)

Just ask Hermes naturally:

```
"Can you scrape all videos from @examplechannel?"
```

or

```
"Extract videos from this playlist: https://www.youtube.com/playlist?list=PLxxxxx"
```

Hermes auto-activates the scraper workflow.

## Command Line (Advanced)

```bash
# Scrape a channel
python stage_runner.py "scrape https://www.youtube.com/@examplechannel"

# Scrape a specific playlist
python stage_runner.py "extract https://www.youtube.com/playlist?list=PLxxxxx"

# Include descriptions
python stage_runner.py "scrape @examplechannel, include descriptions"

# Multiple playlists
python stage_runner.py "scrape these playlists: url1 url2 url3"
```

## Output

Everything goes to `youtube_scrapes/`:

```
youtube_scrapes/
├── videos.json              # All videos (JSON)
├── summary_stats.json       # Stats: count, views, etc
├── stage_01_raw_videos.jsonl # Raw output (for debugging)
└── DELIVERY_REPORT.md       # Human-readable report
```

## What You Get

Each video entry has:
- `video_id` — YouTube video ID
- `video_title` — Title
- `video_url` — Full YouTube link
- `view_count_raw` — View count (e.g., "12K views")
- `upload_date_raw` — Upload date (e.g., "3 months ago")
- `description` — Full video description
- `channel_name`, `playlist_name` — Context

## Load in Python

```python
import json
import pandas as pd

# Load as JSON
with open('youtube_scrapes/videos.json') as f:
    videos = json.load(f)
print(f"{len(videos)} videos")

# Or as Pandas
df = pd.read_json('youtube_scrapes/videos.json')

# Analyze
top_videos = df.nlargest(10, 'view_count_raw')
print(top_videos[['video_title', 'view_count_raw']])
```

## Share Results

```bash
# Zip everything
zip -r my_youtube_data.zip youtube_scrapes/

# Or export to CSV (via Pandas)
pd.read_json('youtube_scrapes/videos.json').to_csv('videos.csv')
```

## Troubleshooting

**Q: "No YouTube URLs found"**
- A: Make sure URL is in format: `youtube.com/@channel` or `/playlist?list=`

**Q: "Rate limit (429)"**
- A: Normal, scraper waits 60s and retries automatically

**Q: Descriptions are empty**
- A: Add "descriptions" or "description" to your request

**Q: Want transcripts too?**
- A: Use `yt-dlp --write-auto-subs https://youtube.com/watch?v=...` on individual videos

## Workflow Stages (Transparent & Resumable)

If something fails, each stage is independent:

| Stage | What | Duration |
|-------|------|----------|
| 00 | Parse request | <1s |
| 01 | Scrape with Scrapling | 1-5m |
| 02 | Structure & dedupe | <10s |
| 03 | Report & deliver | <5s |

Details: See `CONTEXT.md` for router or `icm/stages/[00-03]/CONTEXT.md` for each stage.

## Need More?

- **Deep dive:** `CLAUDE.md`
- **How it works:** `CONTEXT.md`
- **Stage details:** `icm/stages/`
- **GitHub:** `https://github.com/executiveusa/pauli-hermes-agent`

---

**Made by:** Pauli Hermes Agent  
**Last updated:** 2026-01-15
