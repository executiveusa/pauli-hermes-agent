# ICM Mandatory Code Review Lifecycle

This is the canonical code-review completion gate for Hermes/Cosmos software work.

## Trigger

Run this process whenever a project, feature, rescue, migration, refactor, infrastructure-as-code change, or bug fix reaches a claimed completion boundary. If no code/config changed, record `NO_CODE_DIFF` and ask the owner whether they want to review the project artifacts instead.

A project MUST NOT be marked `DONE`, `SHIPPED`, or equivalent until the review receipt exists and the human review prompt has been surfaced.

## Review sequence

1. **Freeze the comparison point** — record repository, base ref/SHA, head ref/SHA, originating spec/ticket, changed files, and intended outcome.
2. **Run automated checks** — execute the smallest relevant test/lint/type/build/security checks. Record exact commands and results. A green build is evidence, not approval.
3. **Dispatch independent code review** — the builder may not be the final reviewer. Prefer two logically separate review passes, which may be delegated to fresh subagents:
   - **Standards review**: architecture, repo conventions, code smells, failure modes, security/reliability, unnecessary complexity, ownership/sovereignty, rollback.
   - **Spec review**: fidelity to the accepted spec/ticket, acceptance criteria, non-goals, target behavior, and proof requirements.
4. **Reconcile findings** — deduplicate findings, assign severity (`BLOCKER`, `HIGH`, `MEDIUM`, `LOW`, `NIT`), attach file/line or concrete evidence, and state remediation.
5. **Fix and re-review** — blockers/high findings must be fixed or explicitly accepted by the owner. The same builder may fix findings but must not approve its own fixes; re-dispatch a fresh reviewer when material code changed.
6. **Proof gate** — verify the actual target where possible. Never infer production correctness from code, CI, or deployment intent alone.
7. **Human code-review gate** — Hermes MUST ask the owner to review the code before closing the project. Use a direct prompt such as: **“The independent code review is complete. Do you want to review the diff/code with me before I close or ship this project?”**
8. **Close only with receipt** — record the owner response as `APPROVED`, `CHANGES_REQUESTED`, or `DECLINED_REVIEW`. `DECLINED_REVIEW` means the user chose not to inspect the diff; it does not waive automated proof or independent review.

## Review receipt

```text
PROJECT
REPOSITORY
BASE_SHA
HEAD_SHA
SPEC_OR_TICKET
CHANGED_FILES
AUTOMATED_CHECKS
STANDARDS_REVIEW
SPEC_REVIEW
FINDINGS
FIXES
RE_REVIEW
TARGET_PROOF
ROLLBACK
HUMAN_REVIEW_PROMPTED: yes/no
HUMAN_REVIEW_RESULT: APPROVED | CHANGES_REQUESTED | DECLINED_REVIEW | PENDING
STATUS: PASS | HOLD | BLOCKED
```

`HUMAN_REVIEW_PROMPTED` must be `yes` for project completion. `HUMAN_REVIEW_RESULT: PENDING` means Hermes must not silently convert the project to `DONE`.

## Scheduled / cron review mode

Code review may also run independently of a live project session.

A cron review should:

1. inspect repos/branches or project receipts changed since the last successful review;
2. select only candidates with new commits or unresolved review debt;
3. create a fixed base/head comparison;
4. dispatch fresh subagents for Standards and Spec review when subagents are available;
5. prohibit those subagents from merging, deploying, deleting, spending money, changing credentials, or approving their own fixes;
6. write a durable review receipt/report;
7. notify the owner only for new material findings or a project waiting on the mandatory human review gate;
8. do nothing when no material code change exists.

Scheduled reviewers are advisory/verifying agents. They never self-merge fixes merely because a cron run found a problem.

## Subagent packet

Every dispatched reviewer receives only:

- repo + fixed base/head SHAs;
- originating spec/ticket or explicit outcome;
- changed-file list/diff;
- repo rules (`AGENTS.md`, `CLAUDE.md`, relevant `CONTEXT.md`/ADRs);
- review axis (`standards` or `spec`);
- explicit non-authority to merge/deploy/approve its own work.

Reviewer output must be structured findings with severity, evidence, and remediation. A reviewer that cannot inspect the actual diff returns `NOT_RUN`, never `PASS`.

## Failure policy

- Missing spec on material work -> `HOLD` until intended behavior is reconstructed or owner accepts the gap.
- Missing diff/base -> `BLOCKED`.
- Failed automated check -> `HOLD` unless proven unrelated and explicitly accepted.
- Blocker/high finding -> `HOLD`.
- No independent reviewer available -> `HOLD` for material releases; small reversible work may proceed only under the governing authority policy.
- No human prompt at project end -> completion contract violated; reopen the project gate.

## Relationship to Gauntlet

Code review is mandatory for code changes. Gauntlet is an additional adversarial quality loop and does not replace this review gate. Use both for material releases when useful.
