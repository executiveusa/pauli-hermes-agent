# TubeFlow Production Orchestration Reference

Source: https://github.com/wnstify/tubeflow
License: MIT
Role in Hermes: production orchestration pattern for YouTube research, creation, publishing preparation, and social repurposing.

## Why this source matters

TubeFlow contributes a research-first, multi-agent production workflow rather than another generic YouTube knowledge prompt. Hermes should reuse the workflow mechanics, not blindly copy Claude-specific commands or model names.

## Net-new capabilities to adopt

### 1. Parallel research gatherers

For a serious video or series thesis, Hermes may fan out bounded research into four independent lanes:

- topic / subject-matter evidence;
- competitor / existing YouTube landscape;
- SEO / discoverability / search intent;
- community / questions / pain points / language.

Each lane should preserve raw evidence and produce a bounded summary. The synthesis agent consumes summaries first and may inspect raw evidence only when needed.

### 2. Sequential strategist after parallel research

Do not synthesize before the research lanes complete.

The strategist should determine:

- target audience;
- audience knowledge level;
- unresolved questions;
- competitive gaps;
- recommended angle;
- single-video vs series decision;
- production difficulty;
- source requirements;
- monetization fit;
- next experiment.

### 3. Research-pack artifact

A production-ready research pack should contain:

- executive recommendation;
- topic overview;
- competitor landscape;
- audience + pain points;
- SEO/search packaging hypothesis;
- community language/questions;
- differentiation thesis;
- single-video or series recommendation;
- production notes;
- source manifest;
- risks;
- next action.

### 4. Preserve raw evidence separately from synthesis

Recommended structure:

```text
youtube/
  research/<slug>/
    research-pack.md
    series-structure.md        # only when justified
    summaries/
      topic.md
      competitors.md
      seo.md
      community.md
    raw/
      topic/
      competitors/
      seo/
      community/
```

This allows Hermes to keep the strategist context compact while retaining inspectable evidence.

### 5. Creator and publisher separation

TubeFlow separates content creation from publication. Preserve this boundary in Hermes.

- Creator agent: research, angle, outline, script, visual cues, description, thumbnail hypothesis, pinned comment, source manifest.
- Publisher agent: validates assets, metadata, approvals, target channel, and final upload state.
- Publisher must never approve its own work.

Publishing stays human-approved unless a separately governed policy explicitly promotes it.

### 6. Video production package

For a full video package, produce:

- structured outline;
- hook;
- narration/script;
- scene/B-roll/visual cues;
- CTA/outro;
- title hypotheses;
- description;
- thumbnail text/brief;
- pinned comment;
- chapters when appropriate;
- source manifest;
- social repurposing payload.

Do not copy TubeFlow's fixed brand voice, self-hosting references, or hard-coded links. Hermes must load the active channel's own voice, offer, CTA, and brand constraints.

### 7. Tool/application research discipline

When a video discusses named software, services, frameworks, or products, prefer primary sources and collect:

- official site;
- official docs;
- official repository when applicable;
- license/open-source status when relevant;
- maintainer/developer identity;
- commercial offering where relevant;
- material limitations or policy constraints.

For technical claims, primary sources beat creator commentary.

### 8. Social derivative contract

After a video package reaches approved/published state, create a structured derivative payload that can feed separate platform specialists.

Suggested schema:

```json
{
  "video_id": null,
  "slug": "",
  "title": "",
  "hook": "",
  "summary": "",
  "key_points": [],
  "claims": [],
  "source_manifest": [],
  "cta": "",
  "video_url": null,
  "platform_targets": ["linkedin", "x", "facebook", "instagram", "shorts"]
}
```

Platform adaptation should be handled by platform-specific skills rather than forcing one universal post format.

### 9. Parallelism policy

Parallelize only independent work.

Good parallel targets:

- topic research;
- competitor research;
- SEO research;
- community research;
- multiple independent idea explorations;
- platform-specific derivative drafts.

Sequential dependencies:

- research -> synthesis;
- synthesis -> script;
- script/asset verification -> publish preparation;
- explicit approval -> publish;
- verified publish -> distribution/repurposing.

### 10. Context-budget discipline

Do not dump all raw research into the strategist context. Each gatherer should return a bounded summary with links/source IDs to full evidence.

Hermes should optimize for:

- traceability;
- low context waste;
- source preservation;
- deterministic handoffs;
- ability to reopen evidence when challenged.

## Hermes ICM production state machine

```text
IDEA
  -> RESEARCHING
  -> RESEARCHED
  -> THESIS_APPROVED
  -> SCRIPTING
  -> PACKAGE_READY
  -> REVIEW_REQUIRED
  -> APPROVED_FOR_PRODUCTION
  -> ASSETS_READY
  -> APPROVED_FOR_PUBLISH
  -> PUBLISHED
  -> VERIFIED
  -> REPURPOSED
  -> MEASURED
  -> ITERATED
```

No state may be skipped when the missing gate contains unresolved factual, ownership, copyright, account, or publication risk.

## Production proof

Before declaring a package ready:

- research lanes completed or intentionally waived with reason;
- source manifest exists;
- thesis is explicit;
- script is original synthesis, not transcript paraphrase;
- factual claims have verification status;
- visual/B-roll plan exists;
- title + thumbnail hypothesis exists;
- CTA aligns with commercial intent;
- copyright/policy risks are noted;
- downstream owner/action is clear.

Before declaring published:

- correct target channel proven;
- human approval receipt exists;
- upload succeeded;
- public/unlisted/private state verified against instruction;
- final URL/video ID recorded;
- metadata checked after upload;
- rollback/unpublish path recorded.

## What not to inherit blindly

Do not inherit:

- Claude-specific Task syntax;
- hard-coded Haiku/Opus model routing;
- fixed token counts as universal constants;
- fixed platform engagement statistics without fresh verification;
- hard-coded brand voice;
- hard-coded social links;
- automatic publishing authority.

Hermes should preserve the architecture while choosing the cheapest capable model/agent at runtime.

## Commercial use

Use this production orchestration only after a channel thesis has a plausible 30-day validation path. The system should prioritize a small number of measurable content experiments over building a large content factory before demand is proven.
