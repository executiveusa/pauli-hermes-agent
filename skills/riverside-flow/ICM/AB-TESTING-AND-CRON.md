# Riverside Flow — A/B Testing + Cron Contract

## Objective
Use scheduled analysis to learn which packaging drives qualified action while preserving the same factual narrative.

## Allowed A/B variables
Change only one major variable per test when possible:
- hook / opening sentence;
- title;
- thumbnail;
- first 3 seconds;
- clip duration;
- caption density/style;
- CTA wording;
- publish window;
- platform-specific packaging.

Do not A/B test contradictory claims, fabricated urgency, altered quotes, misleading context, or sensitive-targeting strategies.

## Required experiment manifest
Each experiment must record:

- experiment_id
- source_recording_id
- source_clip/timestamp
- narrative_spine_version
- platform
- audience
- primary_outcome
- primary_metric
- secondary_metrics
- hypothesis
- variable_changed
- variant_A artifact/version
- variant_B artifact/version
- publish timestamps
- approval receipt(s)
- 24h metrics
- 72h metrics
- 7d metrics
- relationship/business outcome
- decision: KEEP_A | KEEP_B | INCONCLUSIVE | RETEST | STOP

## Metrics hierarchy

### Tier 1 — Business / mission outcomes
- qualified donor/sponsor conversation
- donation or pledge attributable to content
- qualified client lead
- paid conversion / booked audit / proposal
- mentor or volunteer application
- partner introduction
- youth opportunity or placement
- newsletter/signup conversion

### Tier 2 — Intent signals
- CTA clicks
- profile/site visits
- replies / DMs
- saves
- shares
- comments with identifiable intent
- meeting bookings

### Tier 3 — Consumption
- chose-to-view / swipe-away where available
- 3-second hold
- average watch time
- average percentage viewed
- completion rate
- rewatch/loop signal

### Tier 4 — Reach
- impressions
- views
- followers

Tier 4 never overrides Tier 1.

## Statistical discipline
Do not call a winner from tiny samples. When traffic is low, record a directional result as INCONCLUSIVE rather than manufacturing certainty. Prefer repeated tests over false precision.

## Cron-ready jobs
These are desired Hermes cron jobs. They are definitions, not proof that the local Hermes scheduler is currently activated.

### Job: riverside-24h-pulse
Cadence: hourly check for experiments whose 24h checkpoint is due.
Action:
1. find active Riverside Flow experiments;
2. collect available platform analytics;
3. write the 24h snapshot;
4. flag tracking failures;
5. do not declare a final winner unless a stop rule is already met;
6. send a concise operator brief.

### Job: riverside-72h-evaluation
Cadence: every 6 hours.
Action:
1. find experiments whose 72h checkpoint is due;
2. compare A/B on the declared primary metric;
3. check Tier 1/2 outcomes;
4. return KEEP_A, KEEP_B, INCONCLUSIVE, RETEST, or STOP;
5. record the reason and evidence.

### Job: riverside-7d-learning
Cadence: daily.
Action:
1. find experiments whose 7-day checkpoint is due;
2. capture final engagement and relationship/business outcomes;
3. update the pattern library;
4. identify one next experiment maximum;
5. route meaningful winning pattern to Scroll Media learning context.

### Job: riverside-followup-due
Cadence: daily morning.
Action:
1. find interviews with relationship follow-up due;
2. prioritize donor/client/partner/mentor opportunities;
3. draft next-step options;
4. never send outreach without the applicable approval rule;
5. surface stale high-value relationships.

### Job: riverside-weekly-black-swan
Cadence: weekly.
Action:
1. review the week's interviews, transcripts, opportunities, audience data, and experiments;
2. invoke black-swan-skills;
3. find one asymmetric opportunity, hidden relationship, new sellable offer, or distribution insight;
4. require evidence and state what would falsify the thesis;
5. propose one bounded experiment, not a new platform build.

## Suggested natural-language Hermes cron prompts

### 24h
"Run Riverside Flow 24h pulse. Analyze only experiments whose checkpoint is due. Compare the declared metric, preserve the locked narrative spine, update evidence receipts, and notify me only of meaningful signal, broken tracking, or a decision-ready result."

### 72h
"Run Riverside Flow 72h evaluation. Compare A/B variants on the predeclared metric plus qualified relationship/business outcomes. Return KEEP_A, KEEP_B, INCONCLUSIVE, RETEST, or STOP with evidence. Do not optimize vanity metrics over donor, client, partner, mentor, or conversion outcomes."

### 7d
"Run Riverside Flow 7-day learning. Close due experiments, write the winning/failed pattern, route reusable social learning to Scroll Media, and recommend at most one next test."

### relationship follow-up
"Run Riverside Flow relationship follow-up. Surface high-value guest follow-ups due today and draft the smallest next action that could produce a donor, sponsor, client, partner, mentor, introduction, or collaboration outcome. Do not send anything without approval."

### weekly black swan
"Run Riverside Flow weekly Black Swan review across interview transcripts, relationship records, content performance, and commercial outcomes. Find one non-obvious asymmetric opportunity and propose one cheap falsifiable test."

## Activation gate
Do not activate analytics cron jobs until the required platform analytics connectors or manual metric source are defined. A scheduled job with no reliable data source is not an automation; it is noise.
