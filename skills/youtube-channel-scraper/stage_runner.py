#!/usr/bin/env python3
"""
YouTube Scraper — Stage Runner

Executes ICM workflow stages: parse → scrape → structure → deliver
"""

import json
import sys
import time
import re
from pathlib import Path
from datetime import datetime
from typing import Any

# Import scraper module (with fallback for different environments)
try:
    from skills.youtube_intelligence_pipeline.scrape import (
        discover_playlists,
        scrape_playlist,
        fetch_description,
        scrape_channel,
    )
except ImportError:
    # Fallback for relative imports
    import sys
    from pathlib import Path

    parent_dir = str(Path(__file__).parent.parent / "youtube-intelligence-pipeline")
    if parent_dir not in sys.path:
        sys.path.insert(0, parent_dir)

    from scrape import (
        discover_playlists,
        scrape_playlist,
        fetch_description,
        scrape_channel,
    )

OUTPUT_BASE = Path("youtube_scrapes")
OUTPUT_BASE.mkdir(exist_ok=True)


def stage_00_parse_request(user_request: str) -> dict:
    """Stage 00: Parse natural language request into structured targets."""
    print("\n[Stage 00] Parse Request")
    print(f"Input: {user_request}")

    targets = []
    preferences = {
        "fetch_descriptions": "description" in user_request.lower(),
        "fetch_transcripts": "transcript" in user_request.lower(),
        "rate_limit_req_per_min": 40,
    }

    # Extract URLs
    url_pattern = r"https?://(?:www\.)?youtube\.com/(?:@[\w-]+|playlist\?list=[\w-]+)"
    urls = re.findall(url_pattern, user_request)

    if not urls:
        return {"error": "No YouTube URLs found in request", "status": "FAIL"}

    for url in urls:
        if "@" in url:
            targets.append({"url": url, "type": "channel", "auto_discover_playlists": True})
        else:
            targets.append({"url": url, "type": "playlist", "auto_discover_playlists": False})

    output = {
        "request_id": f"req_{datetime.now().strftime('%Y%m%d_%H%M%S')}",
        "targets": targets,
        "preferences": preferences,
        "constraints": {"max_videos": None, "max_playlists": 10},
        "status": "PASS",
    }

    # Save output
    stage_output = OUTPUT_BASE / "stage_00_targets.json"
    stage_output.write_text(json.dumps(output, indent=2))

    print(f"✓ Extracted {len(targets)} target(s)")
    print(f"✓ Preferences: descriptions={preferences['fetch_descriptions']}")
    print(f"✓ Output: {stage_output}")

    return output


def stage_01_scrape_target(targets: list, preferences: dict) -> dict:
    """Stage 01: Use Scrapling to scrape all targets."""
    print("\n[Stage 01] Scrape Target")

    all_videos = []
    errors = []
    playlists_attempted = 0

    for target in targets:
        try:
            print(f"Processing: {target['url']}")

            config = {
                "url": target["url"],
                "name": target["url"].split("/")[-1],
                "playlists": [] if target["auto_discover_playlists"] else [target["url"]],
            }

            # Run full channel scrape
            videos = scrape_channel(config, fetch_descriptions=preferences["fetch_descriptions"])

            all_videos.extend(videos)
            playlists_attempted += 1
            print(f"  ✓ {len(videos)} videos extracted")
            time.sleep(1)  # Rate limit

        except Exception as e:
            err = f"Failed to scrape {target['url']}: {str(e)}"
            print(f"  ✗ {err}")
            errors.append(err)

    # Save raw output
    stage_output = OUTPUT_BASE / "stage_01_raw_videos.jsonl"
    with open(stage_output, "w") as f:
        for video in all_videos:
            f.write(json.dumps(video) + "\n")

    report = {
        "stage": "01_scrape_target",
        "started_at": datetime.now().isoformat(),
        "targets_processed": len(targets),
        "playlists_attempted": playlists_attempted,
        "videos_extracted": len(all_videos),
        "errors": errors,
        "status": "PASS" if all_videos else "FAIL",
    }

    report_file = OUTPUT_BASE / "stage_01_report.json"
    report_file.write_text(json.dumps(report, indent=2))

    print(f"✓ Total videos: {len(all_videos)}")
    print(f"✓ Output: {stage_output}")

    return report


