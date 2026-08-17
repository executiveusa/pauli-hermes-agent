# YouTube Growth Tool Landscape

Source reviewed: `https://github.com/wvn774/best-youtube-growth-tools`

## Classification

This upstream repository is a curated list/article, not an executable skill suite. It currently contains a large `README.md` plus images, with no `SKILL.md`, no integration code, and no declared repository license.

Hermes MUST NOT vendor, execute, or present this repository as a software dependency.

Use it only as a discovery taxonomy for categories of YouTube growth tooling.

## Useful capability categories extracted

Hermes may consider external tools in these buckets when a verified gap exists:

- keyword and topic research;
- channel/video SEO;
- thumbnail creation and testing;
- competitor benchmarking;
- audience and performance analytics;
- retention analysis;
- publishing and scheduling;
- cross-platform distribution;
- comment/community management;
- social listening;
- creator/channel tracking;
- trend discovery;
- production/recording;
- asset and creative management.

Examples mentioned by the upstream guide include vidIQ, TubeBuddy, Social Blade, Hootsuite, Morningfame, Canva, Ahrefs, Sprout Social, Buffer, Tubular Labs, Rival IQ, and others. These names are discovery candidates, not endorsed dependencies.

## Hermes adoption policy

Default order:

1. YouTube Data API / YouTube Analytics / YouTube Studio exports.
2. Existing Hermes transcript/search/graph capabilities.
3. Local/open tooling already available to Hermes.
4. Existing connected platform already paid for by the owner.
5. Third-party growth service only if a specific missing capability is proven and expected value exceeds cost/lock-in.

Do not add a SaaS because it appears on a "best tools" list.

Before recommending or integrating any paid service, record:

- GAP: exact capability Hermes cannot currently provide reliably;
- EVIDENCE: why native/current tooling is insufficient;
- COST: subscription/API/usage cost;
- DATA ACCESS: what account/channel data the service receives;
- EXPORTABILITY: whether results/data can be exported;
- LOCK-IN: dependency and cancellation risk;
- COMMERCIAL VALUE: measurable revenue, retention, savings, or validated learning;
- EXIT PLAN: native or self-hosted fallback.

## Growth-tool routing

### Research / keywords
Prefer native YouTube search signals, current public research, transcript intelligence, and first-party channel data. External keyword tools are optional enrichment.

### Analytics / retention
Prefer YouTube Studio and YouTube Analytics as source of truth. Never infer private CTR, APV, audience retention, or traffic-source performance from a third-party estimate when first-party data is available.

### Competitor intelligence
Use public metadata, transcripts, publishing cadence, topic clusters, format patterns, and visible performance. Treat external revenue estimates and proprietary scores as estimates, not facts.

### Thumbnail experimentation
Prefer YouTube's native experimentation/testing capabilities where available. External thumbnail/design tools may assist creation, not become the measurement source of truth.

### Distribution
Only adopt cross-platform schedulers when Hermes cannot accomplish the approved distribution workflow through existing connected systems or native APIs.

### Community management
Comment replies, moderation, and public engagement are write actions and remain governed by Hermes approval policy.

## Anti-duplication rule

The presence of a third-party tool does not create a new Hermes skill. Route the request through `youtube-personal-intelligence` and lazy-load/invoke the minimum external capability only when justified.

## Verification rule

The upstream README was published as a 2025 tool list. Product features, pricing, integrations, limits, and availability are time-sensitive. Hermes must verify current vendor documentation before making a purchase recommendation or integration decision.
