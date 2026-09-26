---
name: faceless-youtube-channel-operator
description: Unified ICM-governed workflow for researching, designing, producing, publishing, measuring, and improving a faceless YouTube channel. This is the single user-facing YouTube operator for Hermes; upstream YouTube repos are internal engines, not separate user workflows.
version: 0.1.0
author: Bambu / Pauli Effect
license: MIT
user-invocable: true
tags: [youtube, faceless, research, transcripts, seo, production, publishing, analytics, hyperagent, scrapling, icm]
triggers:
  - run the faceless youtube channel
  - faceless youtube
  - study this youtube creator
  - scrape this youtube channel
  - analyze this youtube course
  - make the youtube plan
  - build the youtube workflow
  - create the next youtube video
entry_point: /faceless-youtube
---

# Faceless YouTube Channel Operator

## Purpose

Hermes operates one end-to-end system, not a bag of YouTube skills.

Internal engines are lazy-loaded behind this workflow:

- `youtube-personal-intelligence` — Bambu/account history, subscriptions, playlists, saved videos, transcript graph.
- ZeroPointRepo/youtube-skills patterns — public channel/video/playlist/transcript acquisition.
- AgriciDaniel/claude-youtube patterns — channel strategy, hooks, scripts, retention, Shorts, monetization, repurposing.
- deeployCO/youtube-seo-skills patterns — surface-specific SEO/packaging, channel/video audit, keyword, title, thumbnail and competitor analysis.
- wnstify/tubeflow patterns — parallel research, evidence preservation, strategist synthesis, production-state workflow and repurposing.
- wvn774/best-youtube-growth-tools — tool landscape only, never an execution dependency.
- D4Vinci/Scrapling — optional adaptive public-web extraction runtime.
- Hermes HyperAgent browser operator — authenticated/dynamic browser acquisition and deterministic replay.
- Firecrawl — optional temporary public-web extraction fallback when explicitly authorized.

Do not expose these as separate decisions to the owner. Route internally.

## ICM lock

MODE: brownfield Hermes capability expansion.

OUTCOME: operate a repeatable, evidence-driven faceless YouTube channel from creator-model research through measured iteration.

TARGET: the owner-approved channel plus benchmark creators/courses.

CONSTRAINTS:

- read-only research is automatic when already authorized;
- no credentials in Git, prompts, receipts, graph nodes, browser caches, or datasets;
- no publishing, commenting, subscribing, paid tools, ads, or account mutations without explicit owner approval;
- do not imitate scripts, thumbnails, voices, branding, or copyrighted expression closely;
- extract process, structure, topic patterns, packaging logic, and business model — synthesize original work;
- preserve source provenance and coverage dates;
- use first-party/current evidence before heuristics;
- no claim of production success without verification.

PROOF: source manifest, scrape counts, transcript coverage, research pack, production artifacts, publish receipt, and post-publish metrics.

ROLLBACK: disable connector/browser routine, revoke OAuth, discard generated drafts, unpublish only with explicit approval, preserve prior dataset versions.

## Operating loop

```text
BENCHMARK CREATOR / COURSE
        ↓
ACQUIRE EVIDENCE
        ↓
NORMALIZE + TRANSCRIBE
        ↓
PROCESS GRAPH
        ↓
EXTRACT CREATOR PLAYBOOK
        ↓
VALIDATE CHANNEL THESIS
        ↓
RESEARCH VIDEO OPPORTUNITY
        ↓
PACKAGE BEFORE SCRIPT
        ↓
SCRIPT + SHOT/ASSET PLAN
        ↓
INDEPENDENT QA
        ↓
HUMAN APPROVAL
        ↓
PRODUCE
        ↓
PRE-PUBLISH QA
        ↓
HUMAN PUBLISH APPROVAL
        ↓
PUBLISH + VERIFY
        ↓
DISTRIBUTE / REPURPOSE
        ↓
MEASURE
        ↓
LEARN BACK INTO GRAPH
        ↺
```

# Workflow A — Benchmark Creator Reverse Engineering

Use when Bambu names a creator/channel/course whose process should become the reference model.

### A1. Acquire

For public YouTube channels collect, at minimum:

- channel metadata and channel ID;
- newest N long-form videos;
- newest N Shorts;
- URL/video ID;
- title;
- publish date/time where available;
- duration;
- views/likes/comments where available;
- description;
- thumbnail URL/image;
- transcript/captions where available;
- chapters;
- links/CTAs/offers in description;
- sponsor/affiliate/product references;
- recurring series/format indicators.

