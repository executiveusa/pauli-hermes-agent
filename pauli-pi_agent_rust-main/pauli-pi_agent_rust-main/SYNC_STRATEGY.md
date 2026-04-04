# Sync Strategy

## Source of Truth
- Primary: JSONL
- Rationale: Matches legacy behavior and preserves human/Git-friendly sessions. SQLite is a derived index for fast lookup and search.

## Sync Triggers
- On session save: `AgentSession::persist_new_messages` updates the SQLite index
- Manual rebuild: `SessionIndex::reindex_all()` (CLI command planned)
- Timer/throttle: no background timer; indexing is incremental on session save

## Versioning
- DB marker: `meta.last_sync_epoch_ms`, per-session `sessions.last_mtime_ms` + `last_size_bytes`
- JSONL marker: file `mtime` + `size` (filesystem metadata)

## Concurrency
- Lock file path: `~/.pi/agent/session-index.lock`
- Busy timeout: 5 seconds (SQLite busy timeout)

## Failure Handling
- DB locked: retry with busy timeout, then surface a clear error and keep JSONL authoritative
- JSONL parse error: skip indexing for that file, report error, allow manual repair
- Git commit error: not applicable (no automatic git operations)

## Planned CLI Helpers
- `pi sessions reindex` — rebuild SQLite index from JSONL
- `pi sessions export-jsonl` — dump index metadata for inspection
