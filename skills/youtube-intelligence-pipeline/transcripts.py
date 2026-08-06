"""
Transcript extraction for YouTube videos using youtube-transcript-api.
Falls back gracefully when transcripts are disabled or unavailable.
"""
import re
import time
from typing import Any


def _extract_video_id(url: str) -> str:
    match = re.search(r"(?:v=|youtu\.be/)([a-zA-Z0-9_-]{11})", url)
    return match.group(1) if match else ""


def fetch_transcripts(
    videos: list[dict[str, Any]],
    languages: list[str] | None = None,
    delay: float = 0.3,
) -> list[dict[str, Any]]:
    """
    Attach transcript text to each video dict.
    Tries the preferred languages in order; silently skips if unavailable.
    """
    try:
        from youtube_transcript_api import (
            YouTubeTranscriptApi,
            NoTranscriptFound,
            TranscriptsDisabled,
        )
    except ImportError:
        print(
            "  [SKIP] youtube-transcript-api not installed.\n"
            "         Run: pip install youtube-transcript-api"
        )
        return videos

    langs = languages or ["en", "en-US", "en-GB"]
    succeeded = 0
    failed = 0

    for video in videos:
        video_id = _extract_video_id(video.get("video_url", ""))
        if not video_id:
            continue
        try:
            entries = YouTubeTranscriptApi.get_transcript(video_id, languages=langs)
            video["transcript"] = " ".join(e["text"] for e in entries)
            succeeded += 1
        except (NoTranscriptFound, TranscriptsDisabled):
            failed += 1
        except Exception as e:
            print(f"  [WARN] Transcript error for {video_id}: {e}")
            failed += 1
        time.sleep(delay)

    print(f"  Transcripts: {succeeded} fetched, {failed} unavailable")
    return videos
