---
name: prospect-seo-intelligence
description: Use public website, search, local, technical SEO, content, schema, and AI-search evidence to prepare Bambu/Jeremy for prospect and client conversations before the first call, and to improve owned/client sites through the Vibe commercial path.
---

# Prospect SEO Intelligence

## Owner
This is a Bambu/Jeremy Hermes operating skill. It is part of Jeremy's personal Hermes workflow and should be available whenever a prospect, client, owned site, launch, rescue, or growth review involves a public web presence.

## Source capability
Primary source implementation:
- `executiveusa/pauli-claude-seo`
- upstream lineage: `AgriciDaniel/claude-seo`
- license: MIT

Claude SEO is a Claude Code plugin with a large SEO workflow surface. Do not claim it is natively executable by Hermes merely because this orchestration skill exists. Hermes may use it through an available Claude Code/runtime adapter or equivalent verified execution path. If no compatible executor is available, mark execution `BLOCKED` and still perform only the reconnaissance Hermes can verify with its own available tools.

Never duplicate the full upstream skill tree into Hermes just to appear integrated. Prefer one orchestration skill that delegates to the maintained source capability.

## Mission
Before Jeremy/Bambu takes a first sales call, use public evidence to understand the prospect's website/search situation well enough to:
1. avoid asking questions we can answer ourselves;
2. identify observable leaks and opportunities;
3. separate verified facts from hypotheses;
4. prepare a sharper discovery call;
5. recommend the smallest commercially valuable next step;
6. produce proof assets that can later support a Vibe Audit, Vibe Rescue Sprint, Sovereign Launch, or MAXX Operations engagement.

This is not a license to diagnose the entire business from its website. SEO/search evidence informs the diagnosis; it does not replace customer, operational, financial, or fulfillment evidence.

## Default triggers
Run this skill when:
- a new prospect with a public website is identified;
- Jeremy asks for a pre-call brief;
- a proposal or audit is being prepared;
- a Vibe Audit begins;
- a site is being considered for rescue/rebuild;
- an owned portfolio site needs a baseline or growth review;
- a client asks about SEO, local visibility, schema, content quality, AI search, Google visibility, Core Web Vitals, rankings, citations, or competitor search presence;
- a recurring MAXX Operations review needs search/visibility drift evidence.

Do not wait for the prospect to explain obvious public facts that can be verified before the call.

## Hard invariants
1. Public reconnaissance first; private/client systems require permission.
2. Never claim rankings, traffic, revenue, leads, conversions, GSC data, GA4 data, GBP ownership, backlink metrics, or AI share-of-voice unless the underlying evidence was actually retrieved.
3. Label findings `VERIFIED`, `CLIENT-STATED`, `INFERRED`, or `UNKNOWN`.
4. Never equate an SEO health score with revenue loss or business value. Revenue impact is a hypothesis until tied to traffic, conversion, lead, deal, or customer evidence.
5. Do not change the prospect's site, DNS, analytics, GBP, Search Console, content, schema, ads, or credentials during reconnaissance.
6. Do not submit forms, send outreach, create accounts, or trigger live changes merely to complete an audit.
7. Respect robots, access controls, rate limits, terms, privacy, and legal boundaries.
8. Prefer primary-source Google/search documentation for technical claims.
9. Competitor comparison is evidence gathering, not permission to copy content or branding.
10. No guaranteed-ranking language. Do not promise first-page placement, AI citations, traffic, or revenue.
11. Builder cannot approve its own audit. Material client-facing conclusions should be independently reviewed before delivery when feasible.
12. When credentials or paid data sources are absent, state the limitation instead of inventing certainty.

## Core Claude SEO capabilities to use when available
Use the smallest relevant subset, not every command by default.

### Baseline
- `/seo doctor` — prove runtime readiness.
- `/seo audit <url>` — full-site audit, business-type detection, health score, prioritized action plan.
- `/seo page <url>` — deep page-level analysis.
- `/seo technical <url>` — crawlability, indexability, security, URL structure, mobile, Core Web Vitals, structured data, JS rendering, IndexNow.

### Content and discovery
- `/seo content <url>` — E-E-A-T/content quality and freshness.
- `/seo geo <url>` — AI-search / generative-search citability and entity clarity.
- `/seo schema <url>` — detect/validate structured data and opportunities.
- `/seo images <url>` — image SEO, performance, accessibility-related signals.
- `/seo sitemap <url>` — sitemap health and coverage.

