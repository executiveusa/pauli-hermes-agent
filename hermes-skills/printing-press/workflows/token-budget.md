# Workflow: Token budget enforcement

## Goal
Reduce token burn by preferring local compact CLI paths.

## Rules
1. Prefer `*-pp-cli --json --compact` over browser/API exploration.
2. Prefer local sync + search/sql over repeated remote calls.
3. Prefer compound commands over multi-step API loops.
4. Cache expensive discovery in SQLite.
5. Return concise summaries by default; only return raw payloads when explicitly requested.

## Measurement
Track for each task:
- number of remote API/browser round trips avoided,
- approximate prompt/response token reduction,
- whether cache hit occurred.

## Exit criteria
- Task completed via compact CLI flow.
- Summary includes efficiency note and any cache usage.
