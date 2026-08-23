---
name: youtube-channel-operator
description: ICM-governed YouTube channel launch and growth operator for Hermes with ADHD intake, canonical story routing, research, channel setup, asset generation, browser-assisted configuration, starter content, verification, monetization readiness, and handoff.
version: 1.0.0
entry_point: /youtube-channel-operator
---

# YouTube Channel Operator

## Mission

Turn a plain-language YouTube goal into a verified, owner-controlled channel operating system while reusing Hermes' existing YouTube research, Scroll Media, editing, Gauntlet, and publishing capabilities.

## Canonical workflow

`00 INTAKE → 01 RESEARCH → 02 POSITIONING → 03 CHANNEL SPEC → 04 ASSET GENERATION → 05 BROWSER SETUP → 06 CONTENT STARTER PACK → 07 VERIFY → 08 MONETIZATION READINESS → 09 HANDOFF`

This order is locked.

## 00 — Intake

Ask one bounded question at a time. Lead with the outcome. Keep visible choices to 3–5 where possible. Inspect existing repo/site/brand truth before asking the owner. Never ask the owner to repeat information Hermes can retrieve.

Required durable output:

`youtube/<channel-slug>/channel.yaml`

Minimum fields:

```yaml
channel:
  name: ""
  purpose: ""
  primary_audience: ""
  promise: ""
  on_camera_model: ""
  primary_cta: ""
brand:
  existing_assets: []
  source_urls: []
approval:
  public_publish: human_required
  monetization_terms: human_required
  adsense_payment_tax_identity: human_required
  ownership_permissions: human_required
status:
  intake_complete: false
```

Set `intake_complete: true` only when the required fields are resolved.

Suggested question order:

1. What is the main job of this YouTube channel? — make money / get customers / raise donations / grow audience / something else.
2. Who do we most need to reach first?
3. What should someone consistently expect from this channel?
4. Who or what will appear in content? — founder/team / customers/community / interviews / faceless or generated / mixed.
5. Do we already have a name, logo, colors, site, or brand kit? Inspect first.
6. What is the one primary viewer action? — book / buy / donate / subscribe / apply / visit site.

Stop asking when Hermes has enough information to proceed.

## 01 — Research

Route, do not duplicate:

- extraction/archive → `youtube-channel-scraper`
- transcript-backed market intelligence → `youtube-intelligence-pipeline`
- story/content system and YouTube packaging → `scroll-media-operator`

Research should answer the business question first: audience demand, repeated winning patterns, content gaps, proof sources, collaboration targets, and monetizable needs.

## 02 — Positioning

Produce one clear promise:

`This channel helps [audience] get/understand [outcome] through [credible mechanism].`

Choose 3–5 content pillars. Reject vague positioning that could describe hundreds of channels.

## 03 — Channel Spec

Prepare exact proposed values for:

- channel name;
- handle candidates;
- description;
- links;
- primary CTA;
- language/market;
- Home tab strategy;
- trailer/featured-video strategy;
- playlist architecture;
- upload defaults where appropriate;
- owner/human gates.

Do not write secrets into the spec.

## 04 — Asset Generation

Prepare:

- profile image;
- banner;
- video watermark;
- thumbnail grammar;
- optional lower-third or intro/outro assets when justified.

Thumbnail families are a grammar, not one repetitive template. Default families:

- Human Tension
- Proof
- Moment
- Explainer
- Documentary

Run taste-bearing assets through the configured design/Gauntlet process. Verify crop and legibility in the real YouTube surface before calling them complete.

## Mandatory story router

Before any Short, long-form video, Reel, story-driven post, or Interactive Drop, read `references/story-archetypes.md`.

Every content artifact MUST have exactly one `primary_story_type`.

One optional `secondary_story_type` may support the primary, but it may not redefine the story spine.

If classification is ambiguous, ask the minimum number of questions required. Never script with `story_type: unknown`.

All types use the locked engine:

`HOOK → TENSION → PAYOFF → ACTION`

## 05 — Browser Setup

Use the available authenticated Hermes browser/computer-control capability. Rediscover the live YouTube Studio interface at execution time. Do not depend on coordinate macros.

The browser worker may prepare or set owner-approved values for:

- name/handle candidate;
- profile image;
- banner;
- video watermark;
- description;
- links;
- Home tab layout/sections;
- trailer or featured-video slots when an approved artifact exists;
- playlists;
- non-consequential upload/default settings.

For each changed field record:

```yaml
change:
  field: ""
  before: ""
  proposed: ""
  after: ""
  evidence: "screenshot-or-platform-receipt"
  rollback: ""
```

### Hard browser stops

Explicit human approval is mandatory before:

- public publishing;
- accepting monetization, commerce, or other legal terms;
- creating/linking AdSense or entering payment/tax/identity information;
- ownership or channel-permission changes;
- destructive channel changes.

## 06 — Content Starter Pack

Default first pack:

- 20 researched ideas;
- ranked top 5;
- selected first 3;
- one primary story type per selected idea;
- Hook/Tension/Payoff/Action brief;
- title directions;
- thumbnail-family choice;
- CTA;
- source/rights notes.

Do not publicly publish during the production-readiness test.

## 07 — Verify

Verify at minimum:

- channel name/handle state;
- profile crop;
- banner crop/safe area across available preview surfaces;
- description;
- links;
- Home tab state;
- playlists/sections if configured;
- watermark if configured;
- no unauthorized permission changes;
- no secrets exposed;
- changed fields have rollback instructions.

Never call setup complete from intended state alone.

## 08 — Monetization Readiness

Fetch current official YouTube Help requirements every run before reporting eligibility. YouTube thresholds and product requirements can change.

Read the current Earn surface and available account evidence, but STOP before accepting agreements or entering AdSense/payment/tax/identity data.

Return exactly one status:

- `ELIGIBLE`
- `NOT_YET_ELIGIBLE`
- `ACTION_REQUIRED`

Include official source links and the next 1–3 actions.

## 09 — Handoff

Return a durable receipt containing:

- channel status;
- what is live;
- what is only prepared/draft;
- exact changed fields;
- asset locations;
- first three content briefs;
- monetization readiness;
- current metrics if available;
- next milestone;
- rollback instructions;
- unresolved approvals.

## Learning record

Every published experiment should record:

```yaml
learning:
  primary_story_type: ""
  hook_variant: ""
  title_pattern: ""
  thumbnail_grammar: ""
  format: ""
  retention_metrics: {}
  conversion_metrics: {}
  keep_or_discard: ""
```

This allows future AutoResearch-style optimization to compare like-with-like without mutating locked doctrine.

## Required report for major runs

Return:

DECISION
CHANGES
PROOF
STATUS
COMMERCIAL IMPACT
RISKS
ROLLBACK
NEXT
HUMAN APPROVAL
