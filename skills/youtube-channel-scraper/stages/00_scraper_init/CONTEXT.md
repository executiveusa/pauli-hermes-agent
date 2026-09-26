# Stage 00: Scraper Initialization

## Input

User provides:
- (Optional) Channel URL for context
- (Optional) List of playlist URLs to scrape

Or this stage runs as a dependency check before any scrape.

## Process

1. **Check Python** — Require Python 3.9+
   ```bash
   python --version
   ```

2. **Check/Install Scrapling**
   ```bash
   python -c "from scrapling import PlayWrightFetcher; print('Scrapling OK')"
   # If missing:
   pip install scrapling[all]
   ```

3. **Check/Install Playwright**
   ```bash
   playwright install chromium
   ```

4. **Verify playgrounds work**
   ```bash
   python -c "
   from scrapling import PlayWrightFetcher
   fetcher = PlayWrightFetcher()
   page = fetcher.fetch('https://www.youtube.com')
   print(f'Page title: {page.title}')
   "
   ```

5. **Create run directory**
   ```
   runs/YYYYMMDD-HHMMSS-<channel-slug>/
   ```

6. **Log initialization**
   ```
   runs/<run-id>/init.log
   - Python version
   - Scrapling version
   - Playwright version
   - Channel URL (if provided)
   - Expected playlist count
   ```

## Output

Success → `runs/<run-id>/init.log` + environment ready

File to write:
```
runs/<run-id>/env_status.json
{
  "python_version": "3.11.2",
  "scrapling_version": "x.y.z",
  "playwright_version": "x.y.z",
  "status": "ready",
  "timestamp": "2026-08-06T23:45:12Z"
}
```

## Exit gates

| Gate | Result |
|------|--------|
| ✓ All deps installed + verified | → Stage 01: Execute scrape |
| ✗ Scrapling install fails | → Retry with `pip install --upgrade scrapling[all]` |
| ✗ Playwright install fails | → Run `playwright install chromium` manually, then retry |
| ✗ Network blocked to YouTube | → Escalate to user (firewall/VPN issue) |

## Next stage

→ `stages/01_scrape_playlist/CONTEXT.md` (user provides URLs)

---

**Note:** This stage is idempotent. Safe to run multiple times.
