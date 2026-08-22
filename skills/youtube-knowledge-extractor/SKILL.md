---
name: youtube-knowledge-extractor
description: "Extracts structured knowledge from any YouTube video or batch of videos — pulls the transcript, runs AI analysis, and outputs a structured knowledge object (summary, key takeaways, action items, code snippets, topics, category). Exports to Notion, Obsidian, Markdown, or JSON. Zero web app required — runs entirely as an agent tool chain."
version: 1.0.0
author: Pauli Hermes Agent
license: MIT
platforms: [linux, macos, windows]
triggers:
  - extract knowledge from
  - summarize this video
  - extract notes from this
  - add this to my second brain
  - save this to Notion
  - what did this video cover
  - import my watch history
entry_point: /youtube-knowledge-extractor [youtube_url | watch-history.json | url_list]
metadata:
  hermes:
    tags: [youtube, transcripts, knowledge-extraction, notion, obsidian, second-brain, llm-analysis, structured-output, icm-workflow]
    related_skills: [youtube-channel-scraper, youtube-intelligence-pipeline, note-taking, hermes-rolodex]
    capabilities: [transcript-extraction, llm-analysis, structured-output, notion-export, obsidian-export, batch-processing, auto-activation]
    activation_style: automatic-on-keyword
prerequisites:
  commands: [yt-dlp, python3]
  optionalEnv:
    - NOTION_TOKEN
    - NOTION_DATABASE_ID
    - OBSIDIAN_VAULT_PATH
    - GEMINI_API_KEY
    - ANTHROPIC_API_KEY
---

# YouTube Knowledge Extractor

Extracts structured knowledge from any YouTube video or batch of videos — pulls the transcript, runs AI analysis, and outputs a structured knowledge object (summary, key takeaways, action items, code snippets, topics, category). Exports to Notion, Obsidian, Markdown, or JSON.

**Relationship to other YouTube skills in this repo:** `youtube-channel-scraper` pulls raw channel/playlist metadata (no analysis). `youtube-intelligence-pipeline` synthesizes *across* many channels into a content-strategy second brain (trends, gaps, content calendar). This skill is the per-video unit: one video (or one batch from watch history) in, one structured knowledge object out — closer to a Readwise/Mem-style note-taker than a strategy tool. Use `youtube-intelligence-pipeline` when the question is "what should I make next"; use this skill when the question is "what did this video say and where do my notes go."

## When to Use

- User pastes a YouTube URL and says "summarize this", "extract notes from this", "what did this cover"
- "Add this to my second brain: [URL]"
- "Summarize this video and save to Notion: [URL]"
- User uploads a Google Takeout `watch-history.json` and wants it processed in bulk
- A list of YouTube URLs that need per-video extraction

## Pipeline

```
YouTube URL | watch-history.json | list of URLs
  → yt-dlp transcript extraction (free, no API key)
  → AI analysis (3,000-char cap — cost guard)
  → structured knowledge object: summary, key_takeaways, action_items, code_snippets, topics, category
  → export: Notion | Obsidian | Markdown | JSON
```

### Step 0 — Detect Input

- Single URL (`youtube.com/watch` or `youtu.be/`) → single-video flow
- Playlist URL → loop single-video flow per entry
- `watch-history.json` (Google Takeout) → bulk flow
- List of URLs → loop single-video flow
- Ambiguous → ask: "YouTube URL, watch history file, or list of URLs?"

### Step 1a — Transcript (Single Video)

Primary:

```bash
yt-dlp --write-auto-sub --sub-lang en --skip-download --sub-format vtt \
  --output "/tmp/yt_%(id)s.%(ext)s" "VIDEO_URL"
```

Strip VTT headers/timestamps with regex to get plain text.

Fallback:

```python
pip install youtube-transcript-api
# YouTubeTranscriptApi.get_transcript(video_id) — join all segments with a space
```

If both fail: `transcript = null`, continue with metadata only. Partial output beats failure.

### Step 1b — Bulk (`watch-history.json`)

- Parse the JSON array, filter entries where `titleUrl` contains `youtube.com/watch`
- Extract: `url`, `title`, `watched_at`
- Process in batches of 10, sleep 2s between batches
- Skip failed videos, log them, never crash the whole batch
- Cap: confirm with the user before processing more than 100 videos

