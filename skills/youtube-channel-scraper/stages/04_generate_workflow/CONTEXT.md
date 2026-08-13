# Stage 04: Generate Workflow Spec

## Input

Required:
- `runs/<run-id>/analysis.json` (patterns from stage 03)
- `runs/<run-id>/processed.json` (video metadata from stage 02)

Optional:
- User-provided workflow intent (e.g., "automate content review", "flag underperformers")

## Process

1. **Identify automation opportunities from patterns**
   - High-frequency uploads → Schedule regular monitoring agent
   - Consistent themes → Tag/categorize agent
   - View performance variance → Alert agent for underperformers
   - Seasonal patterns → Batch processing agent

2. **Call agent-workflow-builder skill**
   - Input: Analysis findings + metadata
   - Generate A2A protocol workflow spec
   - Include guardrails from `guardrails/scraping-safety.md`
   - Output: `workflow_spec.json` (strict Google A2A format)

3. **Workflow example: Content Monitor Agent**
   ```
   - Trigger: Daily (if upload frequency supports it)
   - Input: Latest video URLs from channel
   - Process: Scrape new videos, compare against historical patterns
   - Action: Alert if view growth underperforms (< median by 2 sigma)
   - Output: Report + recommendations
   ```

4. **Workflow example: Content Categorizer Agent**
   ```
   - Trigger: When new video appears in channel
   - Input: Video metadata (title, description)
   - Process: Classify by theme using analysis patterns
   - Action: Tag video, update inventory
   - Output: Tagged video record
   ```

5. **Validate A2A protocol**
   - Check required fields (name, description, capabilities)
   - Verify guardrails are present
   - Test workflow structure against A2A spec

## Output

File to write:
```
runs/<run-id>/workflow_spec.json
{
  "name": "YouTube Channel Monitor",
  "description": "Automated monitoring and analysis of [channel name] videos",
  "version": "1.0.0",
  "agent_type": "content-monitor",
  "trigger": {
    "type": "schedule",
    "interval": "daily",
    "reasoning": "Matches observed upload frequency (14-day avg)"
  },
  "capabilities": [
    "scrape-youtube-playlists",
    "normalize-metadata",
    "analyze-performance",
    "alert-on-anomaly"
  ],
  "guardrails": {
    "rate_limit": "40 requests/minute",
    "youtube_compliance": "public-playlists-only",
    "data_retention": "ephemeral (7 days)",
    "error_handling": "continue-on-partial-failure"
  },
  "input_spec": {
    "channel_url": "https://www.youtube.com/@channel",
    "lookback_days": 30
  },
  "output_spec": {
    "alert_threshold": "views < median - 2*stddev",
    "report_format": "json",
    "report_destination": "runs/<run-id>/"
  },
  "learned_patterns": {
    "upload_frequency_days": 14.2,
    "top_themes": ["Tutorial (40%)", "Review (30%)"],
    "view_median": 2500,
    "view_stddev": 1850
  },
  "next_steps": [
    "Deploy workflow to Hermes agent",
    "Monitor first 7 days for false positives",
    "Tune alert thresholds based on live data"
  ],
  "generated_at": "2026-08-06T23:45:12Z",
  "generated_by": "youtube-channel-scraper/stage-04"
}
```

And handoff summary:
```
runs/<run-id>/workflow_handoff.md
# Workflow Generation Complete

## What we learned
- [channel name] uploads every 14.2 days (highly consistent)
- Tutorial content dominates (40%) and performs well
- Live streams underperform (30% below median views)

## Automated workflow created
**Type:** Content Monitor Agent
**Trigger:** Daily (aligns with channel updates every 2 weeks)
**Actions:**
- Scrape new videos
- Compare against historical patterns
- Alert if view growth underperforms by >2 standard deviations
- Generate daily performance report

## Next: Deploy & Monitor
1. Deploy workflow_spec.json to Hermes agent
2. Monitor alerts for first 7 days
3. Tune thresholds (current: 2 sigma) if too many false positives
4. Archive successful runs to runs/<run-id>/

## Files ready for handoff
- workflow_spec.json (A2A protocol, ready to deploy)
- analysis.json (learned patterns)
- processed.json (historical baseline)
```

## Exit gates

| Gate | Result |
|------|--------|
| ✓ Workflow spec generated + validated | → Agent deployment (external) |
| ✓ A2A protocol verified | → Ready for Hermes integration |
| ✗ Validation fails | → Fix spec, re-validate |

## Next stage

**External handoff to:** `Hermes Agent` or CI/CD pipeline for workflow deployment

Or loop back to Stage 03 if user wants to refine patterns.

---

**Note:** This stage uses the `agent-workflow-builder` skill which:
- Generates strict A2A protocol JSON
- Includes guardrails from codebase
- Auto-detects workflow category (content-monitor, categorizer, etc.)
- Produces Supabase metrics logging hooks
- Generates GitHub Actions cron for autonomous deployment
