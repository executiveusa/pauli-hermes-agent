# Scrapling Configuration for YouTube Channel Scraper

from pathlib import Path
from datetime import datetime

class ScraperConfig:
    """Reusable Scrapling configuration with safety guardrails."""

    # Rate limiting (respects guardrails/scraping-safety.md)
    DELAY_BETWEEN_REQUESTS = 1.0  # seconds
    MAX_REQUESTS_PER_MINUTE = 40
    BATCH_SIZE = 5  # max playlists per run

    # Timeouts
    PAGE_LOAD_TIMEOUT = 30  # seconds
    NETWORK_IDLE_TIMEOUT = 5  # seconds

    # Stealth mode (anti-bot bypass)
    STEALTH_MODE = True
    HEADLESS = True
    NETWORK_IDLE = True

    # Output
    OUTPUT_DIR = Path("youtube_scrapes")

    @classmethod
    def get_fetcher(cls):
        """Factory for PlayWrightFetcher with safety defaults."""
        from scrapling import PlayWrightFetcher  # type: ignore[import-not-found]
        return PlayWrightFetcher()

    @classmethod
    def get_timestamp(cls):
        """ISO8601 timestamp for run ID."""
        return datetime.now().strftime("%Y%m%d_%H%M%S")

    @classmethod
    def get_run_id(cls, channel_slug):
        """Generate run ID."""
        return f"{cls.get_timestamp()}-{channel_slug}"

    @staticmethod
    def extract_playlist_slug(url):
        """Extract playlist slug from YouTube URL."""
        if "list=" in url:
            return url.split("list=")[1][:20]
        return "playlist"

    @staticmethod
    def extract_video_id(url):
        """Extract video ID from YouTube video URL."""
        if "v=" in url:
            return url.split("v=")[1][:11]
        return None

    @staticmethod
    def normalize_view_count(view_str):
        """Convert '12K views' → 12000."""
        if not view_str:
            return 0
        view_str = view_str.lower().replace("views", "").strip()
        multipliers = {"k": 1000, "m": 1000000, "b": 1000000000}
        for suffix, mult in multipliers.items():
            if suffix in view_str:
                return int(float(view_str.replace(suffix, "")) * mult)
        return int(float(view_str)) if view_str.isdigit() else 0

    @staticmethod
    def approximate_date(relative_str):
        """Convert '3 months ago' → ISO8601 date (estimated)."""
        from datetime import timedelta
        today = datetime.now()

        if not relative_str:
            return today.isoformat()[:10]

        relative_str = relative_str.lower()

        if "day" in relative_str:
            days = int(relative_str.split()[0]) if relative_str[0].isdigit() else 1
            return (today - timedelta(days=days)).isoformat()[:10]
        elif "week" in relative_str:
            weeks = int(relative_str.split()[0]) if relative_str[0].isdigit() else 1
            return (today - timedelta(weeks=weeks)).isoformat()[:10]
        elif "month" in relative_str:
            months = int(relative_str.split()[0]) if relative_str[0].isdigit() else 1
            return (today - timedelta(days=months*30)).isoformat()[:10]
        elif "year" in relative_str:
            years = int(relative_str.split()[0]) if relative_str[0].isdigit() else 1
            return (today - timedelta(days=years*365)).isoformat()[:10]

        return today.isoformat()[:10]


if __name__ == "__main__":
    print(f"Run ID example: {ScraperConfig.get_run_id('my-channel')}")
    print(f"View normalize: {ScraperConfig.normalize_view_count('12K views')}")
    print(f"Date approx: {ScraperConfig.approximate_date('3 months ago')}")
