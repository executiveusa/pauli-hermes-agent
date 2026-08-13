# Hermes Governor Heartbeat

The heartbeat is an operating discipline, not a chatty status bot.

## Every governor cycle
1. Read active Kanban work and current ICM stage states.
2. Check for blocked, stale, crashed, over-budget, or approval-waiting work.
3. Reclaim/retry only within the task's retry budget.
4. Confirm no worker crossed its authority boundary.
5. Confirm paid/public actions have receipts and required approvals.
6. Update durable project state with evidence and next action.
7. Notify the owner only for exceptions, meaningful milestones, taste review, or true decision gates.

## Long-running worker heartbeat
Workers must call `kanban_heartbeat` every few minutes during long operations and at least hourly for jobs that may exceed an hour. Heartbeats should state observable progress, not vague activity.

Good: `4/8 source files indexed; transcript stage 62% complete; no paid API calls yet.`
Bad: `Still working.`

## Escalate immediately when
- required credential/permission is unavailable
- target account/client identity is ambiguous
- source ownership or consent is unclear
- event/public facts conflict
- an irreversible/public action lacks approval
- budget threshold would be exceeded
- repeated paid edit attempts exceed policy
- security/privacy risk appears
- worker cannot verify its own mechanical result

## Do not escalate for
- normal read-only discovery
- routine deterministic transforms
- worker-to-worker handoffs that meet contract
- expected retries within the retry budget
- status information the dashboard/ledger already records

## Owner digest
When there is something worth surfacing, report:
- What outcome moved?
- What is verified?
- What failed or is blocked?
- What did it cost?
- What decision, if any, is needed from the owner?
- What happens next automatically?

## Anti-bottleneck rule
Never ask the owner to manually perform routing, copying, status tracking, or tool selection that Hermes can safely perform. Ask for judgment, approval, access, or missing facts only when those are genuinely human responsibilities.
