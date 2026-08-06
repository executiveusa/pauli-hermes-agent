# YouTube Scraper ICM Quick Start

The scraper now has a complete **Interpretable Context Methodology (ICM)** folder structure. This means the folder IS the orchestration framework.

## Folder Structure

```
youtube-channel-scraper/
├── CLAUDE.md                    # Start here (agent setup)
├── AGENTS.md                    # Agent identity + behavior
├── CONTEXT.md                   # Stage router (intent → stage mapping)
├── ICM_QUICKSTART.md            # This file
├── SKILL.md                     # Original skill definition
│
├── icm/
│   ├── methodology.md           # ICM explanation
│   └── _config/
│       └── scraper-config.py    # Reusable Scrapling config
│
├── guardrails/
│   └── scraping-safety.md       # Rate limits, compliance, error handling
│
├── stages/                      # 5 self-contained stages
│   ├── 00_scraper_init/
│   │   ├── CONTEXT.md          # Check dependencies
│   │   └── output/
│   ├── 01_scrape_playlist/
│   │   ├── CONTEXT.md          # Execute scrape
│   │   └── output/
│   ├── 02_process_metadata/
│   │   ├── CONTEXT.md          # Normalize + deduplicate
│   │   └── output/
│   ├── 03_analyze_patterns/
│   │   ├── CONTEXT.md          # Find trends, themes
│   │   └── output/
│   └── 04_generate_workflow/
│       ├── CONTEXT.md          # Create A2A workflow spec
│       └── output/
│
└── runs/                        # Working artifacts
    └── YYYYMMDD-HHMMSS-<slug>/
        ├── init.log
        ├── metadata.json        # Raw scraped data
        ├── processed.json       # Normalized + deduplicated
        ├── analysis.json        # Patterns learned
        ├── workflow_spec.json   # A2A protocol workflow
        └── workflow_handoff.md  # Summary for next stage
```

## How to Use It

### 1. Entry Point: User provides intent

```
User: "Scrape my YouTube channel and turn it into a workflow"
```

### 2. Open CONTEXT.md (the router)

Tells you which stage to enter:
- "Scrape + analyze" → Stages 01 → 02 → 03 → 04
- "Just check setup" → Stage 00
- "Analyze existing data" → Stage 03 (skip to 03, bring your own data)

### 3. Enter the first stage

Read `stages/<stage>/CONTEXT.md` — each stage has:
- **Input** — what you need
- **Process** — what to do (step by step)
- **Output** — files to write
- **Exit gates** — conditions for next stage
- **Next stage** — where to go

### 4. Execute and write artifacts

Example: Stage 01 (scrape)
```bash
python scrape_youtube.py \
  "https://www.youtube.com/@mychannel" \
  "https://www.youtube.com/playlist?list=PL1"

# Creates: youtube_scrapes/PL1_20260806_234512.json
# Then move to: runs/<run-id>/metadata.json
```

### 5. Move to next stage

Stage 01 → Stage 02 → Stage 03 → Stage 04

Each stage's output becomes the next stage's input.

### 6. Final output

Stage 04 generates: `runs/<run-id>/workflow_spec.json`

A complete A2A protocol workflow ready to deploy to Hermes agent.

---

## Example: Full Run

```bash
# User: "Scrape @MyCoolChannel playlists and create a monitor workflow"

# Stage 00: Init
cd ~/pauli-hermes-agent/skills/youtube-channel-scraper
python -c "from scrapling import PlayWrightFetcher; print('OK')"

# Stage 01: Scrape
python scrape_youtube.py \
  "https://www.youtube.com/@MyCoolChannel" \
  "https://www.youtube.com/playlist?list=PL_tutorials" \
  "https://www.youtube.com/playlist?list=PL_reviews"
# → Creates: youtube_scrapes/PL_tutorials_20260806_234512.json (32 videos)
# → Creates: youtube_scrapes/PL_reviews_20260806_234512.json (18 videos)
# → Total: 50 videos scraped

# Stage 02: Process
# Python script normalizes timestamps, deduplicates, validates URLs
# → Output: runs/20260806-234512-MyCoolChannel/processed.json (48 unique videos)

# Stage 03: Analyze
# Compute upload frequency, view stats, theme distribution
# → Output: runs/20260806-234512-MyCoolChannel/analysis.json
#   - Upload frequency: 7.2 days (consistent)
#   - Top theme: Tutorial (52%)
#   - Median views: 5,200

# Stage 04: Generate Workflow
# Call agent-workflow-builder skill
# → Output: runs/20260806-234512-MyCoolChannel/workflow_spec.json
# {
#   "name": "MyCoolChannel Content Monitor",
#   "trigger": "daily",
#   "capabilities": ["scrape-youtube", "analyze-performance", "alert-on-anomaly"],
#   "learned_patterns": {
#     "upload_frequency_days": 7.2,
#     "top_themes": ["Tutorial (52%)", "Review (30%)"],
#     "view_median": 5200
#   }
# }

# Ready to deploy!
```

---

## Key Concepts

### Run ID
Every execution gets a unique ID:
```
YYYYMMDD-HHMMSS-<channel-slug>
Example: 20260806-234512-MyCoolChannel
```

All artifacts for that run go to: `runs/<run-id>/`

### Stages are Self-Contained
Each stage's `CONTEXT.md` has everything you need:
- What to input
- How to process
- What to output
- When to exit
- Where to go next

### Guardrails Embedded
`guardrails/scraping-safety.md` specifies:
- Rate limits (40 req/min)
- YouTube compliance (public playlists only)
- Error handling (log and continue)
- Data privacy (no subscriber data)

### Workflow Generation
Stage 04 uses the `agent-workflow-builder` skill to convert findings into:
- A2A protocol JSON
- Guardrails + capabilities
- Next-stage recommendations
- GitHub Actions cron hooks

---

## For Agents Using This

**When user says:** "Scrape and create workflow"

1. Read `CONTEXT.md` → identifies full path (01 → 02 → 03 → 04)
2. Read `stages/00/CONTEXT.md` → verify dependencies
3. Loop through stages:
   - Read stage CONTEXT.md
   - Execute process (follow steps)
   - Write outputs to `runs/<run-id>/`
   - Check exit gates
   - Move to next stage
4. Final handoff: `runs/<run-id>/workflow_handoff.md` → ready for deployment

**When user says:** "Just analyze this data"

1. Read `CONTEXT.md` → skip to stage 03
2. Provide your own `processed.json`
3. Execute analysis
4. Generate workflow from findings

---

## Quick Reference

| Stage | Purpose | Input | Output |
|-------|---------|-------|--------|
| 00 | Check setup | — | env_status.json |
| 01 | Scrape playlists | Channel URL, playlist URLs | metadata.json |
| 02 | Normalize data | metadata.json | processed.json |
| 03 | Find patterns | processed.json | analysis.json |
| 04 | Create workflow | analysis.json | workflow_spec.json |

---

## Next Steps

1. **Deploy:** Use `agent-workflow-builder` skill on stage 04 output
2. **Monitor:** Watch workflow_handoff.md for recommendations
3. **Refine:** If patterns need adjustment, loop back to stage 03
4. **Archive:** Save successful runs for future reference
