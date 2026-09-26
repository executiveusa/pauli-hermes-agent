# YouTube SEO Specialist

Source skill: `deeployCO/youtube-seo-skills/youtube-seo/SKILL.md`

## Role

Lazy-loaded specialist for discovery, packaging, ranking diagnosis, metadata, thumbnails, and YouTube search optimization. It does not own personal-history ingestion, channel strategy, or publishing authority.

## Routing

- full channel SEO audit -> `youtube-seo-audit`
- single video diagnosis -> `youtube-seo-video`
- title/description/tags optimization -> `youtube-seo-optimize`
- channel branding/about/playlists -> `youtube-seo-channel`
- keyword/topic research -> `youtube-seo-keywords`
- thumbnail/CTR packaging -> `youtube-seo-thumbnail`
- competitor SEO analysis -> `youtube-seo-competitor`

## Surface-aware optimization

Never treat YouTube ranking as one keyword score. First identify the target surface:

- Browse/Home: packaging, viewer fit, session continuation, freshness, satisfaction.
- Suggested: topical adjacency and next-video/session continuation.
- Search: query intent, semantic/entity match, click satisfaction, watch quality for the query.
- Shorts: immediate hook, swipe resistance, completion/rewatch/loop behavior.
- Notifications: subscriber affinity and open behavior.
- External: retention and conversion of new viewers.

A video may be healthy on one surface and weak on another. Recommendations must name the target surface.

## Evidence hierarchy

Prefer first-party or directly observable evidence:

1. YouTube Studio analytics/CSV supplied through an authorized account/export.
2. YouTube Data API v3 for supported metadata/statistics.
3. ZeroPoint transcript/channel/search skill for transcript and public research enrichment.
4. yt-dlp or equivalent public metadata fallback where permitted by runtime policy.
5. Browser inspection only as a last read-only gap-filler.

Do not invent CTR, retention, traffic-source, returning-viewer, or impression data. If the task depends on Studio-only signals and they are unavailable, mark the diagnosis as incomplete.

## Metrics model

Prioritize:

- CTR by traffic surface;
- average percentage viewed / average view duration;
- first-30-second retention;
- retention cliffs;
- session continuation;
- returning viewers;
- subscribe-from-video rate;
- end-screen/card continuation;
- saves/shares/comments relative to channel baseline;
- first-24h velocity for topical releases;
- channel topical coherence and playlist binge paths.

Numerical benchmark tables from upstream are heuristics, not ground truth. Compare against the channel's own baseline and current niche evidence before making a strong claim.

## Packaging review

Check:

- title promise and target surface;
- thumbnail legibility at mobile size;
- title-thumbnail complementarity rather than duplication;
- description above-fold intent clarity;
- chapters/key moments where appropriate;
- captions/translations when useful;
- entity/topic coverage without keyword stuffing;
- playlists/end screens for session continuation;
- copyright, disclosure, synthetic-content, and Made-for-Kids requirements.

## Output contract

Return:

1. target surface;
2. evidence available / missing;
3. scorecard with explicit uncertainty;
4. issues ordered Critical -> High -> Medium -> Low;
5. paste-ready title/description/tag/thumbnail concepts when requested;
6. rationale tied to the relevant YouTube signal;
7. measurement plan with the exact Studio metric, observation window, and success/failure threshold;
8. no publish/change action without the parent skill's approval gate.

## Integration boundary

This reference supplements `youtube-personal-intelligence/SKILL.md`.

- ZeroPointRepo = transcripts/search/channel data.
- AgriciDaniel = creator strategy/scripts/monetization/repurposing.
- deeployCO = SEO/search/packaging specialist.
- Hermes = personal account intelligence, graph, orchestration, provenance, approval, and proof.
