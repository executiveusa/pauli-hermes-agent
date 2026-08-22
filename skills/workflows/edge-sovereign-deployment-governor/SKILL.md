---
name: edge-sovereign-deployment-governor
description: ICM deployment router for Cloudflare, Coolify/VPS, Vercel, and hybrid architectures. Inspects the workload before choosing a platform, prefers Cloudflare for edge-native HTTP/event-driven work, preserves Coolify for persistent owner-controlled compute, treats Vercel as an explicit exception rather than the default, requires cost/rollback/proof gates, and detects agent-commerce opportunities such as AI readiness, MCP/API access, and metered resources.
version: 1.0.0
author: Bambu / Pauli Effect
license: MIT
tags: [icm, cloudflare, workers, pages, durable-objects, workflows, agents, containers, r2, coolify, vps, vercel, deployment, sovereignty, rollback, cost-governance, agent-commerce, x402]
triggers:
  - where should this deploy
  - deploy this to Cloudflare
  - migrate this off Vercel
  - Cloudflare or Coolify
  - Cloudflare or VPS
  - sovereign deployment
  - agent-ready deployment
  - edge deployment
entry_point: /edge-sovereign-deployment-governor
---

# Edge-Sovereign Deployment Governor

## Purpose

Choose the smallest, safest, most economical execution surface for each workload instead of treating a hosting vendor as the architecture.

The default topology is:

```text
Internet / agents / humans
        |
        v
Cloudflare DNS + TLS + WAF + cache + bot/AI controls
        |
        +--> Pages / Workers / Agents / Workflows / Queues / Durable Objects / R2
        |
        +--> Cloudflare Tunnel / protected origin
                 |
                 v
            Coolify / VPS
            persistent services
            arbitrary Docker
            private control plane
            databases when intentionally self-hosted
```

Cloudflare is the preferred **edge and burst plane**. Coolify/VPS is the preferred **persistent sovereign compute plane**. Vercel is an explicit exception when its framework-specific value is proven to exceed its cost, governance, and lock-in tradeoffs.

## Mandatory brownfield contract

Before any deployment decision record:

- MODE: greenfield | brownfield
- OUTCOME: measurable user/system result
- TARGET: app, users, domains, execution surface
- CURRENT STATE: provider, repo/ref, runtime, domains, data, queues, storage, cron, background work, billing plan
- CONSTRAINTS: what must not change
- PROOF: health, functional, data, security, cost and rollback evidence
- COMMERCIAL VALUE: revenue, savings, retention or validated learning
- OWNER CONTROL: code, domain, DNS, database, credentials, backups and account ownership
- ROLLBACK: exact prior ref/deployment/origin/DNS state

Never migrate merely because another platform is newer or cheaper.

## Workload classifier

### Route to Cloudflare Pages

Prefer Pages for:

- static marketing and nonprofit sites;
- documentation and content sites;
- static SPA builds;
- frontends whose dynamic work is already in APIs/Supabase/Workers;
- sites that benefit from global caching and simple preview deploys.

### Route to Cloudflare Workers

Prefer Workers for:

- HTTP APIs and MCP endpoints;
- request/response transformations and gateways;
- auth, routing and policy enforcement;
- lightweight SSR or dynamic applications compatible with the Workers runtime;
- globally distributed bursty compute;
- webhook receivers and event-driven functions.

### Route to Durable Objects / Agents

Prefer Durable Objects or the Cloudflare Agents SDK when the workload needs per-user/per-tenant identity, strongly coordinated state, WebSockets, scheduled per-entity work, recoverable agent sessions, or durable conversational state.

Do not use one giant Durable Object as a global database. Shard by the entity that owns coordination.

### Route to Workflows + Queues

Prefer Workflows for multi-step work that must survive failure, retry, pause, wait for external events, or run beyond a normal request lifecycle. Prefer Queues when producers and consumers should be decoupled and bursts should be buffered.

### Route to Cloudflare Browser / Sandbox / Containers

Use Browser for bounded rendered-page inspection and automation.

Use Sandbox/Containers when a job needs Linux tooling, a filesystem, non-JavaScript runtimes, isolated code execution, or more CPU/memory than a Worker. Treat these as on-demand serverless containers, not as a blanket replacement for an always-on VPS.

### Route to R2

Use R2 for artifacts, backups, media, data products, logs, generated exports, crawl results and other object/blob storage where S3-compatible access and low-egress architecture are useful.

### Route to Coolify / VPS

Keep or move a workload to Coolify/VPS when one or more of these is load-bearing:

- always-on daemon/process semantics;
- arbitrary Docker Compose stacks or multiple tightly coupled services;
- inbound non-HTTP TCP/UDP;
- sustained CPU/RAM where serverless economics are worse;
- system packages/kernel/OS-level control;
- private networking or self-hosted control planes;
- databases/services requiring operational ownership or provider portability;
- long-lived local disk semantics not intentionally externalized;
- GPU or unsupported runtime requirements;
- a brownfield system already proven stable on the VPS where migration adds risk without customer value.

