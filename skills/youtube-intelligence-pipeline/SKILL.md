---
name: youtube-intelligence-pipeline
description: "Scrape any number of YouTube channels, extract transcripts, analyze with Claude, and produce an actionable second brain: trends, content gaps, action items, content calendar, collaborators, and tools — all in Obsidian-ready Markdown and structured JSON."
version: 1.0.0
author: Pauli Hermes Agent
license: MIT
platforms: [linux, macos, windows]
metadata:
  hermes:
    tags: [youtube, scraping, intelligence, second-brain, content-strategy, research, claude, anthropic, transcripts, obsidian]
    related_skills: [youtube-channel-scraper, hardened-longrun-subagent-harness, campaign-factory]
    capabilities: [web-scraping, transcript-extraction, llm-analysis, cross-channel-synthesis, obsidian-output, structured-json]
---

# YouTube Intelligence Pipeline

Turn any set of YouTube channels into an actionable second brain. Point it at competitors, industry leaders, or niche experts — it scrapes, transcribes, and then asks Claude to find what matters and what to do about it.

## What It Produces

For every run you get four output files in `youtube_intelligence/`:

| File | Contents |
|------|----------|
| `second_brain_<ts>.md` | Obsidian-ready note: trends, gaps, top actions, content calendar, collaborators, skills |
| `actions_<ts>.json` | Structured JSON of all action items — pipe into Notion, Linear, Zapier, etc. |
| `content_calendar_<ts>.md` | Specific video/post ideas with format, angle, and reasoning |
| `insights_<ts>.md` | Per-video breakdown: topic, insights, tools, actions, content ideas |

## When to Use This Skill

- "Scrape these YouTube channels and tell me what to do with it"
- "Find content gaps in my niche"
- "Build me a content calendar from competitor research"
- "What are the trends in [niche] on YouTube right now?"
- "Who should I collaborate with in [space]?"
- "I want to turn YouTube research into my second brain"

## Setup

### 1. Install dependencies

```bash
pip install scrapling[all] youtube-transcript-api anthropic pyyaml
playwright install chromium
```

### 2. Configure channels

```bash
cp skills/youtube-intelligence-pipeline/channels.example.yaml channels.yaml
# Edit channels.yaml: add your target channels and goals
```

### 3. Set environment

```bash
# Uses your existing Hermes/NIM proxy if set
export ANTHROPIC_BASE_URL=http://31.220.58.212:8082   # or unset for direct Anthropic
export ANTHROPIC_API_KEY=your-key-or-dummy
```

### 4. Run

```bash
cd skills/youtube-intelligence-pipeline
python pipeline.py --config ../../channels.yaml
```

## channels.yaml Structure

```yaml
channels:
  - name: "My Competitor"
    url: "https://www.youtube.com/@competitor"
    playlists: []          # empty = auto-discover all playlists
    priority: high

  - name: "Industry Leader"
    url: "https://www.youtube.com/@leader"
    playlists:
      - "https://www.youtube.com/playlist?list=PLxxx"

goals:
  - "Find content gaps to fill with original videos"
  - "Identify tools worth reviewing"
  - "Spot trends before they peak"
  - "Generate a 3-month content calendar"
  - "Find collaborators worth reaching out to"

output:
  data_dir: "youtube_data"
  output_dir: "youtube_intelligence"
  max_videos: 100           # each = 1 Claude API call
```

## Pipeline Stages

```
channels.yaml
     │
     ▼
Phase 1: Scrape (Scrapling + Playwright)
  • Auto-discover all playlists per channel
  • Extract: title, URL, upload date, view count
  • Fetch full descriptions per video
     │
     ▼
Phase 2: Transcripts (youtube-transcript-api)
  • Pull auto-captions for each video
  • Multi-language fallback (en → en-US → en-GB)
     │
     ▼
Phase 3: Per-Video Analysis (Claude)
  • Topic, insights, tools, techniques
  • Actions matched against your goals
  • Content ideas inspired by each video
     │
     ▼
Phase 4: Cross-Channel Synthesis (Claude)
  • Trends across all channels
  • Content gaps = your opportunities
  • Top 10 prioritized action items
  • 15 content calendar ideas with angles
  • Collaborators, tools, skills to develop
     │
     ▼
Phase 5: Outputs
  • second_brain.md (Obsidian-ready)
  • actions.json (structured, pipeable)
  • content_calendar.md
  • insights.md (per-video)
```

## CLI Options

```bash
python pipeline.py --help

Options:
  --config PATH          Path to channels.yaml (default: channels.yaml)
  --skip-scrape          Re-analyze cached data without scraping again
  --skip-transcripts     Skip transcript fetching (faster, less context for Claude)
  --no-descriptions      Skip per-video description pages (much faster scrape)
  --max-videos INT       Max videos to analyze with Claude (default: 100)
```

## Running on a Schedule (Hermes Routine)

Set up a weekly intelligence run via the Claude Code Remote trigger system:

```python
# In a Claude Code session:
mcp__Claude_Code_Remote__create_trigger(
    name="YouTube Weekly Intelligence",
    cron_expression="0 8 * * 1",   # every Monday at 8 AM UTC
    prompt="""
Run the YouTube intelligence pipeline:
  cd /path/to/pauli-hermes-agent/skills/youtube-intelligence-pipeline
  python pipeline.py --config ../../channels.yaml --skip-scrape=false
Then summarize the top 5 actions from the new second_brain.md file.
""",
    create_new_session_on_fire=True,
    notifications={"push": True}
)
```

## Obsidian Integration

Drop the output directory into your Obsidian vault:

```bash
# Symlink outputs into your vault
ln -s $(pwd)/youtube_intelligence ~/Documents/Obsidian/YouTube\ Research
```

The `second_brain_*.md` files include YAML frontmatter with tags, making them instantly searchable and linkable from other notes.

## Feeding Actions Into Notion

The `actions_*.json` file has a stable structure you can POST to Notion via their API or Zapier:

```json
{
  "top_actions": [
    {"action": "...", "reason": "...", "urgency": "now|soon|later"},
    ...
  ],
  "content_calendar": [...],
  "collaborators": [...],
  "tools": [...],
  "gaps": [...],
  "trends": [...]
}
```

## Cost Estimate

- **Scraping**: free (Scrapling + local Playwright)
- **Transcripts**: free (youtube-transcript-api)
- **Claude analysis**: ~1 API call per video + 1 synthesis call
  - 100 videos × ~800 tokens avg ≈ 80K tokens input + 100K output ≈ $0.30–$1.50 depending on model
  - Through NIM proxy: **$0**

## File Structure

```
skills/youtube-intelligence-pipeline/
├── SKILL.md                   # this file
├── pipeline.py                # main orchestrator (CLI entry point)
├── scrape.py                  # Scrapling-based scraper
├── transcripts.py             # youtube-transcript-api integration
├── analyze.py                 # Claude per-video + synthesis analysis
├── output.py                  # Markdown + JSON writers
└── channels.example.yaml      # config template
```

## Agent Instructions

When the user asks to "run the YouTube pipeline" or "scrape channels and give me what to do":

1. Ask for (or extract from message): target channel URLs, their goals
2. Write/update `channels.yaml` with the channels and goals
3. Run: `python pipeline.py --config channels.yaml`
4. Read `youtube_intelligence/second_brain_*.md`
5. Present the executive summary, top actions, and content calendar to the user
6. Offer to push outputs to Notion or Obsidian
