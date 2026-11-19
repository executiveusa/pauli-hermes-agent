# Cosmos deterministic heartbeat — phase 2 (OFF)

Schedule proposal: `*/5 * * * *` (owner-adjustable). **No job is installed or enabled by this PR.** The runner uses Python stdlib only and never imports agent/model/deploy modules.

It requires three separately authenticated and timestamped snapshots: Paperclip tasks assigned to Cosmos, private agent-commons messages addressed to Cosmos, and Cosmos's own CodeDen handoff states. **A board message never grants work authority.** The check does not create tasks, call any model, dispatch any agent, send outbound messages, read credentials, or perform network access. An approved adapter must first export actual states. A snapshot is not proof of source authenticity merely because it contains `verified: true`; attest/validate that at the adapter boundary.

Each snapshot must be a JSON object with `source` (`paperclip`, `agent_commons` or `codeden`), `verified: true`, `observed_at` ISO UTC within ten minutes, and `items` array. Items may include `id`, `assignee`, `owner`, `to`, `status`, `unread`. Missing/stale/invalid source = `SOURCE_HOLD`, never `NO_WORK`.

Run dry-run first with config explicitly enabled **only in an isolated test copy**:
```sh
python scripts/cosmos_heartbeat.py --config examples/cosmos-heartbeat.disabled.json --dry-run
```
The checked-in config has `enabled=false`, so this returns `DISABLED`. For acceptance testing, create a temporary copy with `enabled=true` and test-only snapshots; this **does not** install a cron. Any production enablement and Paperclip/board credentials require separate approval and a verified trusted adapter.

Status reporting can optionally write `last_seen` plus one `instinct_line` to an existing local directory, never to a public endpoint. Never silently create a cron job. Future model invocation would require a separately reviewed authenticated mission dispatcher, NOT this script.
