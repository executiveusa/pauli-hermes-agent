# Workflow: Weekly Impact Report

## Trigger
- Every Monday 8:00 AM (scheduled)
- Manual: "generate weekly report"

## Steps

1. **Pull KPIs**
   - Children served (this week vs. last week)
   - Donations received (amount + count)
   - Construction progress (% complete vs. plan)
   - Active campaigns (raised vs. goal)
   - Donor strength distribution (ACTIVE / WARM / FADING)

2. **Detect anomalies**
   - Donation drop > 30% week-over-week → flag
   - Construction behind schedule > 1 week → flag
   - Donor strength FADING > 20% of base → flag
   - No new stories logged in 14 days → flag

3. **Propose actions** (per anomaly)
   - Donation drop → suggest campaign or donor outreach
   - Construction delay → surface funding gap, trigger review
   - Fading donors → queue gratitude engine
   - No stories → prompt field team for update

4. **Score impact**
   - Calculate current Impact Score:
     `children_served × outcome_quality × sustainability × narrative_reach`
   - Compare to last week
   - Trend: UP | FLAT | DOWN

5. **Generate report**
   - Format: Markdown
   - Sections: KPIs, Anomalies, Impact Score, Proposed Actions
   - Length: concise — scannable in 2 minutes

6. **Emit Paperclip log**
   - `paperclip.log("weekly_impact_report", "COMPLETE", report_summary)`

7. **Surface to human**
   - Deliver via configured channel (CLI / Telegram / Slack)
   - Flag any actions requiring approval

## Output
- Weekly impact report (Markdown)
- Action proposals (prioritized list)
- Paperclip log entry