def stage_02_structure_output() -> dict:
    """Stage 02: Clean, dedupe, and enrich scraped data."""
    print("\n[Stage 02] Structure Output")

    # Load raw videos
    raw_file = OUTPUT_BASE / "stage_01_raw_videos.jsonl"
    if not raw_file.exists():
        return {"error": "Raw videos not found, run Stage 01 first", "status": "FAIL"}

    videos = []
    with open(raw_file) as f:
        for line in f:
            videos.append(json.loads(line))

    print(f"Loaded {len(videos)} raw videos")

    # Deduplicate by video_url
    seen_urls = set()
    deduplicated = []
    for video in videos:
        if video.get("video_url") not in seen_urls:
            seen_urls.add(video["video_url"])
            deduplicated.append(video)

    removed = len(videos) - len(deduplicated)
    print(f"Deduplicated: removed {removed}, kept {len(deduplicated)}")

    # Enrich and normalize
    enriched = []
    for video in deduplicated:
        enriched.append(
            {
                "video_id": video.get("video_url", "").split("v=")[-1][:11],
                "channel_name": video.get("channel_name", ""),
                "channel_url": video.get("channel_url", ""),
                "playlist_name": video.get("playlist_name", ""),
                "playlist_url": video.get("playlist_url", ""),
                "video_title": video.get("video_title", ""),
                "video_url": video.get("video_url", ""),
                "view_count_raw": video.get("view_count", ""),
                "upload_date_raw": video.get("upload_date", ""),
                "description": video.get("description", ""),
                "scraped_at": datetime.now().isoformat(),
            }
        )

    # Generate stats
    view_counts = []
    for v in enriched:
        vc = v.get("view_count_raw", "").lower()
        if "k" in vc:
            view_counts.append(float(vc.replace("k views", "")) * 1000)
        elif "m" in vc:
            view_counts.append(float(vc.replace("m views", "")) * 1000000)

    stats = {
        "total_videos": len(enriched),
        "duplicates_removed": removed,
        "total_view_count": sum(view_counts) if view_counts else 0,
        "avg_view_count": sum(view_counts) / len(view_counts) if view_counts else 0,
    }

    # Save structured output
    videos_file = OUTPUT_BASE / "videos.json"
    videos_file.write_text(json.dumps(enriched, indent=2, ensure_ascii=False))

    stats_file = OUTPUT_BASE / "summary_stats.json"
    stats_file.write_text(json.dumps(stats, indent=2))

    print(f"✓ Enriched: {len(enriched)} videos")
    print(f"✓ Avg views: {stats['avg_view_count']:,.0f}")
    print(f"✓ Output: {videos_file}")

    return {"status": "PASS", "files": [str(videos_file), str(stats_file)]}


def stage_03_deliver_results() -> dict:
    """Stage 03: Generate user report and delivery summary."""
    print("\n[Stage 03] Deliver Results")

    videos_file = OUTPUT_BASE / "videos.json"
    stats_file = OUTPUT_BASE / "summary_stats.json"

    if not videos_file.exists():
        return {"error": "videos.json not found, run Stage 02 first", "status": "FAIL"}

    # Load results
    videos = json.loads(videos_file.read_text())
    stats = json.loads(stats_file.read_text())

    # Generate report
    report = f"""
# YouTube Scraper — Workflow Complete ✓

## Summary
- **Videos extracted:** {stats['total_videos']}
- **Duplicates removed:** {stats['duplicates_removed']}
- **Total views:** {stats['total_view_count']:,}
- **Avg views:** {stats['avg_view_count']:,.0f}

## Output Files

All files saved to: `youtube_scrapes/`

- `videos.json` — Master dataset ({stats['total_videos']} videos)
- `summary_stats.json` — Statistics and metadata

## What's Next?

### Load in Python
```python
import json
with open('youtube_scrapes/videos.json') as f:
    videos = json.load(f)
print(f"{{len(videos)}} videos loaded")
```

### Load in Pandas
```python
import pandas as pd
df = pd.read_json('youtube_scrapes/videos.json')
df.sort_values('view_count_raw', ascending=False)
```

### Share
```bash
zip -r youtube_scrapes.zip youtube_scrapes/
# Then email or upload the zip
```

---
Generated: {datetime.now().isoformat()}
"""

    report_file = OUTPUT_BASE / "DELIVERY_REPORT.md"
    report_file.write_text(report)

    print(report)
    print(f"\n✓ Report: {report_file}")
    print(f"✓ Ready to use: {videos_file}")

    return {"status": "PASS", "videos": len(videos), "report": str(report_file)}


def main():
    if len(sys.argv) < 2:
        print("Usage: python stage_runner.py '<user request>' [--stage N]")
        print("")
        print("Examples:")
        print("  python stage_runner.py 'scrape https://www.youtube.com/@examplechannel'")
        print("  python stage_runner.py 'extract videos from https://youtube.com/playlist?list=PL123'")
        print("")
        sys.exit(1)

    user_request = sys.argv[1]

    # Run all stages
    print("=" * 60)
    print("YouTube Scraper — ICM Workflow")
    print("=" * 60)

    try:
        # Stage 00
        result_00 = stage_00_parse_request(user_request)
        if result_00.get("status") == "FAIL":
            print(f"ERROR: {result_00.get('error')}")
            sys.exit(1)

        targets = result_00["targets"]
        preferences = result_00["preferences"]

        # Stage 01
        result_01 = stage_01_scrape_target(targets, preferences)
        if result_01.get("status") == "FAIL":
            print(f"ERROR: {result_01.get('error')}")
            sys.exit(1)

        # Stage 02
        result_02 = stage_02_structure_output()
        if result_02.get("status") == "FAIL":
            print(f"ERROR: {result_02.get('error')}")
            sys.exit(1)

        # Stage 03
        result_03 = stage_03_deliver_results()
        if result_03.get("status") == "FAIL":
            print(f"ERROR: {result_03.get('error')}")
            sys.exit(1)

        print("\n" + "=" * 60)
        print("✓ All stages complete")
        print("=" * 60)

    except KeyboardInterrupt:
        print("\n[INTERRUPTED] Workflow stopped by user")
        sys.exit(130)
    except Exception as e:
        print(f"\n[ERROR] {type(e).__name__}: {str(e)}")
        import traceback

        traceback.print_exc()
        sys.exit(1)


if __name__ == "__main__":
    main()
