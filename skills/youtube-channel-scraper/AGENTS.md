# Agent Roles & Responsibilities

## Hermes Main Agent

**Role:** Workflow router and user proxy

**On scraper request:**
1. Route to Stage 00 (Parse Request)
2. Pass result to Stage 01 (Scrape Target)
3. Orchestrate stages sequentially
4. Report final summary to user

**Activation triggers:**
- "scrape YouTube channel"
- "extract videos from"
- "download YouTube data"
- "get all videos from"
- "archive this channel"
- "/youtube-channel-scraper"
- Any request containing "youtube" + "scrape"/"extract"/"download"

## Stage Executor

**Role:** Independent stage processor

Each stage:
- Reads inputs from prior stage
- Executes defined process
- Writes outputs to `stage/*/output/`
- Reports completion with gates

## User

**Role:** Request origin, approval authority

Provides:
- YouTube URLs to scrape
- Preferences (descriptions: yes/no, transcripts: optional)
- Constraints (rate limit, max videos)

Approves:
- Scope changes mid-workflow
- Long-running operations

## Verifier

**Role:** QA on outputs

- Validates JSON structure
- Spot-checks video URLs
- Confirms file counts match metadata
- Reports any decode errors
