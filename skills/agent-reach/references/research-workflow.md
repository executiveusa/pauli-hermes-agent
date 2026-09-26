# Cross-platform research workflow

Use this workflow when the user asks for a deep dive, market scan, source
comparison, sentiment check, product research, or “what are people saying?”

## Stage 0 — Lock the question

Record:

- decision or outcome the research must support;
- subject/entities;
- date window;
- geography/language;
- required platforms;
- result limit;
- whether outputs should remain temporary or enter the user's ICM/second brain.

Do not ask questions that can be resolved from the supplied URL or task.

## Stage 1 — Health and routing

```bash
agent-reach doctor --json
```

Create a route plan from `active_backend` values. Announce the selected route
briefly, for example: “Using Agent Reach: YouTube via yt-dlp, web via Exa/Jina,
and Reddit via OpenCLI.”

## Stage 2 — Source plan

Use different source classes for different jobs:

| Need | Sources |
|---|---|
| Current discovery | Exa/web search, recent platform search |
| Primary facts | official sites, documentation, repositories, original videos |
| Practitioner experience | Reddit, V2EX, GitHub issues/discussions |
| Public reaction | YouTube comments, Twitter/X, Instagram/Facebook, XiaoHongShu |
| Long-form explanation | YouTube transcripts, articles, podcasts |
| Software truth | repository, releases, issues, documentation |

Do not treat social sentiment as authoritative product documentation.

## Stage 3 — Parallel collection

Run independent reads in parallel with conservative concurrency. Suggested
limits:

- web/search requests: 5 concurrent;
- transcript jobs: 3 concurrent;
- login-backed social requests: sequential or 2 concurrent;
- 2–3 second spacing for platforms prone to verification challenges.

Collect the smallest sufficient evidence set. Avoid scraping thousands of items
when 20–50 representative records answer the question.

## Stage 4 — Normalize

Create one source record per item using the schema in `SKILL.md`. Preserve raw
URLs and timestamps. Deduplicate by canonical URL, post/video ID, or content
hash.

For each extracted claim, label it:

- `primary` — official/original source;
- `reported` — reputable secondary source;
- `experience` — user/practitioner report;
- `sentiment` — opinion/reaction;
- `inference` — Hermes conclusion from evidence.

## Stage 5 — Verify

For load-bearing claims:

1. prefer primary sources;
2. seek a second independent source;
3. check publication/version dates;
4. record disagreement rather than averaging it away;
5. separate “not found” from “does not exist.”

## Stage 6 — Synthesize

Use this report shape:

```markdown
# Research question

## Decision-ready answer

## Strongest evidence

## What users/practitioners report

## Disagreement and uncertainty

## Source coverage

| Platform | Backend | Items read | Status |
|---|---|---:|---|

## Recommended next action

## Sources
```

Every non-obvious factual claim should map to a source. Do not dump raw search
results without synthesis.

## Stage 7 — Persist only by request

Default temporary paths:

```text
/tmp/agent-reach-<task-id>/
```

When the user requests second-brain storage, create an ICM package:

```text
<topic>/
├── CONTEXT.md
├── sources/
│   ├── source-index.json
│   └── transcripts/
├── output/
│   ├── research-brief.md
│   └── claims.json
└── references/
```

`CONTEXT.md` should state the question, scope, retrieval date, source coverage,
known gaps, and how another agent should reuse the material.

## Stage 8 — Closeout

```bash
agent-reach check-update
```

Return:

- answer;
- sources;
- blocked channels;
- saved artifact locations;
- one next action.

## Failure policy

- Retry transient failures once with backoff.
- Follow documented backend retry chains; never guess new commands.
- Do not bypass authentication, paywalls, private content, robots controls, or
  platform access restrictions.
- If a platform is blocked, continue with independent sources and explicitly
  lower confidence.