### Market-specific
- `/seo local <url>` — local SEO signals for location-based businesses.
- `/seo maps ...` — map/GBP/review/competitor intelligence when the required integration exists.
- `/seo hreflang <url>` — international/multilingual sites.
- `/seo ecommerce <url>` — commerce-specific analysis.
- `/seo programmatic <url>` — programmatic SEO opportunities/risks.

### Evidence extensions when configured
- `/seo google ...` — PageSpeed, CrUX, GSC, URL Inspection, GA4, sitemap/indexing evidence depending credential tier.
- `/seo backlinks <url>` — backlink analysis from configured sources.
- `/seo ahrefs ...`, `/seo dataforseo ...`, `/seo seranking ...`, `/seo profound ...`, `/seo bing ...`, `/seo unlighthouse ...` — use only when those integrations are actually available and authorized.

## Pre-call reconnaissance workflow

### Step 1 — Resolve identity
Confirm:
- exact business name;
- canonical domain;
- primary location(s);
- apparent services/products;
- primary customer type;
- likely conversion action: call, form, booking, checkout, donation, signup, visit, etc.

If identity is ambiguous, do not merge multiple companies into one brief.

### Step 2 — Build the public surface map
Inspect, where applicable:
- homepage;
- key service/product pages;
- about/team/trust pages;
- contact/booking/lead path;
- pricing or offer page;
- blog/resources;
- location pages;
- sitemap/robots;
- search-result snippets;
- local/GBP/map presence when publicly accessible;
- 3-5 relevant competitors when comparison adds value.

### Step 3 — Run the smallest useful SEO evidence set
Default prospect scan:
1. `/seo doctor`
2. `/seo audit <domain>`
3. `/seo page <homepage>`
4. `/seo technical <domain>`
5. `/seo geo <homepage or strongest money page>`
6. `/seo schema <domain>`
7. `/seo local <domain>` only for local/location-driven businesses

Add deeper commands only when the initial evidence justifies them.

### Step 4 — Map findings into the studio diagnostic
Use the broader 8-dimension business diagnostic, but only fill dimensions supported by evidence:
- **Market** — public niche/location/competitor evidence only.
- **Offer** — clarity of public offer, service/product structure, price/CTA visibility.
- **Positioning** — differentiation, trust, expertise, entity clarity, competitor contrast.
- **Acquisition** — crawlability, search visibility evidence, local SEO, content coverage, backlinks when verified.
- **Conversion** — public page experience, CTA path, trust friction, mobile/performance signals; do not invent conversion rates.
- **Fulfillment** — usually `UNKNOWN` before the call unless verifiable public evidence exists.
- **Retention** — usually `UNKNOWN` before the call unless verifiable public evidence exists.
- **Measurement** — public analytics/tagging/schema/search instrumentation signals plus verified GSC/GA4 only if authorized.

### Step 5 — Identify the primary observable constraint
Do not produce a 50-item scare list.
Choose:
- one primary constraint;
- up to three supporting leaks/opportunities;
- evidence for each;
- what would falsify the conclusion;
- what additional information is required on the call.

Examples:
- technically healthy site but weak offer/positioning;
- strong local business but weak local/search discoverability;
- good content but indexability/schema problems;
- visible demand opportunity but poor conversion path;
- no credible evidence that SEO is the main constraint.

The last outcome is valid. Do not force an SEO sale.

## Required pre-call output
Produce a compact `Prospect Intelligence Brief` with:

### Identity
Business, domain, location, apparent offer, customer, conversion action.

### What we verified
5-10 high-value facts with evidence labels.

### Search / website baseline
- health score if actually produced;
- technical blockers;
- content/trust findings;
- schema findings;
- local findings if relevant;
- AI-search/citability findings;
- competitor deltas that are actually observed.

### Primary constraint hypothesis
One sentence, clearly marked `INFERRED` until the call validates it.

### Top 3 observable opportunities
For each:
- evidence;
- likely business consequence;
- confidence;
- what proof is still missing.

### Questions we still need to ask
Only ask what public reconnaissance could not answer. Prioritize questions about:
- actual lead/customer volume;
- conversion rates;
- highest-margin services/products;
- geographic/service priorities;
- sales capacity;
- fulfillment bottlenecks;
- retention/repeat business;
- access to GSC/GA4/CRM/GBP;
- acceptable budget/timeline;
- ownership/hosting constraints.

