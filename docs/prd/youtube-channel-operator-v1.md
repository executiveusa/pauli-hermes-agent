# PRD — YouTube Channel Operator v1

Status: implementation-ready / production-test candidate  
Owner runtime: Hermes  
Primary skill: `skills/youtube-channel-operator/`  
Date: 2026-08-23

## 1. Outcome

Give Hermes one governed workflow that can take a plain-language goal such as “set up and grow this YouTube channel” and turn it into:

1. a minimal ADHD-friendly intake;
2. market/channel research;
3. channel positioning and channel spec;
4. required brand assets and thumbnail grammar;
5. authenticated browser-assisted YouTube Studio setup;
6. a first content pack routed through a canonical story type;
7. verification receipts;
8. monetization-readiness status based on current official YouTube requirements;
9. a durable handoff and learning record.

The owner should not have to understand YouTube Studio internals or Hermes implementation details.

## 2. Canonical workflow

`00 INTAKE → 01 RESEARCH → 02 POSITIONING → 03 CHANNEL SPEC → 04 ASSET GENERATION → 05 BROWSER SETUP → 06 CONTENT STARTER PACK → 07 VERIFY → 08 MONETIZATION READINESS → 09 HANDOFF`

This order is locked. Browser setup and content generation do not begin before intake is complete.

## 3. Product laws

- Outcome before content.
- One bounded question at a time.
- Inspect existing repo/site/brand truth before asking the owner.
- Every story artifact has exactly one primary story type.
- All story types use `HOOK → TENSION → PAYOFF → ACTION`.
- Human approval is mandatory before public publishing, legal/monetization terms, AdSense/payment/tax/identity actions, ownership/permission changes, or destructive channel changes.
- Browser automation must produce inspectable before/after receipts for consequential setup changes.
- Never store credentials or secrets in repo artifacts.
- Never claim a channel is live, monetized, eligible, or verified without direct platform evidence.
- Monetization thresholds are fetched from current official YouTube Help at execution time; do not treat hard-coded thresholds as permanent truth.

## 4. Canonical story router

Exactly one primary type per content run:

1. Overcame It — adversity → struggle → earned outcome
2. Nobody Saw This Coming — surprising cause/effect
3. The Person Behind It — organization/project → human story
4. Before It Was Fixed — problem → intervention → proof
5. One Decision Changed Everything — one choice as the hinge
6. Why This Matters — larger issue through one person/event
7. Receipts — claim → evidence → result
8. Challenge — call out → tension → action
9. Hidden Opportunity — overlooked value → why it matters → action
10. Interactive Choice — audience choice changes the next reveal/branch

`Interactive Choice` is experiment-shelf only. Interactive Sora or other branching-video systems are optional experiment providers, never a core production dependency.

## 5. Intake contract

Required durable output: `youtube/<channel-slug>/channel.yaml`.

Minimum resolved fields:

- channel purpose
- primary audience
- channel promise
- on-camera/content model
- existing brand sources
- primary CTA
- human approval boundaries

The intake should usually resolve in 4–6 questions. Hermes may propose options from existing context rather than asking open-ended questions.

## 6. Research contract

Use existing skills rather than duplicating them:

- `youtube-channel-scraper` for extraction/archive jobs;
- `youtube-intelligence-pipeline` for transcript-backed, cross-channel research;
- `scroll-media-operator` for content strategy, Shorts/Reels, publishing/analytics and the locked story engine;
- `gauntlet-loop` for taste-bearing comparisons;
- `cinematic-master-editor` where video editing is required;
- `social-drop-factory` when the YouTube CTA should lead to an Interactive Drop.

The operator is an orchestrator, not a replacement for those skills.

## 7. Browser setup contract

The browser worker may prepare and fill approved values for:

- channel name and handle candidate;
- profile picture;
- banner;
- video watermark;
- description;
- site links;
- Home tab layout, trailer/featured content, sections;
- playlists;
- upload defaults and non-consequential channel configuration.

Official YouTube Studio currently exposes channel customization through Home and Profile surfaces, including profile picture, banner, watermark, channel name, handle, description and links. The implementation must rediscover the current UI at execution time rather than depend on brittle coordinates.

Hard stop before:

- accepting monetization/commerce/legal terms;
- creating/linking AdSense or entering payment/tax/identity data;
- changing ownership or delegated permissions;
- publishing public content;
- destructive channel changes.

## 8. Asset contract

Generate or prepare:

- profile image;
- channel banner with safe-area verification;
- video watermark;
- thumbnail grammar, not one repetitive template;
- optional lower-thirds / intro-outro assets if the format needs them.

Thumbnail families may include Human Tension, Proof, Moment, Explainer, and Documentary. Selection is story-dependent.

All generated assets require crop/legibility verification in the real YouTube surface before being called complete.

## 9. Monetization readiness

At runtime, query current official YouTube Help before reporting eligibility.

As of 2026-08-23, official YouTube Help states full ad/Premium YPP entry at 1,000 subscribers plus either 4,000 qualified public watch hours in the prior 12 months or 10 million qualified public Shorts views in the prior 90 days. Expanded YPP fan-funding/select Shopping access may begin at 500 subscribers, 3 public uploads in 90 days, plus either 3,000 qualified watch hours in 12 months or 3 million qualified Shorts views in 90 days, where available. YouTube has also announced higher ad/Premium watch/view thresholds for new creators beginning 2027-02-01, so the agent MUST fetch current requirements every run.

The operator outputs only:

- `ELIGIBLE`
- `NOT_YET_ELIGIBLE`
- `ACTION_REQUIRED`

with source URLs, observed account/channel evidence, and the next 1–3 actions.

## 10. Learning contract

Every published experiment records at minimum:

- primary story type;
- hook variant;
- title pattern;
- thumbnail grammar;
- length/format;
- retention metrics;
- conversion metrics;
- keep/discard decision.

This schema is intentionally compatible with a future AutoResearch-style loop. Optimization must compare like-with-like (for example, Receipts against Receipts) before changing locked core doctrine.

## 11. Production-test slice

The first production test is deliberately bounded.

### Test input

One owner-controlled or test YouTube channel with an authenticated browser session and no need to accept monetization/payment/ownership terms.

### Test actions

1. Run intake and write a complete `channel.yaml`.
2. Classify one content idea into a primary story type.
3. Produce channel spec and one complete asset pack.
4. Browser worker opens YouTube Studio and prepares/updates non-consequential customization settings.
5. Verify visible profile/banner/description/link/home-section state.
6. Generate three content briefs, not public posts.
7. Read the Earn surface and current official YPP documentation; report readiness without accepting terms.
8. Emit a handoff receipt.

### Pass criteria

- no unresolved intake fields;
- exactly one primary story type per content run;
- no browser action crosses a human gate;
- all changed settings have evidence receipts;
- asset dimensions/crops are visually acceptable in Studio;
- monetization report cites current official requirements;
- no secrets in repo/run artifacts;
- public publishing count = 0 for the test;
- rollback instructions exist for every changed channel field;
- skill contract validator passes.

## 12. Out of scope for v1

- unattended public publishing;
- automatic acceptance of YPP/Commerce/AdSense terms;
- tax/payment/identity automation;
- automatic ownership changes;
- self-modifying core doctrine;
- Interactive Sora as a required dependency;
- promises of revenue or virality.

## 13. Rollback

Channel changes must be reversible field-by-field using the pre-change receipt. Repo implementation rollback is a normal revert of the YouTube Channel Operator merge commit.
