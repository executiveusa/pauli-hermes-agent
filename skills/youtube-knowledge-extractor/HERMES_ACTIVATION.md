# Hermes Agent Activation Protocol

## How Hermes Auto-Activates This Skill

When Hermes receives a YouTube URL alongside knowledge-extraction language, or a watch-history import request, it automatically:

1. **Detects request type** — Scans for "summarize/extract/save/second brain" + a YouTube URL, or a `watch-history.json` upload
2. **Routes to workflow** — Loads `skills/youtube-knowledge-extractor/SKILL.md` and follows its pipeline
3. **Runs the pipeline** — Transcript extraction → AI analysis (3K char cap) → structured knowledge object → export
4. **Reports results** — File path (Markdown/Obsidian), Notion page URL, or JSON, plus a one-line summary of what was extracted

## Detection Pattern

**Keyword combinations:**
- "summarize this video" / "extract notes from this" + a `youtube.com` or `youtu.be` URL
- "add this to my second brain" + URL
- "save this to Notion: [URL]"
- "what did this video cover" + URL
- "import my watch history" + a `watch-history.json` upload
- `/youtube-knowledge-extractor`

**Examples that activate:**
```
✓ "summarize this: https://youtube.com/watch?v=..."
✓ "add this to my second brain: https://youtu.be/..."
✓ "import my watch history" (with watch-history.json attached)
✗ "I watched a good video today" (no URL, no extraction intent)
✗ "scrape @channel's videos" (routes to youtube-channel-scraper instead)
```

## Skill Metadata

```json
{
  "youtube-knowledge-extractor": {
    "triggers": [
      "extract knowledge from",
      "summarize this video",
      "extract notes from this",
      "add this to my second brain",
      "save this to Notion",
      "what did this video cover",
      "import my watch history"
    ],
    "entry_point": "/youtube-knowledge-extractor",
    "activation_style": "automatic-on-keyword"
  }
}
```

## Direct Invocation

```
/youtube-knowledge-extractor <youtube_url>
/youtube-knowledge-extractor <url> --export notion
/youtube-knowledge-extractor watch-history.json
```

## Fallback / Manual Activation

```bash
yt-dlp --write-auto-sub --sub-lang en --skip-download --sub-format vtt \
  --output "/tmp/yt_%(id)s.%(ext)s" "VIDEO_URL"
# then run the analysis + export steps documented in SKILL.md
```

## What Hermes Knows

✓ Knows where the skill lives (`skills/youtube-knowledge-extractor/`)
✓ Knows how to trigger it (`/youtube-knowledge-extractor` or auto via keywords)
✓ Knows transcript extraction needs no API key; analysis needs *some* configured LLM provider; Notion/Obsidian export need their own env vars, all opt-in
✓ Knows to cap bulk watch-history imports at 100 videos without explicit confirmation
✓ Knows this is distinct from `youtube-channel-scraper` (raw metadata) and `youtube-intelligence-pipeline` (cross-channel strategy synthesis) — this skill is the single-video knowledge unit

## Known Gaps (as of install)

- No bundled LLM provider — analysis step requires `GEMINI_API_KEY`, `ANTHROPIC_API_KEY`, or an equivalent already configured in this Hermes install
- No `yt-dlp` bundled by default — install via `pip install yt-dlp` or the platform's package manager before first use
- Writing extracted knowledge into any external graph/second-brain store (Neo4j, etc.) is **not** part of this skill's export targets (Markdown/Obsidian/Notion/JSON only) — wiring a new export target to an external database is a separate, explicit integration task, not something this skill does implicitly
