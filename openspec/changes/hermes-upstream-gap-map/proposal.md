# Hermes Phase 02 — Upstream Capability Gap Map

## Objective

Compare the Pauli fork against current upstream Hermes at the implementation surface and classify each capability as `PORT`, `ADAPT`, `KEEP`, or `PARK` before runtime changes.

## Source truth

- Pauli fork baseline: `950aabef00059cbbf8a8a735da0bc215acd2d483`
- Upstream head observed: `4aa9f738cedbe8a69fbd08595d0fb67f812ce2d3`
- Merge base: `c79e3fd0baf41c0adda616b73153eeaa8a4b8231`
- Fork state at audit: 163 commits ahead / 11,864 behind.

## Capability matrix

| Capability | Pauli fork now | Current upstream | Decision | Rationale / next action |
|---|---|---|---|---|
| Core agent loop | Present | Present, heavily evolved | ADAPT | Do not replace wholesale; compare lifecycle/checkpoint changes around the existing Pauli orchestration contracts. |
| Tool registry | Present | Present/evolved | PORT selectively | Preserve Pauli tools; port registry fixes only when dependency-safe. |
| Terminal/process | Present | Present | KEEP + harden | Already first-class. Verify Windows PowerShell and SSH behavior before adding abstractions. |
| File tools | Present | Present | KEEP | Already first-class; no duplicate layer. |
| Browser classic actions | Present | Present | KEEP + selective PORT | navigate/snapshot/click/type/scroll/back/press/images/vision/console/CDP/dialog already exist. |
| `browser_exec` backend | Missing from Pauli toolset | Present | PORT | High-value newer browser-use path; inspect implementation/dependencies first. |
| Browser Camofox | Present in Pauli tree | Present | KEEP/reconcile | Already exists; compare bug/security fixes rather than re-add. |
| Computer use shim | Present, described macOS-only | Present, universal macOS/Windows/Linux | PORT | Direct answer to Windows requirement. Do not build a parallel Windows adapter before testing upstream universal CUA. |
| Computer-use backend package | Older/minimal: 6 files | Expanded: 9 files | PORT as bounded slice | Upstream adds `browser_route.py`, `doctor.py`, `permissions.py` and much larger backend/tool/schema implementations. |
| Computer-use session release | Missing from Pauli shim export | Present | PORT | Required lifecycle cleanup; verify call sites. |
| Computer-use doctor/diagnostics | Missing | Present | PORT | Needed for reliable cross-platform installation and support. |
| Computer-use permission layer | Missing | Present | PORT/ADAPT | Keep upstream platform permission checks; bind Pauli consequential-action approvals above them. |
| Computer-use browser routing | Missing | Present | PORT | Reuse instead of duplicating browser-vs-desktop decision logic. |
| Skills list/view/manage | Present | Present | KEEP + reconcile | Existing progressive skill system already first-class. |
| Pauli custom skills | Extensive | Not upstream | KEEP | Business workflows, ICM, Gauntlet, hosting deployer, research/monetization remain Pauli-owned. |
| Self-authored skill management | Present via `skill_manage` | Present/evolved | ADAPT | Verify mutation approval, provenance, tests, and skill promotion rules. |
| Subagent delegation | Present | Present | KEEP + reconcile | Existing `delegate_task`; compare isolation/budget improvements. |
| Kanban | Present | More capable upstream | PORT selectively | Upstream adds first-class review/request-changes and attachment tools. High priority. |
| Kanban review state | Missing | Present | PORT | Fits GRINIONS independent-review gate directly. |
| Kanban attachments | Missing | Present | PORT | Useful evidence transport; keep mission evidence canonical in Pauli's Place where applicable. |
| Cron | Present | Present | KEEP + fixes | Do not rebuild. Reconcile bug fixes and bounded execution behavior. |
| Watchers | Skill/plugin surface | Newer upstream capability | ADAPT | Useful for conditions/monitoring; must not become duplicate scheduler authority. |
| Messaging | Agent-callable `send_message` in Pauli core | Upstream deliberately removes model-callable send from core | ADAPT/SECURITY REVIEW | Upstream narrowed authority to delivery outside the model loop. Evaluate Pauli use cases before retaining broad agent-callable outbound messaging. |
| MCP | Dependency + architecture present | Present/evolved | KEEP + PORT fixes | Inspect CLI/gateway/provider changes before adding anything. |
| Plugins | Present | Present/evolved | KEEP + reconcile | Avoid copying entire plugin tree blindly. |
| Hooks | Present in architecture/gateway | More mature upstream | PORT selectively | Prioritize lifecycle/tool policy/observability hooks. |
| Memory | Present | Present with provider plugins | KEEP + boundary hardening | Pauli personal/business isolation takes precedence over upstream defaults. |
| Session search | Present | Present | KEEP | No duplicate memory search layer. |
| Graphify | Pauli core additions present | Not in current upstream core list | KEEP/POLICY REVIEW | Pauli-specific context graph capability; verify cost/duplication against memory providers. |
| VPS/Ralphy controls | Pauli core additions present | Not upstream core | KEEP | Directly supports GRINIONS coding-factory architecture. |
| Project desktop workspaces | Missing from Pauli toolsets | Present upstream | PARK then PORT | Valuable, but after execution/kernel reconciliation; GUI-only narrow waist must be preserved. |
| Desktop UI affordances | Missing from Pauli toolsets | Present upstream | PARK then PORT | Useful for future desktop app; not required for first Golden Path. |
| Coding posture | Missing explicit upstream posture block | Present upstream | PORT | High-value way to narrow tools automatically in code workspaces. |
| BFL FLUX 3 video | Missing | Present upstream | PARK | BARS owns media specialization; no need in Hermes kernel now. |
| Generic video generation evolution | Older | Newer upstream | PARK/route to BARS | Avoid turning Hermes into media worker. |
| Home Assistant | Present | Present | KEEP | Personal/device authority should eventually route through Pi/Jarvis policy. |
| Spotify | Present | Present | KEEP but not core | Low priority for Hermes business lane. |
| Discord/admin | Present | Present | KEEP | Existing. |
| Webhook safe toolset | Present | Present | KEEP | Important prompt-injection containment. |
| API server | Present | Present/evolved | KEEP + reconcile | Preserve Pauli API bridges; port security fixes deliberately. |
| ACP/editor integration | Present | Present/evolved | KEEP + reconcile | Useful coding surface. |
| TUI/dashboard | Present | Present/evolved | KEEP + reconcile | Do not rebuild chat UI. |
| Updater/rollback | Older/uncertain | Newer upstream | PORT after dependency audit | High priority but broad blast radius; do after bounded execution slices. |
| Observability plugin | Present architecture | Evolved upstream | PORT selectively | Map traces/cost/tool evidence into Pauli mission IDs. |
| Pauli orchestration charter | Present | Absent | KEEP | Core differentiated authority. |
| Pi routing | Present | Absent | KEEP | Personal/business boundary is Pauli-specific. |
| BARS routing | Present architecture | Absent | KEEP | Operator boundary is Pauli-specific. |
| Jarvis routing | Present architecture | Absent | KEEP | Presence boundary is Pauli-specific. |
| Lightning evaluation | Present architecture, incomplete runtime | Absent | KEEP/finish later | Must not be overwritten by generic upstream evaluators. |
| Pauli's Place mission/evidence | Present architecture | Absent | KEEP | Canonical release/mission evidence where applicable. |

