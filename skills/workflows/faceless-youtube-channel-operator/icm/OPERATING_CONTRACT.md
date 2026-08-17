# ICM Operating Contract — Faceless YouTube Channel OS

## Identity

This folder governs the single Hermes faceless-YouTube operating system. It is not a collection of user-facing skills. Upstream skills, scrapers, research agents, SEO routines, Sollo, HyperAgent, and publishing adapters are internal providers selected by the orchestrator.

## Outcome

Operate a faceless YouTube channel with minimal owner time by letting AI agents handle research, validation, packaging, scripting, production preparation, quality review, repurposing, analytics, and learning loops while preserving explicit approval for the few actions that can create material external consequences.

## Owner-time objective

Default owner interaction is limited to:

1. **Channel thesis approval** — approve the niche/audience/monetization thesis before the first production cycle or a major pivot.
2. **Spend approval** — approve any new paid tool, ad spend, contractor spend, or material generation expense outside a pre-approved budget.
3. **Publish approval** — approve a finished video/Short before first publication until publishing authority is separately promoted by evidence.
4. **Account/authentication intervention** — complete login, CAPTCHA, MFA, OAuth consent, or recovery when required.
5. **Exception decisions** — copyright/policy ambiguity, material factual dispute, reputational risk, or strategy pivot.

Everything else should be agent-handled by default when access is already authorized.

## Core law

`evidence -> decision -> package -> script -> production -> independent QA -> approval gate -> publish -> verify -> measure -> learn`

Do not invert this into `generate -> publish -> hope`.

## Autonomous responsibilities

Hermes may automatically:

- maintain benchmark creator corpora and provenance;
- research niches, channels, topics, competitors, communities, search surfaces, and first-party sources;
- score channel opportunities and kill weak theses;
- generate research packs and source manifests;
- generate title/thumbnail hypotheses before scripting;
- write original scripts from evidence;
- prepare shot lists, B-roll plans, visual manifests, voiceover instructions, editing plans, chapters, descriptions, tags, pinned-comment drafts, and end-screen bridges;
- use approved production providers such as Sollo for draft generation;
- run independent factual, originality, retention, packaging, policy, monetization, and production-feasibility reviewers;
- generate Shorts and social derivatives from approved source videos;
- collect and normalize analytics where already authorized;
- compare performance against the channel's own baseline;
- feed measured lessons back into topic, package, hook, script, production, CTA, and monetization decisions;
- pause or kill an experiment when predeclared kill criteria fail.

## ICM state model

Every channel has one durable state object with these phases:

`DISCOVER -> VALIDATE -> PACKAGE -> SCRIPT -> PRODUCE -> QA -> APPROVAL -> PUBLISH -> DISTRIBUTE -> MEASURE -> LEARN`

A stage may be `pending | running | blocked | failed | passed | approved | skipped`.

The orchestrator must always know:

- current channel thesis;
- current experiment;
- current video job;
- evidence coverage;
- active blockers;
- next automatic action;
- whether human approval is required;
- rollback path.

## Agent topology

Hermes is the governor. Specialist agents are disposable workers and cannot approve themselves.

### Research swarm

May run in parallel:

- topic/source researcher;
- competitor/outlier researcher;
- SEO/surface researcher;
- community/pain researcher;
- benchmark-pattern researcher.

### Strategist

Consumes condensed research plus raw evidence pointers and chooses:

- thesis fit;
- video angle;
- target viewer;
- monetization intent;
- long-form vs Short role;
- package candidates;
- success metric.

### Creator

Produces script and production artifacts only after package lock.

### Independent critics

At least one independent critic must inspect creator output. For higher-risk content, use separate critics for facts, originality/policy, and packaging/retention.

### Publisher

Cannot publish until the approval policy permits it. It verifies the live URL and metadata after publication.

### Analyst

Collects post-publish metrics, writes lessons, and updates the graph. It cannot rewrite history to justify the original thesis.

## Provider routing

Providers are implementation details:

- personal evidence: `youtube-personal-intelligence`;
- public transcripts/channel research: YouTube APIs / ZeroPoint patterns / transcript providers;
- deep creator craft: AgriciDaniel patterns;
- surface SEO/packaging: deeployCO patterns;
- parallel research/production state: TubeFlow patterns;
- adaptive public extraction: Scrapling;
- browser/authenticated UI: HyperAgent/CDP;
- public-web fallback: Firecrawl or Bright Data when configured;
- draft production: Sollo browser adapter;
- growth-tool registry: advisory only.

Hermes chooses providers automatically by reliability, authorization, cost, and evidence quality. Do not ask the owner to choose between internal tools unless cost or authority changes.

## Benchmark truth — One Person Business

The 2026-08-16 corpus is the current reference benchmark. It proved:

- 25/25 requested newest long-form videos collected;
- 10/30 requested Shorts collected because the channel has only 10 Shorts total;
- 35/35 transcript attempts;
- 30 complete transcripts;
- 5 partial transcripts;
- 0 unavailable transcripts;
- 684 graph nodes;
- 713 graph edges;
- Skool course content still requires authenticated access;
- thumbnails were not image-captured in the corpus and therefore visual thumbnail conclusions are provisional.

Treat this as `PARTIAL` only because the requested 30 Shorts do not exist and Skool remains unauthenticated—not because the YouTube acquisition failed.

## Creator doctrine adopted as hypotheses, not universal truth

The benchmark supports testing these operational hypotheses:

- validate demand from recent outlier channels rather than intuition alone;
- prefer founder/topic fit over trend chasing;
- model proven packaging structures across niches without copying expression;
- package before scripting;
- seek novelty/lesser-known facts to differentiate the script;
- use long-form where high-value audiences and monetization support it;
- favor sustainable simple production over fragile overproduction;
- optimize for revenue/value per view rather than vanity views alone;
- use content as distribution into owned offers/audience assets;
- validate before scaling the factory.

Any numeric rule from the benchmark must be treated as a testable heuristic unless first-party channel data confirms it.

## Completion receipt

Every autonomous cycle ends with:

```text
DECISION
MODE
CHANNEL
EXPERIMENT
CURRENT STAGE
AUTOMATIC ACTIONS COMPLETED
EVIDENCE
ARTIFACTS
QA
METRICS
COMMERCIAL IMPACT
RISKS
ROLLBACK
STATUS
HUMAN APPROVAL: none|required
NEXT AUTOMATIC ACTION
```
