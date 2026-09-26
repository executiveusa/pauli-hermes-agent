---
name: youtube-personal-intelligence
description: ICM-governed YouTube intelligence and faceless-channel expertise for Bambu/Hermes. Use for personal YouTube history analysis, subscriptions, playlists, liked/saved videos, transcript extraction, creator/topic categorization, graph construction, channel research, faceless-channel strategy, hooks, scripts, SEO, Shorts, monetization, repurposing, and recurring intelligence reports.
version: 0.1.0
author: Bambu / Pauli Effect
license: MIT
tags: [youtube, transcripts, history, subscriptions, playlists, knowledge-graph, faceless-youtube, strategy, scripts, seo, monetization, icm]
triggers:
  - analyze my YouTube
  - scan my YouTube history
  - organize my subscriptions
  - analyze my playlists
  - analyze my liked videos
  - graph my YouTube interests
  - research this YouTube channel
  - summarize these YouTube videos
  - find faceless YouTube opportunities
  - create a faceless YouTube strategy
  - write a YouTube script
entry_point: /youtube-personal-intelligence
---

# YouTube Personal Intelligence

## Mission

Turn Bambu's YouTube activity into a searchable, explainable personal research graph, then use that evidence to operate as a faceless YouTube expert.

Hermes has three responsibilities:

1. **Observe** — ingest authorized YouTube account data, exports, transcripts, channel/video metadata, and saved material.
2. **Understand** — classify creators/topics, summarize transcripts, identify recurring concepts, connect related videos/channels, and track interests over time.
3. **Create** — use the resulting evidence to design faceless-channel theses, content pillars, hooks, scripts, SEO, Shorts, monetization paths, and experiments.

Do not collapse these into a generic YouTube bot. Personal intelligence is evidence; publishing is a separate governed action.

## Upstream capability map

### ZeroPointRepo/youtube-skills

Primary use: public YouTube research and transcript acquisition.

Reuse concepts/capabilities for:

- video transcripts;
- YouTube search;
- channel search and browsing;
- all/latest channel videos;
- within-channel search;
- public playlist enumeration;
- captions/subtitles and metadata.

Source: `https://github.com/ZeroPointRepo/youtube-skills`

The upstream `youtube-full` skill is explicitly not an authenticated account-management/history API. Never claim it can see Bambu's private watch history by itself.

### AgriciDaniel/claude-youtube

Primary use: creator-side YouTube expertise.

Reuse concepts/capabilities for:

- channel/video analysis;
- audits;
- content strategy;
- competitor research;
- ideation;
- hooks;
- scripting and retention structure;
- metadata and SEO;
- Shorts optimization;
- calendars;
- monetization;
- repurposing.

Source: `https://github.com/AgriciDaniel/claude-youtube`

Do not duplicate every upstream file. Hermes should lazy-load only the relevant creator discipline for the current task.

## Source-of-truth hierarchy

Use the least invasive reliable source in this order:

1. **YouTube Data API + OAuth 2.0** for supported private account data.
2. **User-authorized Google Takeout / My Activity exports** for historical backfill and data the API does not expose.
3. **ZeroPointRepo YouTube skill / TranscriptAPI** for public transcripts and public research.
4. **Hermes HyperAgent browser operator** only for read-only gaps in an already authenticated browser session.

Never scrape credentials or persist browser cookies/session tokens in the intelligence graph.

## Hard API boundary

Treat the following as supported through authenticated YouTube APIs where scopes and account access allow it:

- subscriptions;
- user-created playlists;
- accessible playlist items;
- liked/rated videos;
- channel metadata;
- public/private video metadata permitted to the user.

Treat **watch history** and **Watch Later** as import/browser-source data. The official YouTube Data API does not provide a reliable endpoint for retrieving those personal collections.

If a source cannot prove a field, store `unknown` instead of inventing it.

## ICM contract

Before any new ingestion or account connection record:

- MODE: brownfield.
- OUTCOME: exact intelligence/result requested.
- TARGET: exact Google/YouTube account or imported dataset.
- CONSTRAINTS: read-only unless explicitly authorized otherwise.
- PROOF: counts, source receipts, checksums, sample records, and coverage dates.
- COMMERCIAL VALUE: learning, production leverage, or revenue hypothesis enabled.
- ROLLBACK: revoke OAuth, remove imported dataset version, or disable scheduled sync without deleting originals.