## Priority order produced by the audit

1. Cross-platform upstream computer-use package.
2. Computer-use diagnostics + permission model + lifecycle cleanup.
3. `browser_exec` / newer browser routing.
4. Kanban review and attachment state.
5. Coding posture / tool narrowing.
6. Hooks + observability mapped to Pauli mission IDs.
7. Updater/rollback reconciliation.
8. Remaining upstream fixes by targeted diff, not branch sync.

## Critical correction

The earlier sprint assumption that upstream computer use was still macOS-only is stale. Current upstream source explicitly describes `computer_use` as universal across macOS, Windows, and Linux, and its package has materially expanded. Therefore the Windows plan changes from `BUILD FROM SCRATCH` to `PORT + VERIFY UPSTREAM UNIVERSAL CUA FIRST`.

## Non-goals

- no wholesale upstream merge
- no production deployment
- no secrets or credential changes
- no database changes
- no media-feature expansion in Hermes
- no weakening of Pauli authority boundaries

## Acceptance criteria

1. Every major kernel capability has an explicit disposition.
2. Windows computer use has a source-backed path.
3. First implementation slice is bounded and reversible.
4. Pauli-specific differentiators are explicitly protected.
5. Security-sensitive differences (especially outbound messaging) are called out for review instead of silently preserved or overwritten.

## Risk

LOW — audit/specification only.
