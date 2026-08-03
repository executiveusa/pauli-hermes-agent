# Reusable Agent Reach prompts for Hermes

These are natural-language task contracts. Hermes should load the Agent Reach
skill automatically from the intent.

## YouTube transcript — complete

```text
Use Agent Reach to inspect this YouTube URL. Get the best available transcript
using manual subtitles first, automatic subtitles second, and speech-to-text
only if no captions exist. Return the title, channel, date, duration, transcript
source, executive summary, timestamped key points, notable short quotes,
implementation actions, and any claims that need verification. Preserve the raw
transcript in a temporary artifact and tell me its path.
```

## Tutorial to ICM learning module

```text
Use Agent Reach to transcribe this tutorial and turn it into an ICM learning
module. Create CONTEXT.md, a source record, a cleaned transcript, concepts,
procedures, commands, risks, and a verification checklist. Keep every important
claim tied to a video timestamp. Do not save into my second brain until the
transcript and source metadata are verified.
```

## Multi-video expert comparison

```text
Use Agent Reach to find the strongest five recent YouTube videos about [TOPIC].
Prefer original experts and primary demonstrations over reaction channels.
Transcribe them, compare where they agree and disagree, identify stale claims,
and produce one decision-ready brief with source timestamps. State search date,
coverage, exclusions, and confidence.
```

## Cross-platform product research

```text
Use Agent Reach to research [PRODUCT/PROJECT] across its official site, GitHub,
YouTube, Reddit, Twitter/X, and relevant technical discussions. Run doctor first
and use each platform's active backend. Separate official claims, practitioner
experience, sentiment, and your inference. Return the strongest opportunities,
recurring complaints, unresolved risks, evidence links, and one recommended
commercial next action.
```

## Open-source repository diligence

```text
Use Agent Reach to inspect [REPO]. Read the README, architecture, recent
releases, active issues, pull requests, and discussions. Then search the wider
web and YouTube for independent usage evidence. Determine what is actually
working, what is marketing, maintenance health, integration cost, security
risks, and whether we should USE, SELL, MERGE, PARK, or ARCHIVE it.
```

## Competitor messaging scan

```text
Use Agent Reach to collect current homepage copy, product docs, launch videos,
and public user discussion for [COMPETITORS]. Extract each competitor's frame,
promise, target user, pricing/offer, proof, objections, and repeated complaints.
Do not copy their language. Produce whitespace opportunities and a differentiated
positioning recommendation for our product.
```

## Social listening

```text
Use Agent Reach to collect a representative sample of public discussion about
[TOPIC] from Reddit, Twitter/X, YouTube comments, and other healthy configured
channels. Keep the workflow read-only. Deduplicate reposts, distinguish volume
from evidence, identify repeated themes and minority warnings, and state which
platforms were unavailable or login-blocked.
```

## Research into second brain

```text
Use Agent Reach to research [TOPIC]. After verification, package the result for
my second brain using the ICM method: CONTEXT.md, source index, transcripts or
raw captures, claims.json, and a decision-ready research brief. Include retrieval
dates, source quality, known gaps, and instructions for another agent to reuse
or refresh the material.
```

## Monitoring workflow specification

```text
Use Agent Reach to identify the best public sources for monitoring [TOPIC].
Return an implementation-ready monitoring plan: sources, RSS feeds, search
queries, channels, cadence, deduplication key, significance rules, storage
schema, alert threshold, failure fallback, and the exact condition under which
Hermes should notify me. Do not create a recurring task until I approve the
plan.
```