## Authority model

### Automatic read/analyze

When access is already authorized, Hermes may:

- list subscriptions, playlists, liked videos, and accessible playlist items;
- import YouTube/Google history exports supplied by Bambu;
- retrieve public transcripts and metadata;
- classify channels and videos;
- generate embeddings, summaries, clusters, timelines, and graph edges;
- identify repeated creators, topics, tools, claims, products, business models, and formats;
- compare themes over time;
- generate read-only reports and faceless-channel hypotheses.

### Explicit human approval required

Hermes must obtain approval before:

- connecting a new Google account or granting new OAuth scopes;
- uploading or publishing a video/Short;
- commenting, liking/disliking, subscribing/unsubscribing;
- creating, deleting, or renaming playlists;
- changing channel/account settings;
- purchasing ads, footage, music, tools, sponsorships, or services;
- deleting the canonical personal intelligence dataset.

## Acquisition modes

### A. OAuth incremental sync

Minimum-first scope policy: request read-only scopes unless a specifically authorized write needs more.

Logical collections:

- `youtube_accounts`
- `youtube_subscriptions`
- `youtube_playlists`
- `youtube_playlist_items`
- `youtube_liked_videos`
- `youtube_video_metadata`
- `youtube_channels`

Tokens belong only in the approved secret store. Never put OAuth tokens, API keys, client secrets, cookies, or session artifacts in Git, logs, prompts, graph nodes, or receipts.

### B. Historical backfill

Ingest user-authorized Google Takeout/My Activity YouTube exports.

Normalize records into:

```json
{
  "event_id": "stable-hash",
  "event_type": "watch|search|like|playlist_add|subscription|unknown",
  "occurred_at": "ISO-8601|null",
  "video_id": "string|null",
  "channel_id": "string|null",
  "title": "string|null",
  "query": "string|null",
  "source": "youtube_api|google_takeout|browser|transcriptapi",
  "source_record_id": "string|null",
  "confidence": 0.0
}
```

Imports must be idempotent. Re-importing the same export must not create duplicates.

### C. Public transcript enrichment

For each important/recent/saved video, enrich only as needed:

1. video/channel metadata;
2. transcript when available;
3. language;
4. timestamped chunks;
5. summary;
6. claims/key ideas;
7. people/products/tools/entities;
8. topics;
9. actionable lessons;
10. relationships to other videos/topics.

Do not fetch every transcript indiscriminately. Prioritize saved/liked/recent/high-frequency creators and requested topics to control cost.

### D. Browser gap-filling

Use `hyperagent-browser-operator` only when an authenticated session is already available and API/export routes cannot answer the read-only question.

Examples:

- inspect Watch Later pages;
- inspect history UI for a bounded date range;
- verify private playlist contents visible to the user.

Browser output is provisional until normalized and independently checked where possible.

## Personal YouTube knowledge graph

### Node types

- `Account`
- `Channel`
- `Creator`
- `Video`
- `Playlist`
- `Topic`
- `Concept`
- `Tool`
- `Product`
- `Company`
- `Person`
- `Claim`
- `Format`
- `BusinessModel`
- `FacelessOpportunity`

### Core edges

- `ACCOUNT_SUBSCRIBES_TO_CHANNEL`
- `ACCOUNT_WATCHED_VIDEO`
- `ACCOUNT_LIKED_VIDEO`
- `ACCOUNT_SAVED_VIDEO`
- `PLAYLIST_CONTAINS_VIDEO`
- `CHANNEL_PUBLISHED_VIDEO`
- `VIDEO_MENTIONS_TOPIC`
- `VIDEO_MENTIONS_TOOL`
- `VIDEO_MAKES_CLAIM`
- `VIDEO_SIMILAR_TO_VIDEO`
- `CHANNEL_SPECIALIZES_IN_TOPIC`
- `TOPIC_RELATED_TO_TOPIC`
- `OPPORTUNITY_DERIVED_FROM_TOPIC`
- `OPPORTUNITY_INSPIRED_BY_FORMAT`

