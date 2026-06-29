# Workflow: Construction Review

## Trigger
- Weekly (part of weekly_impact_report)
- On milestone update
- Manual: "construction review" or "build status"

## Steps

1. **Load current build plan**
   - Retrieve all `ConstructionMilestone` records for `project=indigo_azul`
   - Status: PLANNED | IN_PROGRESS | COMPLETE | BLOCKED

2. **Calculate progress**
   - % milestones complete
   - Budget spent vs. allocated (per milestone + total)
   - Timeline: on track | behind | ahead

3. **Detect funding gaps**
   - For each PLANNED or IN_PROGRESS milestone:
     - `gap = budget_allocated - budget_spent - donations_reserved`
   - Flag any gap > $500
   - Priority: milestone with nearest start date

4. **Risk analysis**
   - Weather / seasonal delays (Puerto Vallarta rainy season: June–October)
   - Supplier lead times
   - Permit status (if tracked)

5. **Generate phase report**
   - Current phase summary
   - Milestone table (status, budget, timeline)
   - Identified gaps with funding needed

6. **Trigger fundraising if gap detected**
   - If gap > threshold: auto-trigger `fundraising_campaign.md`
   - Include gap amount + milestone description in campaign brief

7. **Photo/media update** (if new photos available)
   - Log to milestone record
   - Queue for content_engine (donor story material)

8. **Emit Paperclip log**

## Output
- Construction status report (Markdown)
- Funding gap list (sorted by priority)
- Triggered campaigns (if any)
- Updated milestone records