For private course/community content, use an already-authenticated browser session only. Never bypass authentication or access controls.

### A2. Scraper routing

Choose the cheapest reliable path:

1. YouTube/API/transcript deterministic source when sufficient.
2. HyperAgent read-only browser routine for dynamic/authenticated pages.
3. Scrapling for public pages that need adaptive selectors/stealth extraction.
4. Firecrawl only when explicitly authorized and other native routes are unavailable.

Temporary API keys are secrets even when the owner says they will be deleted. Use them only in secret/runtime environment variables; never commit, echo, store, or place them into action caches.

### A3. Normalize

Write canonical records:

```json
{
  "creator_id": "string",
  "channel_id": "string|null",
  "video_id": "string",
  "format": "long|short|live|unknown",
  "title": "string",
  "url": "string",
  "published_at": "ISO-8601|null",
  "duration_seconds": 0,
  "views": null,
  "likes": null,
  "comments": null,
  "description": "string|null",
  "thumbnail_url": "string|null",
  "transcript_status": "complete|partial|missing",
  "source": "youtube_api|transcriptapi|browser|scrapling|firecrawl|other",
  "retrieved_at": "ISO-8601",
  "confidence": 1.0
}
```

Never silently mix Shorts and long-form in the same performance analysis.

### A4. Transcript extraction

Prioritize transcripts for the benchmark sample. For every transcript derive:

- hook/opening pattern;
- promise;
- target viewer;
- pain/problem;
- curiosity/open loops;
- story/example structure;
- teaching blocks;
- proof/authority devices;
- CTA(s);
- offer/product bridge;
- pacing estimate;
- recurring phrases only as analytical features, not copy material;
- visual/B-roll cues inferred from available evidence;
- factual sources/claims needing verification.

### A5. Playbook synthesis

Produce a creator playbook that answers:

1. What topics are repeatedly chosen?
2. What audience job-to-be-done is served?
3. Which title structures recur?
4. Which thumbnail structures recur?
5. What is the long-form cadence?
6. What is the Shorts cadence?
7. How are Shorts related to long-form?
8. What are the hook archetypes?
9. What retention structures recur?
10. What offers/CTAs appear and where?
11. What monetization model is visible?
12. What publishing/series patterns exist?
13. What content appears to outperform baseline?
14. What can be generalized without copying expression?
15. What should Hermes deliberately NOT imitate?

# Workflow B — Private Course Process Extraction

Use for owner-authorized private training such as Skool.

### B1. Browser contract

Before scan prove:

- browser session is authenticated by owner;
- group/course target is correct;
- read-only authority;
- no payment/enrollment change is required;
- no downloads outside normal member permissions;
- no credential/session material will persist in routine cache.

### B2. Course inventory

Extract:

- classroom/module names;
- lesson titles/order;
- lesson URLs/IDs;
- lesson text/resources visible to member;
- video URLs/embeds visible to member;
- templates/checklists/resources;
- process steps;
- explicit metrics/benchmarks;
- monetization/offer methods;
- channel creation workflow;
- research workflow;
- production workflow;
- thumbnail/title workflow;
- upload/publish workflow;
- analytics/improvement workflow.

Do not automatically download copyrighted course videos. Prefer lesson metadata, notes, owner-visible transcripts/captions, and process summaries unless the platform explicitly provides a permitted download.

### B3. Convert course into executable process

For each lesson classify:

`principle | decision | input | action | artifact | metric | gate | exception`

Then link lessons into a state machine rather than preserving them as isolated notes.

# Workflow C — Channel Thesis and Validation

Before generating a content factory, define:

- niche/audience;
- transformation/promise;
- why faceless is appropriate;
- differentiation from benchmark creator;
- content pillars;
- long-form vs Shorts roles;
- monetization path;
- evidence of demand;
- production complexity;
- copyright/policy risk;
- 30-day validation test;
- kill criteria.

No scale automation before the thesis passes.

# Workflow D — Per-Video Research Pack

Run four evidence lanes in parallel when useful:

1. Topic / first-party facts / primary sources.
2. Competitor videos / format gaps / audience expectations.
3. Search + Browse + Suggested + Shorts packaging opportunities.
4. Community/customer questions and language.

Then run one strategist synthesis over condensed evidence.

Artifacts:

- `research-pack.md`
- `source-manifest.json`
- `competitor-grid.json`
- `audience-questions.json`
- `series-structure.md` when warranted.

