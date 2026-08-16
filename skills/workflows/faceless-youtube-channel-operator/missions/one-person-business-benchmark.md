# One Person Business Benchmark Acquisition Mission

## Target

Creator: Dave Nick / One Person Business

Public YouTube:
- Channel: https://www.youtube.com/@One-Person-Business
- Videos: https://www.youtube.com/@One-Person-Business/videos
- Shorts: https://www.youtube.com/@One-Person-Business/shorts
- Verified public channel ID at mission creation: `UC5sGRuouCtllvIEL1q8ay0Q`

Private training:
- https://www.skool.com/obf

## Required sample

- exactly 25 newest long-form videos, newest first;
- exactly 30 newest Shorts, newest first;
- all available transcripts/captions for those 55 items;
- title, URL, video ID, date, duration, views, thumbnail, description, chapters, visible CTA/offer links;
- course inventory and process extraction from owner-authorized Skool session.

If fewer than the required count can be proved, stop and return the exact count plus blocker. Do not substitute guessed or search-indexed lists while claiming completeness.

## HyperAgent browser mission

Authority: READ-ONLY.

### YouTube long-form

1. Open `/videos`.
2. Verify handle/channel identity is One Person Business.
3. Sort/order must remain newest/default; do not change account state.
4. Scroll until at least 25 unique long-form cards are loaded.
5. Extract from DOM for each item:
   - ordinal;
   - title;
   - canonical watch URL;
   - video ID;
   - visible views;
   - visible age/date;
   - duration;
   - thumbnail URL if accessible without exposing auth/session data.
6. Save exactly the first 25 unique long-form items.
7. Verify no `/shorts/` URLs are present.

### YouTube Shorts

1. Open `/shorts`.
2. Verify same channel identity.
3. Scroll until at least 30 unique Shorts are loaded.
4. Extract:
   - ordinal;
   - title;
   - canonical Shorts URL;
   - video ID;
   - visible views;
   - visible age/date;
   - thumbnail URL when accessible.
5. Save exactly the first 30 unique Shorts.
6. Verify every item is a Short.

### Transcript enrichment

For each of the 55 video IDs use the deterministic transcript engine first. Record:

- language;
- complete/partial/missing;
- timestamped transcript where available;
- source and retrieval time.

Do not use speech recognition unless captions/transcripts are unavailable and the task explicitly permits media processing.

## Skool mission

Prerequisite: owner is already logged into Skool in the browser session.

1. Open `https://www.skool.com/obf`.
2. Verify group name is Online Business Friends and owner/teacher identity matches Dave Nick.
3. Confirm classroom/training navigation is visible.
4. Inventory every visible course/module/lesson in order.
5. For each lesson capture visible:
   - module;
   - lesson title;
   - URL/ID;
   - resource names/links;
   - lesson text/notes;
   - transcript/captions if already exposed by platform/player;
   - process steps;
   - decisions/gates;
   - metrics/benchmarks;
   - templates/checklists;
   - monetization/offer guidance.
6. Do not buy, enroll in paid upgrades, comment, react, message, download restricted media, or change account state.
7. Do not persist cookies, tokens, localStorage, authorization headers, or personal account identifiers into artifacts or routine cache.

## Scraper fallback

For public YouTube pages only:

1. deterministic YouTube/transcript route;
2. HyperAgent DOM extraction;
3. Scrapling adaptive public-page extraction;
4. Firecrawl only when explicitly authorized and runtime networking supports it.

Any temporary Firecrawl credential is runtime-only and must never be stored in Git or output files.

## Output

`benchmarks/one-person-business/manifest.json`

```json
{
  "creator": "One Person Business",
  "channel_id": "UC5sGRuouCtllvIEL1q8ay0Q",
  "long_form_requested": 25,
  "shorts_requested": 30,
  "long_form_collected": 0,
  "shorts_collected": 0,
  "transcripts_complete": 0,
  "transcripts_partial": 0,
  "transcripts_missing": 0,
  "skool_access": "not_checked|authenticated|blocked",
  "coverage_start": null,
  "coverage_end": null,
  "retrieved_at": null,
  "sources": [],
  "blockers": []
}
```

Then generate:

- `long-form.json`
- `shorts.json`
- `creator-playbook.md`
- `title-patterns.json`
- `thumbnail-patterns.json`
- `hook-patterns.json`
- `offer-cta-map.json`
- `cadence.json`
- `course/inventory.json`
- `course/process-map.json`
- `course/course-playbook.md`
- `workflow-diff-vs-hermes.md`

## Proof gate

Do not call the acquisition complete until:

- `long_form_collected == 25`;
- `shorts_collected == 30`;
- all 55 IDs are unique inside their format lists;
- long-form list contains zero Shorts URLs;
- Shorts list contains only Shorts URLs;
- every record has title + canonical URL + ID;
- transcript coverage totals equal 55;
- Skool status states authenticated or the exact access blocker;
- artifacts contain no credentials/session material.
