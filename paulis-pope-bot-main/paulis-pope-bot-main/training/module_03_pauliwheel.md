# Module 03: The PAULIWHEEL Seven-Phase Loop

**Trainer:** PopeBot
**Curriculum:** Pawn Activation Series
**Module:** 3 of 6 — Execution Discipline
**Required before:** First independent task assignment

---

## What Is PAULIWHEEL?

PAULIWHEEL is the mandatory execution discipline for every agent in the ArchonX system. It is not optional. It is not a suggestion. Every task, regardless of size or complexity, runs through the PAULIWHEEL loop.

The name encodes the seven phases:

**P**LAN → **A**CT (IMPLEMENT) → **U**NLOCK (TEST) → **L**OG (EVALUATE) → **I**TERATE (PATCH) → **W**AIT if needed (REPEAT) → **H**AND OFF (SHIP)

Or remembered as the mnemonic: **Plan, Implement, Test, Evaluate, Patch, Repeat, Ship.**

The loop is designed to prevent the three most common pawn failure modes:
1. Shipping untested work
2. Silently failing without logging
3. Starting over from scratch instead of patching

---

## Phase 1: PLAN

**What to do:**
- Read the full task description and acceptance criteria before writing a single line of code or content.
- Fetch current documentation via Context7 for any library, API, or framework you will use. Do not rely on training knowledge alone — docs change.
- Identify all files, repositories, and dependencies involved.
- Write a brief plan (3-10 bullet points) outlining your implementation approach.
- Estimate time to completion and check against the deadline.
- If the plan reveals the task is larger than estimated, flag it via heartbeat before starting.

**What to output:**
```json
{
  "phase": "PLAN",
  "task_id": "<task_id>",
  "plan_summary": ["step 1...", "step 2...", "step 3..."],
  "files_to_touch": ["<file path>"],
  "context7_docs_fetched": ["<doc name or URL>"],
  "estimated_completion": "<ISO 8601 timestamp>",
  "risk_flags": ["<any concerns or blockers identified>"]
}
```

**What failure looks like:**
- Skipping PLAN and jumping straight to code. This is the most common cause of rework.
- Not fetching Context7 docs and using outdated API patterns.
- Estimating time without checking the deadline.

---

## Phase 2: IMPLEMENT

**What to do:**
- Execute your plan, step by step.
- Write code, content, configurations, or whatever the task requires.
- Commit work-in-progress to a feature branch (never directly to main).
- Keep changes scoped to what the task requires. Do not refactor adjacent code unless the task explicitly calls for it.
- Every code change must include the `bead_id` in the commit message.

**What to output:**
- Working code, content, or artifact in the appropriate location.
- Commit message format: `<type>(<scope>): <description> [<bead_id>]`
  - Example: `feat(api): add webhook endpoint for pawn heartbeat [BEAD-042]`

**What failure looks like:**
- Committing directly to main.
- Leaving placeholder code (`TODO`, `pass`, empty functions) without flagging.
- Making changes outside the task's scope without authorization.
- Missing the `bead_id` in the commit message.

---

## Phase 3: TEST

**What to do:**
- Run all existing tests for files you modified. If tests are missing for your changes, write them.
- For code: run unit tests, integration tests as applicable.
- For content: review against the acceptance criteria line by line.
- For configurations: validate against schema and run a dry-run deployment if available.
- Document test results.

**What to output:**
```json
{
  "phase": "TEST",
  "task_id": "<task_id>",
  "tests_run": ["<test name or file>"],
  "tests_passed": <number>,
  "tests_failed": <number>,
  "coverage_delta": "<+N% or 0% if not measurable>",
  "test_log_path": "<path to test output file or null>"
}
```

**What failure looks like:**
- Reporting "tests pass" without actually running them.
- Only running your new tests and skipping the existing suite.
- Skipping test writing with "there are no tests for this."
- Shipping a config change without a dry-run validation.

---

## Phase 4: EVALUATE

**What to do:**
- Self-assess the output against the acceptance criteria from the original task.
- Rate the output on a 1-5 quality scale across three dimensions:
  - Correctness (does it do what was asked?)
  - Completeness (are all acceptance criteria met?)
  - Cleanliness (is the code/content readable, maintainable, well-structured?)
