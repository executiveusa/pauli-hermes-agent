# OpusClip Integration Contract

## Authority
Official references:
- OpusClip API overview: https://help.opus.pro/api-reference/overview
- Opus skill repo: https://github.com/opus-pro/opus-skills
- Hosted MCP: use Opus OAuth/MCP when available; do not paste API keys into prompts or repositories.

## Purpose
OpusClip is the current cloud production/distribution engine for short-form video. It is not the story brain and it is not the owner of canonical project knowledge.

Hermes owns:
- client context
- story map
- series plan
- edit brief
- cost ledger
- approval state
- evidence
- live verification

Opus owns:
- provider processing jobs
- candidate clips
- server-side editing/rendering
- HD export
- connected social posting/scheduling

## Tool priority
1. Opus MCP/API for project/transcript/list/get/export/schedule operations.
2. EditingScript round-trip for precise supported edits.
3. Structured browser automation for editor-only controls.
4. Pixel/mouse control only as a last resort.

## Required provider operations
Hermes should be able to:
- query API usage/caps
- list projects
- create one project per source video where practical
- retrieve source transcript with timings
- list/get candidate clips
- duplicate before destructive/substantive edits when supported
- fetch/apply EditingScript for precise edits
- inspect render status
- export HD MP4
- list connected social accounts
- generate proposed social copy
- schedule/publish only after approval

## Cost policy
Opus API and web pricing can diverge. Treat paid operations as metered side effects.

Before a new processing job:
1. inspect source duration
2. query current usage/cap when available
3. estimate credits/cost
4. write ledger entry
5. proceed only within the configured threshold

After paid work:
1. query usage again where possible
2. calculate actual delta
3. attach project/clip/job IDs
4. record outputs and failures

Never submit the same long source repeatedly just to make separate reels if one project/transcript can serve the series.

## Edit policy
- Read-only inspection first.
- Duplicate selected clip before substantive edits when possible.
- Keep source media immutable.
- More than three paid re-render/edit operations on the same clip in one run requires escalation.
- Never run paid edits in a loop.
- Prefer local ffmpeg operations for safe mechanical tasks when they preserve required captions/branding and reduce provider cost.

## Social posting policy
- List/verify the target account before scheduling.
- `APPROVED` is a hard precondition to schedule/publish.
- Store platform/account, schedule/post ID, requested time, actual time, caption hash, media hash/pointer, and result.
- After publication, independently verify the live post.

## Suggested Hermes MCP configuration
Use Hermes' remote MCP/OAuth support. Exact Opus endpoint/auth should be confirmed from current Opus documentation at install time; never hardcode a bearer key into the repo.

Example shape:
```yaml
mcp_servers:
  opusclip:
    url: "<CURRENT_OPUS_MCP_ENDPOINT>"
    enabled: true
    timeout: 180
    supports_parallel_tool_calls: false
    tools:
      include:
        - "*project*"
        - "*transcript*"
        - "*clip*"
        - "*export*"
        - "*post*"
        - "*usage*"
      resources: false
      prompts: false
```

Run `hermes mcp test opusclip` and inspect the discovered tool names before enabling production write tools.

## Secret handling
Store credentials in Hermes' owner-controlled secret/env mechanism (`~/.hermes/.env` or OAuth token storage). Never commit secrets, echo them to logs, or place them in task bodies/receipts.
