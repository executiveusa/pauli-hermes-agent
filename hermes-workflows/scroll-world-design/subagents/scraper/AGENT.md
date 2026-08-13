# Scraper Subagent

## Role

Executes the YouTube scrape for Stage 01. Calls `skills/youtube-channel-scraper`
and returns structured video data.

## Called By

Stage 01 orchestrator (`stages/01_scrape_and_graph/CONTEXT.md`)

## Input

```json
{
  "targets": [
    { "url": "https://www.youtube.com/@bycrawford/videos", "max_videos": 20 }
  ],
  "fetch_descriptions": true,
  "output_path": "runs/current/stage_01_raw.jsonl"
}
```

## Execution

```bash
python skills/youtube-channel-scraper/stage_runner.py \
  "scrape https://www.youtube.com/@bycrawford/videos include descriptions"
```

Then move output:
```bash
cp youtube_scrapes/stage_01_raw_videos.jsonl runs/current/stage_01_raw.jsonl
cp youtube_scrapes/videos.json runs/current/stage_01_videos.json
```

## Output

- `runs/current/stage_01_raw.jsonl` — one video per line, raw
- `runs/current/stage_01_videos.json` — enriched, deduped

## Quality Check (before returning)

- Count entries in JSONL
- Verify each entry has `video_title`, `video_url`, `channel_name`
- Return `{ "video_count": N, "status": "PASS | FAIL", "errors": [] }`

## On Failure

- Rate limit (429): wait 60s, retry once
- Access denied: record in receipt, BLOCK
- Zero videos: BLOCK with reason

## Receipt Format

```json
{
  "agent": "scraper",
  "target": "https://www.youtube.com/@bycrawford/videos",
  "videos_scraped": 20,
  "status": "PASS",
  "files": ["runs/current/stage_01_raw.jsonl", "runs/current/stage_01_videos.json"],
  "timestamp": "ISO"
}
```

Write to: `runs/current/receipts/scraper.json`
