# Engineering capability library

Load this file only when an engineering-phase trigger is present. Use one named capability at a time unless the task genuinely crosses phases.

## ask-matt
Router. Determine the smallest workflow that fits the situation. Prefer an existing capability over inventing a new process. Return the chosen capability, why it fits, and the next action.

## grill-with-docs
Interrogate the plan while updating domain terminology and durable decisions. Resolve ambiguous nouns, actors, states, invariants, boundaries, failure modes, and ownership before implementation. Persist only resolved knowledge and ADR-worthy decisions.

## triage
Move work through explicit states. Establish severity, user impact, reproducibility, owner, blockers, evidence, and next state. Do not mark resolved without proof.

## improve-codebase-architecture
Brownfield architecture audit. Record baseline, map modules/data/deployment, identify deepening opportunities and blast radius, rank by leverage/risk, then select one bounded improvement. Never rewrite solely for taste.

## setup-matt-pocock-skills
Repository bootstrap concept. Ensure issue-tracker conventions, domain/context docs, decision records, review boundaries, and skill-routing expectations exist. In Hermes, adapt to ICM conventions instead of creating duplicate governance.

## to-spec
Convert the current conversation and evidence into an implementable specification. Include outcome, non-goals, constraints, interfaces, acceptance criteria, failure behavior, observability, rollback, and proof requirements. Separate facts from assumptions.

## to-tickets
Break a spec into tracer-bullet tickets that each produce a verifiable vertical slice. Declare dependencies/blocking edges. Prefer independently testable outcomes over layer-by-layer chores.

## implement
Implement only from an accepted spec or ticket. Inspect affected code first, make the minimum isolated change, use TDD at meaningful seams, run relevant checks, then hand off to an independent review pass before commit/merge.

## wayfinder
For work too large for one agent session. Build a shared map of unresolved decisions and dependency edges. Resolve the highest-leverage blocking decision first. Do not explode the plan into implementation tickets until the route is sufficiently known.

## prototype
Build a disposable artifact only to answer a specific uncertainty. Define the question and success signal first. Keep it isolated from production, cheap to discard, and do not let prototype shortcuts silently become architecture.

## diagnosing-bugs
Use a disciplined loop: reproduce -> create a failing signal -> minimize -> form hypotheses -> instrument -> isolate cause -> fix -> regression-test. Do not patch symptoms without a causal explanation when the bug is material.

## research
Investigate uncertain technical questions using high-trust primary sources when available. Capture claims, evidence, dates, uncertainty, and implications. Research should terminate in a decision or narrowed uncertainty, not an unbounded reading task.

## tdd
Red -> green -> refactor on one vertical slice. The failing test must represent user/system behavior or a meaningful interface seam. Avoid tests that only mirror implementation details.

## domain-modeling
Sharpen the project's language and invariants. Identify entities, value objects, states, transitions, ownership, boundaries, and forbidden states. Stress-test terms with concrete scenarios and update durable context/ADRs when terminology changes.

## codebase-design
Design deep modules with small stable interfaces and hidden implementation detail. Prefer clear seams, low coupling, high cohesion, and interface-level tests. Design twice when the first option feels inevitable.

## code-review
Run two independent axes: (1) standards/architecture/code smells and (2) faithfulness to the originating spec/ticket. Review the actual diff from a fixed base. Findings need severity, evidence, and actionable remediation. The builder does not self-approve.

## resolving-merge-conflicts
Resolve conflicts hunk by hunk from intent, not by blanket ours/theirs. Trace each side to its source change/spec, preserve compatible intent, run targeted checks, and complete the merge/rebase with an auditable resolution. Do not abort merely because conflicts are difficult.

## wizard
Generate an interactive human-execution wizard only for actions the agent cannot or should not perform directly: credentials, privileged provisioning, irreversible provider changes, manual approvals, physical steps, or unfamiliar dashboards. Each step must say what to inspect, what to change, expected evidence, and how to back out.

## Recommended chains

### Greenfield feature
`grill-with-docs -> domain-modeling -> research (only unknowns) -> to-spec -> to-tickets -> prototype (only risky assumptions) -> implement -> tdd -> code-review`

### Brownfield rescue
`improve-codebase-architecture -> diagnosing-bugs or domain-modeling -> to-spec -> to-tickets -> implement -> tdd -> code-review`

### Multi-session transformation
`wayfinder -> resolve decision tickets -> to-spec -> to-tickets -> implement -> code-review`