Cloudflare should still normally sit in front for DNS/TLS/WAF/cache and can route to the VPS through a protected origin/Tunnel.

### Route to Vercel only by exception

Vercel is acceptable when a current production requirement depends materially on Vercel/Next.js platform primitives, the cost envelope is understood, account/billing ownership is explicit, spend controls are configured, and a tested export/rollback path exists.

Do not choose Vercel because the repo happens to be React/Next.js. Runtime requirements decide the host.

## Decision matrix

| Need | First choice | Why |
|---|---|---|
| Static/site/docs | Cloudflare Pages | simple global delivery |
| API/MCP/webhook | Workers | edge-native request compute |
| Stateful realtime entity | Durable Objects / Agents | coordinated durable identity |
| Multi-step/retry/wait | Workflows | durable execution |
| Burst buffering | Queues | decoupled async work |
| Browser inspection | Browser | managed rendered automation |
| Linux job / isolated code | Sandbox/Containers | on-demand container execution |
| Object/data artifact | R2 | object storage, portable access |
| Persistent daemon / arbitrary stack | Coolify/VPS | full process/OS control |
| Existing stable self-hosted service | Keep + Cloudflare front door | avoid migration risk |
| Vercel-specific Next.js capability | Vercel exception | use only when value is proven |

## Cost governor

For every candidate deployment compare:

1. idle cost;
2. cost per request/job;
3. storage and egress;
4. retries and failure amplification;
5. observability cost;
6. human operations cost;
7. vendor-exit cost;
8. worst-case monthly spend.

Do not optimize only for the cheapest nominal plan. Prefer the architecture with the lowest **risk-adjusted total cost** for the proven workload.

Any paid service must have:

- budget owner;
- hard/soft spend alerts where supported;
- named billing account;
- no autonomous plan upgrade;
- no agent authority to buy capacity without explicit approval.

## Deployment pipeline contract

Production is a promotion, not a build.

```text
inspect
 -> classify
 -> baseline
 -> build/test
 -> deploy preview OR upload version
 -> verify preview
 -> security/cost checks
 -> independent review
 -> explicit production promotion
 -> verify public target
 -> retain rollback
```

Rules:

- `push` may build/test and create a preview/version.
- Production promotion requires an explicit environment gate or dispatch unless the owner has separately approved a repository-specific automatic production policy.
- Record the source commit and resulting deployment/version.
- Never expose Cloudflare/VPS/Vercel tokens in logs or committed files.
- Builder and verifier must be separate roles.
- A successful CI job is not production proof.

## Agent-internet opportunity scan

After a workload is classified, check whether the same infrastructure creates a sellable resource layer without increasing production risk.

Ask:

1. What information or action do customers/agents need repeatedly?
2. Is it fragmented, changing, expensive, or annoying to obtain?
3. Can it be exposed as a trustworthy API, MCP tool, search index, feed, or structured file?
4. Which access should be free, authenticated, private, subscription, or usage-metered?
5. Can the human/service version be sold before an x402 payment rail is required?
6. Does the resource become more valuable as observations, outcomes, freshness, or provenance accumulate?

Do not make x402 load-bearing while Monetization Gateway/pay-per-crawl remain limited rollout/beta capabilities. Build the sellable service with ordinary invoicing/subscriptions first; add machine-native metering later as an optional channel.

## Revenue routing

When the scan finds a credible opportunity, route it through the existing commercial path rather than creating a new offer family:

```text
Vibe Audit
 -> Vibe Rescue Sprint
 -> Sovereign Launch
 -> MAXX Operations
```

The Cloudflare/agent layer is a capability inside that path:

- Vibe Audit: show what AI systems and crawlers can/cannot understand today; include hosting/cost/ownership risk.
- Rescue Sprint: fix source-of-truth, structured content, pricing/proof/schema, llms.txt, APIs/MCP as justified.
- Sovereign Launch: move the public edge to Cloudflare and persistent workloads to owner-controlled Coolify where appropriate; prove rollback.
- MAXX Operations: continuously measure AI answers, crawl access, uptime, spend, freshness and business outcomes.

## Portfolio governor

This skill may not create a fourth active workstream.

Classify every proposed Cloudflare experiment as SELL, USE, MERGE, PARK or ARCHIVE.

- SELL: must have a realistic paid customer path inside 30 days.
- USE: must materially improve delivery of the active revenue offer or shared platform.
- MERGE: fold into an existing product/skill.
- PARK/ARCHIVE: no engineering allocation.

Default active allocation for this strategy:

1. REVENUE OFFER: Agent-Ready Sovereign Launch (sell the audit first).
2. SHARED PLATFORM: Hermes deployment governor + reusable Cloudflare/Coolify deployment pipeline.
3. BOUNDED EXPERIMENT: one metered agent-readable resource/data product; x402 is optional and must not be the revenue assumption.

## Completion record

```text
DECISION
CHANGES
PROOF
STATUS
COMMERCIAL IMPACT
RISKS
ROLLBACK
NEXT
HUMAN APPROVAL
```

Statuses: PASS | HOLD | BLOCKED | NOT_RUN | UNVERIFIED.
