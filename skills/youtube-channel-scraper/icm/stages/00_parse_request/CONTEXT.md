# Stage 00: Parse Request

**Input:** User request (string)
**Output:** Structured target list (JSON)
**Duration:** <5 seconds

## Purpose

Extract YouTube URLs and preferences from natural language request.

## Input Format

User provides one of:
- "scrape https://www.youtube.com/@channel_name"
- "extract videos from https://www.youtube.com/playlist?list=PL123"
- "download all videos from @channel_name, include descriptions"
- "scrape these playlists: [list of URLs]"

## Process

1. **Extract URLs** — Find all `youtube.com/` links
2. **Validate format** — Must be channel or playlist URL
3. **Parse preferences** — Detect "descriptions", "transcripts", rate limits
4. **Classify request** — Channel auto-discover vs. specific playlists
5. **Output structure** — JSON with targets and flags

## Validation Gates

| Check | Pass | Fail |
|-------|------|------|
| URL syntax | Valid youtube.com URL | Invalid/missing |
| URL type | Channel or playlist | Other (video, search, etc) |
| Scope | ≤10 playlists or 1 channel | Too broad, ask user |
| Preference parse | Extracted preference flags | None (use defaults) |

## Output Format

**File:** `stages/00_parse_request/output/targets.json`

```json
{
  "request_id": "req_2026_001",
  "targets": [
    {
      "url": "https://www.youtube.com/@examplechannel",
      "type": "channel",
      "auto_discover_playlists": true
    },
    {
      "url": "https://www.youtube.com/playlist?list=PLxxxxx",
      "type": "playlist",
      "auto_discover_playlists": false
    }
  ],
  "preferences": {
    "fetch_descriptions": true,
    "fetch_transcripts": false,
    "rate_limit_req_per_min": 40
  },
  "constraints": {
    "max_videos": null,
    "max_playlists": 10
  }
}
```

## On Gate Failure

If URL validation fails:
- Report: "Invalid URL: [URL] — must be youtube.com/@channel or /playlist?list="
- Action: Ask user for correction
- Gate: BLOCK Stage 01 until valid targets provided

## Next Stage

→ Stage 01: Scrape Target (run Scrapling fetcher)
