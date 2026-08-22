# YouTube Knowledge Extractor

## Quick Start

```bash
/youtube-knowledge-extractor <youtube_url>
# Or: "summarize this video: <url>"
# Or: "add this to my second brain: <url>"
# Or: upload watch-history.json and say "import my watch history"
```

Hermes auto-activates on the triggers in `SKILL.md`. No setup required for transcript extraction; export targets (Notion/Obsidian) need their own env vars.

## What This Does

One video (or one batch from watch history) in, one structured knowledge object out:

1. **Transcript** — `yt-dlp` auto-subs, falls back to `youtube-transcript-api`; metadata-only if both fail
2. **Analyze** — truncated (3K char cap) transcript through whatever LLM provider is configured
3. **Structure** — summary, key takeaways, action items, code snippets, topics, category
4. **Export** — Markdown (default), Notion, Obsidian, or JSON

## Authority

- **Automatic:** Read-only transcript fetch + analysis on explicit or keyword-triggered request
- **Gated:** Notion export requires `NOTION_TOKEN` + `NOTION_DATABASE_ID`; Obsidian export requires `OBSIDIAN_VAULT_PATH` — both opt-in, never assumed
- **Gated:** Bulk watch-history import over 100 videos requires explicit user confirmation before running

## Output Location

- Default (Markdown): `./knowledge/{category}/{date}_{channel}_{title}.md`
- Obsidian: `$OBSIDIAN_VAULT_PATH/YouTube/{category}/`
- Notion: pushed directly to the configured database
- JSON: stdout or file, per request

## Related Skills

- `skills/youtube-channel-scraper/` — raw channel/playlist metadata, no analysis
- `skills/youtube-intelligence-pipeline/` — cross-channel content-strategy synthesis (trends, gaps, content calendar); use that skill for "what should I make next," this one for "what did this video say and where do my notes go"
- `skills/note-taking/`, `skills/hermes-rolodex/` — downstream note/memory destinations

See `HERMES_ACTIVATION.md` for the full trigger list and detection pattern.