Raw evidence is preserved separately from condensed strategy.

# Workflow E — Package Before Script

Do not write the full script until the package hypothesis exists.

Required package candidates:

- audience + desired outcome;
- 3-10 title candidates categorized by discovery surface;
- thumbnail concept(s);
- one-sentence promise;
- first 30-second hook concept;
- target video length/format;
- primary CTA;
- next-video/session bridge;
- success metric.

For established channels use YouTube Studio first-party data when available. Do not guess CTR, retention, traffic-source, or audience metrics.

# Workflow F — Script + Faceless Production Plan

Script artifact includes:

- hook;
- open loops;
- scene-by-scene narration;
- on-screen text;
- visual/B-roll/source plan;
- graphics/data/diagram calls;
- citation/provenance markers for factual claims;
- transitions;
- pattern interrupts;
- CTA;
- end-screen/next-video bridge;
- voiceover pronunciation notes where useful;
- asset-license status.

Never closely paraphrase a competitor transcript.

# Workflow G — Independent Review Gate

Creator cannot approve itself.

Run independent reviews for:

- factual/source integrity;
- audience value;
- hook/retention structure;
- title/thumbnail promise alignment;
- originality/copyright;
- YouTube policy/synthetic-content disclosure risk;
- monetization alignment;
- production feasibility;
- brand/taste.

Target release score: 8.5/10 or explicit owner override.

# Workflow H — Production and Publishing

Production may include approved voice, generated/original visuals, licensed stock, motion graphics, captions, music, edit, and thumbnail generation.

Publishing remains a separate explicit approval gate.

Pre-publish receipt:

```text
TARGET CHANNEL
VIDEO
SOURCE MANIFEST
FACT CHECK
ASSET LICENSE/ORIGIN
TITLE
THUMBNAIL
DESCRIPTION
DISCLOSURES
CTA
MONETIZATION
ROLLBACK/UNPUBLISH PATH
APPROVAL
```

After publish independently verify the live URL and expected metadata.

# Workflow I — Repurpose and Distribution

After verified publication, derive channel-consistent assets rather than generic reposts:

- Shorts cutdowns/derivatives;
- LinkedIn;
- X/Twitter;
- Facebook;
- newsletter/email;
- community post;
- blog/article where commercially useful.

Preserve the core promise while adapting format and hook to each surface.

# Workflow J — Analytics Flywheel

At predefined review windows collect first-party metrics available from YouTube Studio/API:

- impressions;
- CTR by traffic source/surface where available;
- views;
- watch time;
- average view duration;
- average percentage viewed;
- first 30-second retention;
- retention cliffs/spikes;
- traffic source mix;
- returning/new viewers;
- subscribers gained/lost from video;
- end-screen/card performance;
- comments/likes/shares;
- revenue/RPM when authorized;
- Shorts viewed-vs-swiped-away and retention when available.

Compare against the channel's own baseline before generic benchmarks.

Feed lessons back into:

`topic → package → hook → script structure → production → CTA → monetization`

# Current benchmark mission: One Person Business / Dave Nick

Benchmark targets:

- Public channel: `https://www.youtube.com/@One-Person-Business`
- Long-form target: newest 25 from `/videos`
- Shorts target: newest 30 from `/shorts`
- Private training target: `https://www.skool.com/obf`

Known public identity should be verified at acquisition time rather than hard-coded forever.

For this mission generate:

```text
benchmarks/one-person-business/
├── manifest.json
├── long-form.json          # exactly 25 if available
├── shorts.json             # exactly 30 if available
├── transcripts/
├── thumbnails/
├── descriptions/
├── course/
│   ├── inventory.json
│   ├── process-map.json
│   └── course-playbook.md
├── creator-playbook.md
├── title-patterns.json
├── thumbnail-patterns.json
├── hook-patterns.json
├── offer-cta-map.json
├── cadence.json
└── workflow-diff-vs-hermes.md
```

Coverage receipt must state exact counts collected and any missing items.

# Completion format

```text
DECISION
MODE
OUTCOME
TARGET
ACQUISITION SOURCES
COVERAGE
WORKFLOW STATE
CHANGES
PROOF
CREATOR PLAYBOOK DELTA
CHANNEL OPPORTUNITY
RISKS
ROLLBACK
STATUS
HUMAN APPROVAL
NEXT
```

Never call the system complete because a scrape, script, render, upload request, or CI job succeeded. Prove the requested state.