Every derived edge stores source IDs, confidence, and derivation version.

## Categorization model

Each subscribed channel may have multiple weighted categories, not one rigid bucket.

Minimum useful taxonomy:

- AI / agents / automation
- software / coding / infrastructure
- SaaS / product / startups
- marketing / sales / growth
- design / creative / media
- finance / investing / crypto
- business models / entrepreneurship
- social impact / nonprofit
- health / wellness
- psychology / behavior
- spirituality / philosophy
- politics / geopolitics
- China / international technology
- science / engineering
- construction / fabrication / DIY
- travel / local culture
- entertainment / culture
- YouTube / creator economy
- other

Track category confidence and allow taxonomy evolution instead of forcing bad classifications.

## Required personal overview

When Bambu asks for "my YouTube overview," return:

1. total subscriptions by category;
2. top creators by watch/save/like frequency;
3. top topics for 7d / 30d / 90d / all-time where coverage exists;
4. saved/playlist backlog by topic;
5. unresolved/unwatched high-value saved videos;
6. major interest shifts over time;
7. repeated tools/products/companies mentioned;
8. recurring business models and monetization ideas;
9. conflicting claims worth investigating;
10. faceless-channel opportunities supported by evidence;
11. coverage gaps and source limitations.

Never present partial imported history as all-time history without explicit coverage dates.

## Faceless YouTube expert workflow

### 1. Discover

Use personal graph + public YouTube research to identify niches where:

- Bambu has sustained interest/knowledge;
- audiences show repeat demand;
- content can be produced without relying on a visible host;
- source material can be cited/verified;
- monetization has a plausible path;
- production can become repeatable and agent-assisted.

### 2. Validate before building

Score each channel thesis on:

- demand evidence;
- competition density;
- differentiation;
- source availability;
- production complexity;
- monetization options;
- policy/copyright risk;
- repeatability;
- 30-day validation path.

Do not propose a content factory before a channel thesis passes validation.

### 3. Creator disciplines

Lazy-load the relevant AgriciDaniel-style discipline:

- `analyze` for video/channel pattern analysis;
- `audit` for a channel health audit;
- `competitor` for comparable channels;
- `ideate` for evidence-backed topics;
- `hook` for first-30-second concepts;
- `script` for retention-oriented scripts;
- `metadata` / `seo` for titles/descriptions/search packaging;
- `shorts` for short-form adaptations;
- `calendar` for publishing cadence;
- `monetize` for revenue paths;
- `repurpose` for derivatives.

### 4. Content provenance

Every script/brief should keep a source manifest:

- source video IDs/URLs;
- transcript segments used;
- factual claims requiring verification;
- licensed/original asset plan;
- generated assets disclosed internally;
- copyright/reuse risks.

Never treat another creator's transcript as a script to rewrite closely. Extract facts/ideas, synthesize independently, and preserve originality.

### 5. Publishing gate

Publishing remains human-approved until separately promoted after production proof.

Required pre-publish receipt:

- audience + promise;
- source manifest;
- factual verification status;
- title/thumbnail hypothesis;
- hook;
- retention structure;
- monetization intent;
- copyright/policy check;
- rollback/unpublish path;
- explicit approval.

## Example commands

- `Hermes, map everything I watched about AI agents in the last 90 days.`
- `Hermes, categorize every channel I'm subscribed to and show overlaps.`
- `Hermes, tell me which saved videos are probably highest leverage for my active work.`
- `Hermes, find recurring SaaS business models across my YouTube history.`
- `Hermes, show how my interests changed over the last year.`
- `Hermes, find three faceless channel opportunities supported by my actual viewing graph.`
- `Hermes, research this niche, compare the top 20 channels, then build a 10-video validation slate.`
- `Hermes, create a source-backed script but do not publish anything.`

## Completion receipt

Return:

```text
DECISION
MODE
OUTCOME
TARGET
SOURCES USED
COVERAGE
CHANGES
PROOF
GRAPH/INDEX DELTA
FACeless OPPORTUNITY IMPACT
RISKS
ROLLBACK
STATUS
HUMAN APPROVAL
```

Never claim full personal-history coverage unless the evidence actually spans the claimed period.
