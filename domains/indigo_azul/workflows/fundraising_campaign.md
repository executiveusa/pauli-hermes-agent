# Workflow: Fundraising Campaign

## Trigger
- Funding gap detected (construction review)
- Planned campaign date reached
- Grant deadline approaching

## Steps

1. **Identify funding need**
   - Source: `construction_review.md` gap detection OR manual input
   - Quantify: amount needed, timeline, what it funds

2. **Select campaign type**
   - Emergency appeal (gap > $5k, timeline < 30 days)
   - Monthly campaign (recurring, story-driven)
   - Grant application (formal, data-heavy)
   - Crypto push (BTC/Lightning, targeted to crypto-native donors)

3. **Generate story hook**
   - Call `content_engine`: extract 1 child story tied to this need
   - If no fresh story: use milestone narrative from `construction` skill

4. **Build campaign assets**
   - Email sequence (3 touches: launch, mid, close)
   - Social posts (Instagram + Facebook — visual + caption)
   - Video script (optional — route to OpenMontage skill if needed)
   - Landing page copy (Zeffy or BTCPay hosted)

5. **Set goal + tracking**
   - Create `Campaign` record with goal_amount, dates, channel
   - Link to funding need

6. **Approval gate**
   - Queue all assets for human review
   - Gate: `EXTERNAL_PUBLISH`

7. **Launch**
   - Publish on approval
   - Monitor: daily donation tracking, conversion rate

8. **Close + report**
   - Send donor thank-you (→ `donor_update.md`)
   - Log outcomes to campaign record
   - Update build plan if funded

## Output
- Campaign record
- Email sequence (3 drafts)
- Social posts
- Updated `ConstructionMilestone` (if funded)
