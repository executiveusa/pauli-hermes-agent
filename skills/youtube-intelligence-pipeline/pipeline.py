#!/usr/bin/env python3
"""
YouTube Intelligence Pipeline
Scrape → Transcribe → Analyze → Actionable Second Brain

Usage:
  python pipeline.py                          # use channels.yaml in cwd
  python pipeline.py --config my-channels.yaml
  python pipeline.py --skip-scrape            # re-analyze cached data
  python pipeline.py --skip-transcripts       # skip transcript fetching
  python pipeline.py --no-descriptions        # faster scrape (no per-video desc fetch)
  python pipeline.py --max-videos 50          # limit Claude API calls
"""
import argparse
import json
import sys
import time
from datetime import datetime
from pathlib import Path

import yaml


def load_config(path: str) -> dict:
    p = Path(path)
    if not p.exists():
        print(f"Config not found: {p}")
        print("Copy channels.example.yaml → channels.yaml and fill in your channels.")
        sys.exit(1)
    cfg = yaml.safe_load(p.read_text(encoding="utf-8"))
    if not cfg.get("channels"):
        print("No channels defined in config. Add at least one entry under 'channels:'.")
        sys.exit(1)
    return cfg


def _load_cached_videos(data_dir: Path) -> list[dict]:
    videos: list[dict] = []
    for f in sorted(data_dir.glob("*.json")):
        if f.stem.startswith(("enriched_", "synthesis_")):
            continue
        data = json.loads(f.read_text(encoding="utf-8"))
        if isinstance(data, list):
            videos.extend(data)
    return videos


def main() -> None:
    parser = argparse.ArgumentParser(description="YouTube Intelligence Pipeline")
    parser.add_argument("--config", default="channels.yaml", help="Path to channels.yaml")
    parser.add_argument("--skip-scrape", action="store_true", help="Skip scraping; use cached JSON from data_dir")
    parser.add_argument("--skip-transcripts", action="store_true", help="Skip transcript fetching")
    parser.add_argument("--no-descriptions", action="store_true", help="Skip per-video description pages (faster)")
    parser.add_argument("--max-videos", type=int, default=100, help="Max videos sent through Claude analysis")
    args = parser.parse_args()

    cfg = load_config(args.config)
    goals: list[str] = cfg.get("goals", [])
    output_cfg: dict = cfg.get("output", {})

    data_dir = Path(output_cfg.get("data_dir", "youtube_data"))
    output_dir = Path(output_cfg.get("output_dir", "youtube_intelligence"))
    max_videos = output_cfg.get("max_videos", args.max_videos)

    data_dir.mkdir(exist_ok=True)
    output_dir.mkdir(exist_ok=True)

    ts = datetime.now().strftime("%Y%m%d_%H%M%S")
    all_videos: list[dict] = []

    # ------------------------------------------------------------------
    # Phase 1: Scrape
    # ------------------------------------------------------------------
    if not args.skip_scrape:
        print("\n=== Phase 1: Scraping ===")
        from scrape import scrape_channel

        for ch in cfg["channels"]:
            print(f"\nChannel: {ch['name']} ({ch['url']})")
            try:
                videos = scrape_channel(ch, fetch_descriptions=not args.no_descriptions)
            except Exception as e:
                print(f"  [ERROR] Scrape failed: {e}")
                continue

            all_videos.extend(videos)
            slug = ch["url"].split("@")[-1].rstrip("/").split("/")[0][:30]
            raw_file = data_dir / f"{slug}_{ts}.json"
            raw_file.write_text(json.dumps(videos, indent=2, ensure_ascii=False))
            print(f"  {len(videos)} videos → {raw_file}")
            time.sleep(2)  # rate limit between channels
    else:
        print("\n=== Phase 1: Loading cached data ===")
        all_videos = _load_cached_videos(data_dir)
        print(f"  Loaded {len(all_videos)} videos from {data_dir}/")

    if not all_videos:
        print("No videos to process. Exiting.")
        sys.exit(0)

    # ------------------------------------------------------------------
    # Phase 2: Transcripts
    # ------------------------------------------------------------------
    if not args.skip_transcripts:
        print(f"\n=== Phase 2: Transcripts ({len(all_videos)} videos) ===")
        from transcripts import fetch_transcripts
        all_videos = fetch_transcripts(all_videos)

    enriched_file = data_dir / f"enriched_{ts}.json"
    enriched_file.write_text(json.dumps(all_videos, indent=2, ensure_ascii=False))
    print(f"\nEnriched data → {enriched_file}")

    # ------------------------------------------------------------------
    # Phase 3: Per-video Claude analysis
    # ------------------------------------------------------------------
    print(f"\n=== Phase 3: Claude Analysis (max {max_videos} videos) ===")
    from analyze import analyze_batch, cross_channel_synthesis
    analyzed = analyze_batch(all_videos, goals, max_videos=max_videos)

    # ------------------------------------------------------------------
    # Phase 4: Cross-channel synthesis
    # ------------------------------------------------------------------
    print("\n=== Phase 4: Cross-Channel Synthesis ===")
    result = cross_channel_synthesis(analyzed, goals)

    synthesis_file = data_dir / f"synthesis_{ts}.json"
    synthesis_file.write_text(json.dumps(result, indent=2, ensure_ascii=False))

    # ------------------------------------------------------------------
    # Phase 5: Outputs
    # ------------------------------------------------------------------
    print("\n=== Phase 5: Writing Outputs ===")
    from output import (
        write_second_brain,
        write_actions,
        write_content_calendar,
        write_per_video_insights,
    )
    write_second_brain(result, output_dir / f"second_brain_{ts}.md")
    write_actions(result, output_dir / f"actions_{ts}.json")
    write_content_calendar(result, output_dir / f"content_calendar_{ts}.md")
    write_per_video_insights(result, output_dir / f"insights_{ts}.md")

    # ------------------------------------------------------------------
    # Summary
    # ------------------------------------------------------------------
    syn = result.get("synthesis", {})
    print("\n" + "=" * 60)
    print("Pipeline complete.")
    print(f"  Videos scraped  : {result.get('total_videos', 0)}")
    print(f"  Videos analyzed : {result.get('analyzed_videos', 0)}")
    print(f"  Channels        : {len(result.get('channels', []))}")
    print(f"  Outputs in      : {output_dir}/")
    print()
    summary = syn.get("executive_summary", "")
    if summary:
        print("Executive summary:")
        print(f"  {summary}")


if __name__ == "__main__":
    main()
