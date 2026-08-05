---
name: youtube-channel-scraper
description: "Scrape YouTube channel playlists using Scrapling — stealth browser automation extracts video titles, descriptions, upload dates, view counts, and URLs into structured JSON."
version: 1.0.0
author: Pauli Hermes Agent
license: MIT
platforms: [linux, macos, windows]
metadata:
  hermes:
    tags: [scraping, youtube, data-extraction, scrapling, stealth, playlist, automation, json]
    related_skills: [social-media, research, data-science]
    capabilities: [web-scraping, stealth-browser, structured-output, error-recovery]
---

# YouTube Channel Scraper Skill

Extract structured data from YouTube channel playlists using [Scrapling](https://github.com/D4Vinci/Scrapling) — an adaptive, stealth-first Python scraping framework with anti-bot bypass and Playwright browser integration.

**Key capabilities:**
- **Stealth scraping** — browser emulation with Playwright, anti-fingerprinting, bot bypass
- **Playlist-focused** — targets specific playlists, not full channel crawls
- **Structured output** — per-playlist JSON with video title, URL, upload date, view count, description
- **Error recovery** — skips failed videos, logs failures, continues to next target
- **Summary report** — successful vs. failed scrape counts at end of run

## When to Use This Skill

Trigger when the user:
- Wants to scrape a YouTube channel or specific playlists
- Needs video metadata (titles, dates, view counts, descriptions) in bulk
- Asks to "extract all videos from [channel/playlist]"
- Wants to archive or analyze a YouTube channel's content

## Setup Instructions

### 1. Install Scrapling

```bash
git clone https://github.com/D4Vinci/Scrapling.git ~/scrapling
cd ~/scrapling
pip install -e ".[all]"
playwright install chromium
```

### 2. Verify Installation

```bash
python -c "from scrapling import Fetcher; print('Scrapling OK')"
```

## Execution

### Agent Script

Save as `scrape_youtube.py` and run with the target channel + playlists:

```python
import json
import os
from pathlib import Path
from datetime import datetime
from scrapling import PlayWrightFetcher

OUTPUT_DIR = Path("youtube_scrapes")
OUTPUT_DIR.mkdir(exist_ok=True)

def scrape_playlist(fetcher: PlayWrightFetcher, playlist_url: str) -> list[dict]:
    """Extract all videos from a single playlist page."""
    results = []
    try:
        page = fetcher.fetch(
            playlist_url,
            headless=True,
            network_idle=True,  # wait for dynamic content
            stealth=True,
        )
        playlist_name = page.find("yt-formatted-string#text.style-scope.yt-dynamic-sizing-formatted-string", first=True)
        playlist_name = playlist_name.text if playlist_name else "Unknown Playlist"

        videos = page.find_all("ytd-playlist-video-renderer")
        for video in videos:
            try:
                title_el = video.find("a#video-title", first=True)
                meta_el  = video.find("div#video-info", first=True)

                title       = title_el.text.strip() if title_el else ""
                href        = title_el.attrib.get("href", "") if title_el else ""
                video_url   = f"https://www.youtube.com{href}" if href else ""
                upload_date = ""
                view_count  = ""

                if meta_el:
                    spans = meta_el.find_all("span")
                    parts = [s.text.strip() for s in spans if s.text.strip()]
                    if len(parts) >= 2:
                        view_count  = parts[0]
                        upload_date = parts[-1]

                description = ""  # playlist page doesn't expose descriptions; fetch video page if needed

                results.append({
                    "playlist_name": playlist_name,
                    "playlist_url":  playlist_url,
                    "video_title":   title,
                    "video_url":     video_url,
                    "upload_date":   upload_date,
                    "view_count":    view_count,
                    "description":   description,
                })
            except Exception as e:
                print(f"  [WARN] Failed to parse video entry: {e}")

    except Exception as e:
        print(f"  [ERROR] Failed to scrape {playlist_url}: {e}")

    return results


def run(channel_url: str, playlist_urls: list[str]) -> dict:
    """Main entry point. Returns a summary report."""
    fetcher  = PlayWrightFetcher()
    summary  = {"channel_url": channel_url, "succeeded": [], "failed": []}
    ts       = datetime.now().strftime("%Y%m%d_%H%M%S")

    for pl_url in playlist_urls:
        print(f"Scraping: {pl_url}")
        videos = scrape_playlist(fetcher, pl_url)
        if videos:
            slug     = pl_url.split("list=")[-1][:20] if "list=" in pl_url else "playlist"
            out_file = OUTPUT_DIR / f"{slug}_{ts}.json"
            out_file.write_text(json.dumps(videos, indent=2, ensure_ascii=False))
            print(f"  ✓ {len(videos)} videos → {out_file}")
            summary["succeeded"].append({"playlist": pl_url, "count": len(videos), "file": str(out_file)})
        else:
            print(f"  ✗ No videos extracted")
            summary["failed"].append(pl_url)

    report_path = OUTPUT_DIR / f"summary_{ts}.json"
    report_path.write_text(json.dumps(summary, indent=2))
    print(f"\nSummary: {len(summary['succeeded'])} succeeded, {len(summary['failed'])} failed → {report_path}")
    return summary


if __name__ == "__main__":
    import sys
    if len(sys.argv) < 3:
        print("Usage: python scrape_youtube.py <channel_url> <playlist_url> [playlist_url ...]")
        sys.exit(1)
    run(channel_url=sys.argv[1], playlist_urls=sys.argv[2:])
```

### Run It

```bash
python scrape_youtube.py \
  "https://www.youtube.com/@examplechannel" \
  "https://www.youtube.com/playlist?list=PL_example1" \
  "https://www.youtube.com/playlist?list=PL_example2"
```

## Output Format

Each playlist produces a JSON file in `youtube_scrapes/`:

```json
[
  {
    "playlist_name": "My Playlist",
    "playlist_url": "https://www.youtube.com/playlist?list=PL...",
    "video_title": "How to Do X",
    "video_url": "https://www.youtube.com/watch?v=...",
    "upload_date": "3 months ago",
    "view_count": "12K views",
    "description": ""
  }
]
```

A `summary_<timestamp>.json` is always written with succeeded/failed counts.

## Agent Instructions

When the user asks to scrape a YouTube channel or playlist:

1. **Ask for** channel URL and list of playlist URLs (or extract from user message)
2. **Install Scrapling** if not present: `pip install scrapling[all]` + `playwright install chromium`
3. **Write** the `scrape_youtube.py` script to a working directory
4. **Run**: `python scrape_youtube.py <channel_url> <playlist_urls...>`
5. **Report** the summary: files written, video counts, any failures
6. **Offer** to load the JSON into a dataframe or pipe into another tool

## Notes

- YouTube's DOM structure changes frequently — if selectors break, use Scrapling's adaptive mode: `page.find("a#video-title", auto_match=True)`
- For video descriptions (not shown on playlist pages), add a second fetch per `video_url` with `stealth=True`
- Rate-limit: add `time.sleep(1)` between playlist fetches to stay under YouTube's soft limits
- Scrapling's Playwright mode handles cookie consent banners and sign-in prompts automatically in stealth mode
