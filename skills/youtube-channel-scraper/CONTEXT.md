# YouTube Scraper Workflow — Context Router

## Current Mission

Extract structured video metadata from YouTube channels/playlists using stealth browser automation.

## Workflow State

| Component | Status | Details |
|-----------|--------|---------|
| Scrapling | Ready | `pip install scrapling[all]` installed |
| Playwright | Ready | `playwright install chromium` present |
| Output dir | Ready | `youtube_scrapes/` exists |
| Run state | None | Starting fresh workflow |

## How to Use

### For Hermes

1. **User makes request** containing YouTube URL + "scrape"/"extract"/"download"
2. **Check stage** below
3. **Route to that stage's CONTEXT.md**
4. **Execute** that stage's process
5. **Move to next stage**

### For Direct Use

```bash
python -m skills.youtube-channel-scraper.icm.stages.runner --urls <url1> <url2>
```

## Active Stage Decision Tree

```
Is user request present?
  YES → Go to Stage 00 (Parse Request)
  NO  → Error: No input

Stage 00 complete?
  YES → Go to Stage 01 (Scrape Target)
  NO  → Wait for user input

Stage 01 complete?
  YES → Go to Stage 02 (Structure Output)
  NO  → Check error log

Stage 02 complete?
  YES → Go to Stage 03 (Deliver Results)
  NO  → Check structure errors

Stage 03 complete?
  YES → Workflow done, report to user
  NO  → Check delivery errors
```

## Key Files

- `CLAUDE.md` — Start here (quick reference)
- `AGENTS.md` — Who does what
- `stages/*/CONTEXT.md` — Per-stage details
- `icm/methodology.md` — Why this structure exists

## Gates & Approvals

| Action | Gate | Approval |
|--------|------|----------|
| Start scraping | None | Auto on request |
| Fetch descriptions | Optional | User preference |
| Save to disk | None | Auto |
| Report results | None | Auto |

## Failure Recovery

If a stage fails:
1. Check `stages/[stage]/output/error.log`
2. Review inputs in `stages/[stage]/input/`
3. Fix issue (usually rate limit or YouTube DOM change)
4. Retry that stage: `python stage_runner.py --stage [N]`

## Performance Notes

- Soft rate limit: 40 requests/min (Scrapling adapts)
- Per-playlist: ~5-30 sec depending on size
- Description fetch: +1-2 sec per video
- Typical run: 100 videos in 2-3 minutes
