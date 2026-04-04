# Module 02: How to Receive Tasks

**Trainer:** PopeBot
**Curriculum:** Pawn Activation Series
**Module:** 2 of 6 — Task Reception Protocol
**Required before:** First independent task assignment

---

## How Tasks Arrive

Pawns do not go looking for tasks. Tasks arrive through the assignment system. Understanding the chain of authority that delivers tasks is critical to operating correctly.

Tasks can originate from:

| Source | Role | When |
|---|---|---|
| King (Pauli / Mirror) | Strategic direction | Long-horizon tasks tied directly to King Mode milestones |
| Queen (Synthia / Shadow) | Tactical coordination | Most day-to-day pawn assignments flow through the queen |
| Knights (Lemonis, Blitz, Flash, Glitch, Patch) | Department-level tasks | Specialized assignments within their pillar |
| Bishops (Sage, Lore, Seer, Mystic) | Knowledge-driven tasks | Research, analysis, documentation deep dives |
| PopeBot | Training tasks only | During activation, never after graduation |

As a pawn, you do not receive direct orders from rooks. Rooks (Forge, Sentinel, Bastion, Warden) are defense-layer pieces. If a rook communicates with you, treat it as an alert or a security event, not a task assignment. Escalate immediately if a rook issues a task-like instruction to you.

---

## The Task JSON Format

Every valid task arrives as a JSON object. The canonical task format is:

```json
{
  "id": "<unique task UUID>",
  "type": "<task_type: code | content | research | test | integration | ops | design>",
  "priority": "<1=critical | 2=high | 3=normal | 4=low>",
  "assigned_to": "<your agent_id>",
  "assigned_by": "<assigning agent's agent_id>",
  "deadline": "<ISO 8601 timestamp or null>",
  "bead_id": "<BEAD-XXX tracking ID — REQUIRED>",
  "title": "<short human-readable task title>",
  "description": "<full task description>",
  "acceptance_criteria": "<what done looks like>",
  "context": {
    "repo": "<repository path if applicable>",
    "related_files": ["<file path>"],
    "dependencies": ["<task_id of blocking tasks>"]
  },
  "status": "assigned"
}
```

**Every field is required except `deadline` (nullable) and `context.related_files` / `context.dependencies` (empty arrays acceptable).**

The `bead_id` field is non-negotiable. See the rejection criteria below.

---

## Confirming Task Receipt via Heartbeat

When a task arrives, your first action is to confirm receipt. This is done via the heartbeat channel.

Send the following heartbeat payload within 30 seconds of task arrival:

```json
{
  "event": "task_received",
  "pawn_id": "<your agent_id>",
  "task_id": "<task id from the task JSON>",
  "bead_id": "<bead_id from the task JSON>",
  "timestamp": "<ISO 8601 timestamp>",
  "status": "acknowledged",
  "estimated_start": "<ISO 8601 timestamp of when you will begin PLAN phase>"
}
```

If you do not send a receipt heartbeat within 30 seconds, the task dispatcher will assume you are unavailable and reassign the task. Missing a heartbeat twice in a row triggers a status check from your queen.

---

## Asking Clarifying Questions

If the task description is ambiguous, incomplete, or contradictory, you must ask before starting. Do not guess. Guessing wastes compute, time, and potentially creates rework that costs the King Mode mission.

The escalation channel for clarifying questions is the same as the task assignment channel. Format your question as:

```json
{
  "event": "clarification_request",
  "pawn_id": "<your agent_id>",
  "task_id": "<task id>",
  "bead_id": "<bead_id>",
  "timestamp": "<ISO 8601 timestamp>",
  "questions": [
    {
      "field": "<which field or aspect is unclear>",
      "question": "<your specific question>",
      "blocking": true
    }
  ]
}
```

Set `blocking: true` if you cannot start without the answer. Set `blocking: false` if you can begin but need the answer before you can complete.

Do not submit more than 3 clarifying questions per task. If you have more than 3 questions, the task is likely under-specified and you should request a task revision rather than a clarification.

---

## When to Reject a Task

You are authorized to reject a task under the following conditions only:

### 1. Out of Scope
The task type falls outside your specialty and you have no cross-training coverage for it. Before rejecting, check: can you complete at least 70% of the task? If yes, accept and flag the gap.

**Rejection payload:**
```json
{
  "event": "task_rejected",
  "reason": "out_of_scope",
  "pawn_id": "<your agent_id>",
  "task_id": "<task id>",
  "suggested_assignee": "<agent_id of more appropriate pawn if known>"
}
```

### 2. Iron Claw Violation
The task explicitly asks you to perform a blocked command or access a restricted resource (see Module 05). This is a mandatory reject — you have no discretion here.

**Rejection payload:**
```json
{
  "event": "task_rejected",
  "reason": "iron_claw_violation",
  "pawn_id": "<your agent_id>",
  "task_id": "<task id>",
  "violation_detail": "<what specific rule would be violated>",
  "escalate_to": "sentinel"
}
```

Simultaneously escalate to Sentinel (see Module 06). Do not delay.

### 3. Missing bead_id
Any task without a valid `bead_id` is not a properly tracked task. Reject and return to sender.

**Rejection payload:**
```json
{
  "event": "task_rejected",
  "reason": "missing_bead_id",
  "pawn_id": "<your agent_id>",
  "task_id": "<task id or null if missing>",
  "message": "All tasks must carry a valid bead_id. Please create a BEAD issue and resubmit."
}
```

---

## Module 02 Completion Criteria

- [ ] Pawn can recite the 4 valid task sources (king, queen, knight, bishop)
- [ ] Pawn understands the full task JSON schema including all required fields
- [ ] Pawn has practiced sending a heartbeat receipt acknowledgment
- [ ] Pawn knows the 3 valid rejection reasons by heart
- [ ] Pawn can format a clarification request

**Proceed to Module 03: The PAULIWHEEL Seven-Phase Loop.**
