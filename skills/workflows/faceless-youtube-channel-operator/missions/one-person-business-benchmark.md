# One Person Business Benchmark Mission

## Status

YouTube benchmark acquisition is complete to the channel inventory limit.

Current result, collected 2026-08-16:

- long-form: **25/25** newest requested videos;
- Shorts: **10/30 requested**, because the channel has only **10 Shorts total**;
- unique videos: **35**;
- transcript attempts: **35/35**;
- transcripts complete: **30**;
- transcripts partial: **5**;
- transcripts unavailable: **0**;
- graph: **684 nodes / 713 edges**;
- Skool: **SKIPPED_BY_OWNER**;
- thumbnails: metadata/concepts available, image-level visual benchmark still pending.

Do not rerun the 30-Short acquisition unless channel inventory changes. The shortfall is an inventory fact, not a scrape failure.

## Target

Creator: Dave Nick / One Person Business

Public YouTube:

- `https://www.youtube.com/@One-Person-Business`
- `https://www.youtube.com/@One-Person-Business/videos`
- `https://www.youtube.com/@One-Person-Business/shorts`
- verified channel ID at acquisition: `UC5sGRuouCtllvIEL1q8ay0Q`

Private Skool training is explicitly out of scope by owner decision. Do not request authentication or spend time attempting to crawl it unless the owner later reopens that scope.

## Canonical benchmark artifacts

Load:

- `../benchmarks/one-person-business/manifest.json`
- `../benchmarks/one-person-business/creator-doctrine.md`
- `../icm/OPERATING_CONTRACT.md`
- `../icm/AUTHORITY.yaml`
- `../workflows/channel-os.json`
- `../providers/sollo-browser.md`

The original machine-readable extraction package contained normalized video records, timestamped transcripts, pattern library, Shorts and long-form formulas, creator graph, provenance, workflow model, and doctrine. The Hermes benchmark folder stores the durable operating conclusions and coverage receipt; raw source corpora should remain in the approved evidence store rather than being duplicated blindly into skill instructions.

## Remaining mission — Optional evidence completion

Only run when useful:

- complete the five partial transcripts;
- capture thumbnail images for visual pattern analysis;
- refresh the benchmark when meaningful new channel content changes the observed process.

Do not rescrape the entire corpus on every video cycle.

## Autonomous use

Hermes may use the benchmark automatically for:

- niche/outlier research;
- package inspiration;
- hook/title structure analysis;
- novelty research prompts;
- Sollo production routing;
- monetization intent;
- production simplicity heuristics;
- experiment design.

Benchmark claims remain hypotheses until the target channel's own performance data confirms them.

## Proof gate

Do not call benchmark refresh complete until:

- target channel identity is verified;
- requested public inventory counts are recorded;
- transcript coverage is recorded;
- every derived workflow delta has source provenance;
- no credentials/session material were stored;
- owner-facing changes are limited to genuinely new approval requirements.