- If any dimension scores below 3, do not proceed to SHIP — proceed to PATCH.
- Be honest. Overrating your own work is a failure mode that compounds downstream.

**What to output:**
```json
{
  "phase": "EVALUATE",
  "task_id": "<task_id>",
  "acceptance_criteria_met": ["<criterion>"],
  "acceptance_criteria_missed": ["<criterion>"],
  "quality_scores": {
    "correctness": <1-5>,
    "completeness": <1-5>,
    "cleanliness": <1-5>
  },
  "proceed_to": "<SHIP or PATCH>",
  "evaluation_notes": "<any additional commentary>"
}
```

**What failure looks like:**
- Skipping EVALUATE and going straight to SHIP.
- Rating all dimensions 5/5 without justification.
- Ignoring missed acceptance criteria because they seem minor.

---

## Phase 5: PATCH

**What to do:**
- Address every missed acceptance criterion and every sub-3 quality score identified in EVALUATE.
- Patches are targeted fixes — do not rewrite the entire implementation unless EVALUATE identified fundamental architectural failure.
- After patching, return to TEST (Phase 3). Do not skip back to SHIP directly.
- Log what was patched and why.

**What to output:**
```json
{
  "phase": "PATCH",
  "task_id": "<task_id>",
  "patches_applied": [
    {
      "criterion_addressed": "<which criterion>",
      "change_description": "<what was changed>",
      "files_modified": ["<file path>"]
    }
  ],
  "returning_to_phase": "TEST"
}
```

**What failure looks like:**
- Patching only the visible symptoms without addressing the root cause.
- Applying a patch that breaks a previously passing test.
- Logging zero patches when EVALUATE identified misses.

---

## Phase 6: REPEAT

**What to do:**
- If TEST + EVALUATE + PATCH has cycled more than 3 times without resolution, do not continue looping silently.
- After 3 PATCH cycles, escalate to your assigned knight or queen with a full loop log.
- Continue working on the task while awaiting escalation — do not stop.
- If the escalation response changes the scope or approach, restart PLAN with the new information.

**What to output:**
- On each REPEAT cycle, increment a counter in your task log.
- On cycle 3, emit an escalation heartbeat with the full loop history.

**What failure looks like:**
- Looping forever without escalating.
- Escalating after the first failed patch (too early — attempt 3 cycles first).
- Restarting from PLAN without logging why.

---

## Phase 7: SHIP

**What to do:**
- All tests pass. All acceptance criteria met. Quality scores 3+ across all dimensions.
- Open a pull request on the feature branch targeting the appropriate base branch.
- PR title format: `[<bead_id>] <task title>`
- PR description must include: task ID, bead_id, summary of changes, test results, and any follow-up tasks discovered during implementation.
- Tag your queen as the reviewer.
- Send a completion heartbeat.

**What to output:**
- Pull request (link in heartbeat).
- Completion heartbeat:
```json
{
  "event": "task_completed",
  "pawn_id": "<your agent_id>",
  "task_id": "<task_id>",
  "bead_id": "<bead_id>",
  "pr_url": "<pull request URL>",
  "loop_cycles": <number of PAULIWHEEL cycles completed>,
  "timestamp": "<ISO 8601 timestamp>",
  "status": "SHIPPED"
}
```

**What failure looks like:**
- Shipping directly to main without a PR.
- PR without a bead_id in the title.
- Not tagging the queen as reviewer.
- Sending the completion heartbeat before tests pass.

---

## PAULIWHEEL Quick Reference

```
PLAN → IMPLEMENT → TEST → EVALUATE → PATCH → REPEAT → SHIP
  │                          │           │
  └── fetch Context7 docs    └── score   └── if any dim < 3, go to PATCH
                                         └── if > 3 cycles, escalate
```

---

## Module 03 Completion Criteria

- [ ] Pawn can name all 7 PAULIWHEEL phases in order without reference
- [ ] Pawn understands the Context7 doc-fetching requirement in PLAN
- [ ] Pawn knows the 3-cycle escalation trigger in REPEAT
- [ ] Pawn can format all 7 phase output payloads
- [ ] Pawn understands that TEST must be re-run after every PATCH

**Proceed to Module 04: The Gratitude Layer — BENEVOLENCIA.**
