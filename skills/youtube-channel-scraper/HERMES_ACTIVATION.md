# Hermes Agent Activation Protocol

## How Hermes Auto-Activates This Skill

When Hermes receives a request containing YouTube scraper keywords, it automatically:

1. **Detects request type** — Scans for "scrape", "extract", "download", "archive" + YouTube mentions
2. **Routes to workflow** — Calls `/youtube-channel-scraper`
3. **Stages the work** — Executes ICM workflow (Stage 00-03)
4. **Reports results** — Returns structured JSON + download links

## Detection Pattern

Hermes triggers this skill when request contains:

**Keyword combinations:**
- `scrape` + (youtube URL or @channel)
- `extract` + (youtube/playlist/videos/channel)
- `download` + (youtube/playlist)
- `archive` + (channel/playlist/youtube)
- `/youtube-channel-scraper`

**Examples that activate:**
```
✓ "scrape YouTube channel @examplechannel"
✓ "extract all videos from this playlist"
✓ "download YouTube data: https://youtube.com/..."
✓ "can you get all videos from @channel?"
✓ "/youtube-channel-scraper https://..."
✗ "I like watching YouTube" (no action keyword)
✗ "Tell me about this video" (no scrape keyword)
```

## Hermes System Prompt Addition

Add to Hermes's main system prompt or instruction file:

```
## YouTube Scraper Skill Auto-Activation

When user request matches:
  - Contains "scrape|extract|download|archive"
  - AND contains "youtube.com" URL or "@channel_name" or "playlist"

THEN:
  1. Route to skill: /youtube-channel-scraper
  2. Pass full user request as input
  3. Stage 00 will parse natural language
  4. Execute stages sequentially
  5. Report results with files and next steps

Example:
  User: "scrape @pauli_effect channel"
  → Hermes calls: /youtube-channel-scraper "scrape @pauli_effect channel"
  → Workflow: Stage 00 → 01 → 02 → 03
  → Result: JSON + report in youtube_scrapes/
```

## Direct Invocation

Users can also explicitly invoke:

```
/youtube-channel-scraper <request>
```

or 

```
@hermes scrape YouTube channel <url> [--descriptions] [--transcripts]
```

## Skill Metadata

Registry entry (see `SKILL_REGISTRY.json`):

```json
{
  "youtube-channel-scraper": {
    "triggers": [
      "scrape YouTube channel",
      "extract videos from",
      "download YouTube data",
      "get all videos from",
      "archive this channel"
    ],
    "entry_point": "/youtube-channel-scraper",
    "activation_style": "automatic-on-keyword"
  }
}
```

## No Manual Activation Needed

Unlike some skills that require `/skill-name` prefix, this one:

- Auto-detects scraper keywords
- Automatically routes requests
- Transparently orchestrates workflow
- User just asks naturally

## Skill Integration Points

For developers integrating Hermes:

1. **Check SKILL_REGISTRY.json** for `youtube-channel-scraper` entry
2. **On detection**, route request to `stage_runner.py`
3. **Stage 00** extracts URLs from natural language
4. **Workflow** runs stages 01-03 automatically
5. **Return** final report to user

## Fallback / Manual Activation

If auto-activation doesn't work:

```bash
# Direct Python
python skills/youtube-channel-scraper/stage_runner.py "scrape @channel"

# Or explicit Hermes command
/youtube-channel-scraper "scrape https://www.youtube.com/@channel"
```

## What Hermes Knows

When Hermes sees a scraper request, it:

✓ Knows where the skill lives (`skills/youtube-channel-scraper/`)  
✓ Knows how to trigger it (`/youtube-channel-scraper` or auto via keywords)  
✓ Knows it's a 4-stage workflow (check `CONTEXT.md`)  
✓ Knows outputs go to `youtube_scrapes/`  
✓ Can resume from any stage if needed  
✓ Will report full results to user  

## Example Hermes Conversation

```
User: "Can you scrape the @pauli_effect channel and get all video metadata?"

Hermes: "I'll scrape @pauli_effect for you. Let me activate the YouTube scraper workflow...

[Stage 00] Parsing request: found channel @pauli_effect
[Stage 01] Scraping with Scrapling stealth browser...
[Stage 02] Structuring 127 videos, removing 2 duplicates...
[Stage 03] Generating report...

✓ Done! 125 videos extracted.

Files ready in youtube_scrapes/:
- videos.json (125 videos)
- summary_stats.json (stats)
- DELIVERY_REPORT.md (human-readable)

What would you like to do next? Load in Pandas, export CSV, or analyze trends?"
```

---

**For Hermes developers:** See `stage_runner.py` for implementation  
**For users:** Just ask naturally; Hermes handles the rest
