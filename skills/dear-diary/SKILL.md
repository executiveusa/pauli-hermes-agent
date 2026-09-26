---
name: dear-diary
description: Record durable operational decisions, facts, changes, lessons, blockers, and outcomes into Terabithia so the five-agent fleet stays synchronized. Never publish private personal memory.
---

# Dear Diary

Use this skill when a conversation, mission, review, or project produces information that should remain true after the current session ends.

## Authority boundary

Dear Diary is a **Hermes skill**. It is not a sixth agent and it is not an independent source of truth.

Hermes proposes a structured entry. Terabithia validates and persists it. All fleet members may read the resulting shared operational knowledge according to their permissions.

Never write private Pi/personal memory into the shared ledger. If the information is private, keep it in the personal memory domain and do not call this skill.

## Record when

Record only information with durable operational value:

- a decision was made;
- a fact became verified;
- a system/project state materially changed;
- a lesson should alter future behavior;
- a blocker affects future work;
- a meaningful outcome completed;
- a previous decision has been replaced.

Do not record casual conversation, speculation, duplicate summaries, secrets, credentials, access tokens, or transient tool output.

## Required structure

Every entry must have:

- `type`: `decision | fact | change | lesson | blocker | outcome`
- `summary`: concise current truth
- `sensitivity`: normally `shared`; use `restricted` when appropriate

Add whenever available:

- `reason`
- `project`
- `affected_agents`
- `context_refs`
- `evidence_refs`
- `supersedes`

If replacing an earlier decision, set `supersedes` to the earlier decision ID instead of creating two apparently-current truths.

## Invocation

Use the repository script so credentials stay in environment variables:

```bash
python scripts/dear_diary.py \
  --type decision \
  --summary "Terabithia is the canonical dispatcher for the five-agent fleet" \
  --project terabithia \
  --agent hermes --agent pi --agent bars --agent jarvis --agent lightning \
  --evidence-ref "github://executiveusa/terabithia"
```

The runtime requires `TERABITHIA_URL` and `TERABITHIA_API_KEY` in the process environment. Never echo, log, commit, or place the API key in a prompt.

## Privacy rule

`--sensitivity private` is intentionally rejected. Private memory must remain in the private/personal domain until an explicit, minimum-necessary shared statement has been produced.

## Completion rule

A Dear Diary write is complete only when Terabithia returns a `decision_id`. Keep that ID when a later entry supersedes the decision.