### Step 2 — AI Analysis

Truncate transcript to **3,000 chars, hard cap, no exceptions**.

Model: configurable — a fast/cheap model by default, a stronger model for high-stakes requests. Use whatever LLM provider is already configured in this environment (see `agent/web_search_provider.py`-style provider selection elsewhere in this repo for the pattern); do not hardcode a specific vendor.

Prompt:

```
You are a knowledge extraction expert. Return JSON only — no preamble, no markdown fences.
VIDEO TITLE: {title}
CHANNEL: {channel}
TRANSCRIPT: {transcript[:3000]}
Return:
{
  "summary": "2-3 sentence plain-English summary",
  "key_takeaways": ["specific insight 1", "specific insight 2", "specific insight 3"],
  "action_items": ["concrete actionable item — omit if nothing concrete"],
  "code_snippets": [{"language": "...", "description": "...", "code": "..."}],
  "topics": ["tag1", "tag2"],
  "category": "programming|business|marketing|design|science|finance|health|productivity|creative|other",
  "difficulty": "beginner|intermediate|advanced"
}
```

Rules: takeaways must be specific (include actual numbers/names/techniques). `action_items`: 0-3 only. `code_snippets`: empty array if none. Valid JSON only.

### Step 3 — Knowledge Object

```json
{
  "schema_version": "1.0",
  "extracted_at": "ISO8601",
  "source": { "url": "", "youtube_id": "", "title": "", "channel": "", "thumbnail_url": "" },
  "transcript_available": true,
  "analysis": { "...": "Step 2 output" },
  "tags": ["from analysis.topics"],
  "category": "from analysis.category"
}
```

### Step 4 — Export (route by user request; default = Markdown)

**Markdown**
- Filename: `{YYYY-MM-DD}_{channel-slug}_{title-slug}.md`
- Save to: `./knowledge/{category}/`
- Format: H1 title, metadata block, Summary section, Key Takeaways bullets, Action Items checkboxes, code blocks if present

**Notion**
- Requires `NOTION_TOKEN` + `NOTION_DATABASE_ID` in env
- `POST https://api.notion.com/v1/pages`
- Properties: Title, Channel, URL, Category (select), Topics (multi_select), Extracted (date)
- Body blocks: Summary paragraph, Takeaways bullets, Action Items todos

**Obsidian**
- Same as Markdown but with YAML frontmatter
- Save to: `$OBSIDIAN_VAULT_PATH/YouTube/{category}/{filename}.md`

**JSON**
- Dump the knowledge object as formatted JSON to stdout or a file

## Guardrails

**Do:**
- Truncate to 3K chars before sending to any LLM
- Cache transcripts within a single run (don't re-fetch the same video twice)
- Export partial output on failure rather than nothing

**Do not:**
- Download video files — `--skip-download` always
- Send full, untruncated transcripts to an LLM
- Process more than 1000 videos without explicit user confirmation
- Fail the entire batch on a single video's error

## Failure Table

| Failure | Handling |
|---|---|
| `TRANSCRIPT_UNAVAILABLE` | Skip, log, continue |
| yt-dlp rate limit | Wait 5s, retry once, then skip |
| AI returns invalid JSON | Retry once with explicit JSON instruction, then store raw text |
| Notion 401 | Stop, surface error, ask user to check `NOTION_TOKEN` |
| Notion 400 | Log response body, skip this video |

## Output Quality Floor

Reject and retry if any of these are missing:
- `summary` (non-empty)
- `key_takeaways` (≥1 item)
- `category` (assigned)
- `source.url` (valid YouTube URL)
- `extracted_at` (ISO8601)

## Environment Variables

| Var | Required for |
|---|---|
| `NOTION_TOKEN` | Notion export only |
| `NOTION_DATABASE_ID` | Notion export only |
| `OBSIDIAN_VAULT_PATH` | Obsidian export only |
| — | No API key needed for transcript extraction itself |

Analysis needs *some* configured LLM provider (`GEMINI_API_KEY`, `ANTHROPIC_API_KEY`, or whatever is already wired into this Hermes install) — none is bundled or assumed by default.
