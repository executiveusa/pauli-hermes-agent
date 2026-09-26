"""
Scrapling-based YouTube scraper.
Handles channel playlist auto-discovery, video extraction, and description fetching.
"""
import time
from typing import Any


def discover_playlists(fetcher: Any, channel_url: str) -> list[str]:
    """Find all playlist URLs from a channel's /playlists page."""
    playlists_url = channel_url.rstrip("/") + "/playlists"
    playlist_urls: list[str] = []
    try:
        page = fetcher.fetch(playlists_url, headless=True, network_idle=True, stealth=True)
        for card in page.find_all("ytd-grid-playlist-renderer"):
            link = card.find("a#thumbnail", first=True)
            if link:
                href = link.attrib.get("href", "")
                if "list=" in href:
                    playlist_urls.append(f"https://www.youtube.com{href.split('&')[0]}")
    except Exception as e:
        print(f"  [WARN] Could not discover playlists for {channel_url}: {e}")
    return playlist_urls


def scrape_playlist(fetcher: Any, playlist_url: str, channel_name: str, channel_url: str) -> list[dict[str, Any]]:
    """Scrape all video entries from a single playlist page."""
    results: list[dict[str, Any]] = []
    try:
        page = fetcher.fetch(playlist_url, headless=True, network_idle=True, stealth=True)

        name_el = page.find(
            "yt-formatted-string#text.style-scope.yt-dynamic-sizing-formatted-string",
            first=True,
        )
        playlist_name = name_el.text.strip() if name_el else "Unknown Playlist"

        for video in page.find_all("ytd-playlist-video-renderer"):
            try:
                title_el = video.find("a#video-title", first=True)
                meta_el = video.find("div#video-info", first=True)

                title = title_el.text.strip() if title_el else ""
                href = title_el.attrib.get("href", "") if title_el else ""
                video_url = (
                    "https://www.youtube.com" + href.split("&")[0] if href else ""
                )

                upload_date = ""
                view_count = ""
                if meta_el:
                    parts = [
                        s.text.strip()
                        for s in meta_el.find_all("span")
                        if s.text.strip()
                    ]
                    if len(parts) >= 2:
                        view_count = parts[0]
                        upload_date = parts[-1]

                results.append(
                    {
                        "channel_name": channel_name,
                        "channel_url": channel_url,
                        "playlist_name": playlist_name,
                        "playlist_url": playlist_url,
                        "video_title": title,
                        "video_url": video_url,
                        "upload_date": upload_date,
                        "view_count": view_count,
                        "description": "",
                        "transcript": "",
                    }
                )
            except Exception as e:
                print(f"    [WARN] Failed to parse video entry: {e}")
    except Exception as e:
        print(f"  [ERROR] Failed to scrape {playlist_url}: {e}")
    return results


def fetch_description(fetcher: Any, video: dict[str, Any]) -> dict[str, Any]:
    """Fetch full description text from the video's own page."""
    if not video.get("video_url"):
        return video
    try:
        page = fetcher.fetch(video["video_url"], headless=True, network_idle=True, stealth=True)
        desc_el = page.find("yt-attributed-string.ytd-text-inline-expander", first=True)
        if not desc_el:
            desc_el = page.find("#description-inline-expander", first=True)
        video["description"] = desc_el.text.strip() if desc_el else ""
    except Exception as e:
        print(f"    [WARN] Description fetch failed for {video.get('video_url', '')}: {e}")
    return video


def scrape_channel(channel_config: dict[str, Any], fetch_descriptions: bool = True) -> list[dict[str, Any]]:
    """
    Full channel scrape pipeline:
      1. Auto-discover playlists if none specified
      2. Scrape every playlist for video metadata
      3. Optionally fetch full descriptions per video
    """
    from scrapling import PlayWrightFetcher

    fetcher = PlayWrightFetcher()
    channel_url = channel_config["url"]
    channel_name = channel_config.get("name", channel_url)
    playlist_urls: list[str] = channel_config.get("playlists") or []

    if not playlist_urls:
        print(f"  Auto-discovering playlists for {channel_name}...")
        playlist_urls = discover_playlists(fetcher, channel_url)
        if not playlist_urls:
            # Fall back to the main channel uploads
            playlist_urls = [channel_url + "/videos"]
        print(f"  Found {len(playlist_urls)} playlist(s)")

    all_videos: list[dict[str, Any]] = []
    for pl_url in playlist_urls:
        print(f"  Scraping: {pl_url}")
        videos = scrape_playlist(fetcher, pl_url, channel_name, channel_url)
        all_videos.extend(videos)
        time.sleep(1)

    if fetch_descriptions and all_videos:
        print(f"  Fetching descriptions for {len(all_videos)} videos...")
        enriched = []
        for i, video in enumerate(all_videos):
            enriched.append(fetch_description(fetcher, video))
            if (i + 1) % 10 == 0:
                time.sleep(0.5)
        return enriched

    return all_videos