### Recommended commercial next step
Choose the smallest justified module:
- **Vibe Audit** when diagnosis/proof is still the need;
- **Vibe Rescue Sprint** when a bounded, high-confidence repair is visible;
- **Sovereign Launch** when the current site/platform should be replaced or a new owned launch is justified;
- **MAXX Operations** when the system is already healthy enough for recurring optimization/monitoring.

Do not recommend a bigger engagement merely because more work exists.

## Offer/package integration
SEO and AI-search intelligence is a **module inside the existing commercial path**, not a fifth core offer.

### Vibe Audit
Include a `Search & AI Visibility` evidence section when relevant:
- technical SEO baseline;
- Core Web Vitals / performance evidence when available;
- crawl/indexability;
- content/E-E-A-T;
- structured data;
- local SEO;
- AI-search/GEO citability;
- competitor search gaps;
- measurement/access gaps;
- prioritized repair hypotheses.

### Vibe Rescue Sprint
Implement only the validated high-value repairs, for example:
- crawl/indexing fixes;
- metadata/headings/canonical cleanup;
- performance issues;
- schema corrections;
- local landing-page fixes;
- content architecture/internal linking;
- sitemap/hreflang issues;
- measurable conversion-path fixes that are within scope.

Proof must compare before/after evidence. Do not sell ranking guarantees.

### Sovereign Launch
Build search readiness into the launch definition of done:
- crawlable/indexable architecture;
- clean metadata/canonicals;
- sitemap/robots;
- structured data;
- Core Web Vitals target evidence;
- international/local setup when relevant;
- analytics/Search Console ownership under the client;
- portable content/data ownership;
- baseline report captured at launch.

### MAXX Operations
Use recurring evidence for:
- SEO drift monitoring;
- Core Web Vitals/CrUX trends;
- GSC/GA4 changes when authorized;
- broken/indexing regressions;
- content decay/opportunity;
- schema regressions;
- local visibility/review intelligence when configured;
- AI-search/share-of-voice tracking only when the relevant provider is available.

## Owned-site use
Run the same discipline on our own portfolio:
- baseline before major changes;
- verify launches rather than assuming SEO is intact;
- use drift monitoring after release;
- identify content/market opportunities from evidence;
- never create hundreds of location/programmatic pages without demand, quality, duplication, and indexation gates;
- preserve source reports as proof assets for future case studies.

## Prospecting use
This skill can support outbound prospecting, but the audit itself is not permission to spam.

Use the findings to make outreach specific:
- mention one or two verified observations;
- avoid fabricated traffic/revenue claims;
- avoid presenting a full unpaid consulting deliverable before qualification;
- retain the deeper brief internally for the call;
- lead with the business consequence, not an SEO jargon dump.

Example internal logic:
`public evidence -> likely constraint -> one useful observation -> call -> validate economics -> paid diagnostic/repair`.

## Evidence tiers
- **Tier 0 — public/no credentials:** site crawl, page/technical/content/schema/GEO/local public evidence.
- **Tier 1 — authorized search data:** GSC, URL Inspection, sitemap/indexing status.
- **Tier 2 — authorized analytics:** GA4 organic/landing/device/country evidence.
- **Tier 3 — commercial SEO data:** keyword/backlink/share-of-voice providers when licensed/configured.

Never describe Tier 0 inference as Tier 1-3 evidence.

## Commercial proof rule
Before claiming this module makes money, collect at least one real receipt tying the workflow to a measurable outcome such as:
- qualified opportunity created;
- paid audit sold;
- rescue sprint sold;
- measurable indexing/performance improvement;
- measurable organic lead improvement;
- measurable local visibility improvement;
- validated rejection showing SEO was not the primary constraint.

A faster audit is operational leverage, not customer value by itself.

## Runtime gate
This Hermes skill is an orchestration contract. The underlying `pauli-claude-seo` capability is proven usable by Hermes only after a test shows:
1. Hermes can invoke the compatible executor/adapter;
2. `/seo doctor` or equivalent succeeds;
3. one public test domain completes an audit;
4. Hermes receives the resulting report/artifacts;
5. the report is transformed into the Prospect Intelligence Brief without invented fields;
6. no secrets are exposed.

Until then mark:
`HERMES_RUNTIME_INTEGRATION = BLOCKED / NOT YET PROVEN`.

## Default status language
Use:
- `VERIFIED` for observed evidence;
- `INFERRED` for reasoned hypotheses;
- `UNKNOWN` when the call or credentials are required;
- `BLOCKED` when a provider/runtime/access dependency prevents proof.

Never use `DONE`, `OPTIMIZED`, `RANKING`, or `READY TO SCALE` without the corresponding evidence.