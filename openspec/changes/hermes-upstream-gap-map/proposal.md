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

## Addendum — Bot Mode + Cloud (2026-08-22)

Triggered by: upstream shipped "Hermes Cloud" (Nous Portal hosting) and "Bot
Mode" (named multi-bot roster, `@mention` routing, shared collab rooms) in
the v0.20.0–v0.20.3 patch train (Aug 3–16, 2026). User wants both, with an
explicit constraint: **additive only — nothing already built in this fork
may be removed or overwritten.**

### Source truth for this addendum

- Upstream tags inspected: `v2026.8.3` (0.20.0), `v2026.8.13`, `v2026.8.16`,
  `v2026.8.16.2` (the desktop-default Bot Mode release).
- Method: `git ls-tree` / `git show` against a read-only `upstream` remote
  added to this checkout — no upstream code executed or merged, no local
  files touched by the recon itself.
- **No merge-base exists** between this fork's `main` and any upstream tag
  back to `v2026.3.12` (five months of upstream history). This fork was not
  produced by an ongoing `git merge`/`git pull` lineage from upstream — it's
  an independent import. That is the real explanation for why
  `scripts/upstream_sync.py` has recorded zero applied commits across 54
  nightly runs (see PR #141 discussion): `git cherry-pick` needs a shared
  ancestor to apply cleanly, and there isn't one. **Cherry-pick replay is
  not a viable mechanism for this port, at all, independent of the
  protected-paths question.** Vendoring (copy specific files/dirs at a
  pinned tag, adapt by hand, PR for review) is the only realistic path.

### Capability matrix additions

| Capability | Pauli fork now | Current upstream | Decision | Rationale / next action |
|---|---|---|---|---|
| Desktop plugin-host (`apps/desktop/src/contrib/plugin*.ts`, `plugins.ts`, `plugins-store.ts`, `src/plugins/`) | **Absent entirely** — no `contrib/` dir, no `plugins/` dir under this fork's `apps/desktop/src` | Present; introduced between upstream `v2026.7.1` (absent) and `v2026.7.20` (13 files), grown to 30+ files by `v2026.8.3` | PORT, but **foundational and high-risk** | This is the real prerequisite, not Bot Mode itself. It is upstream's own newest subsystem — roughly 5 weeks old at the point Bot Mode shipped on top of it — so its own API is still likely to move. Porting it means adopting upstream's plugin-loading contract into this fork's desktop app (currently `hermes` v0.15.1, same lineage/directory shape as upstream's `apps/desktop`, just older — not a divergent rebuild, which is the one genuinely good sign here). |
| Bot Mode plugin (`apps/desktop/src/plugins/hermes-bots/`) | Absent (blocked on the row above) | `plugin.js` (7,199 lines) + LICENSE + 40 dedicated test files (roster, `@mention` completions, mention-handoff, group chat, routines, model-inherit, soul-protocol-backfill, etc.) | PORT, **blocked** | Cannot be vendored standalone — it is written against the plugin-host contract above. Do not attempt to extract just the UI/roster logic and rewire it to this fork's current (pre-plugin) desktop shell; that is a rewrite wearing a port's clothes and will not track upstream's own fixes going forward. Port order must be: plugin-host first, validated and stable in this fork on its own, then `hermes-bots` on top. |
| Cloud request-attribution (`agent/portal_tags.py`) | **Present, but older** — 64 lines, has the basic `product=hermes-agent` / `client=hermes-client-v<version>` tag pair | Present, expanded — adds a `ContextVar`-based ambient conversation-id propagation system (~50 more lines) so background/subagent/batch call sites inherit the right tag context without threading a parameter through every call site | PORT, **low risk** | Single self-contained file, no plugin-host dependency, no directory this fork doesn't already have. Genuinely the cheapest safe win in this whole addendum — diff and adapt just this file, run its own tests, done. |
| Cloud hosting/provisioning (Nous Portal: two-click deploy, scale-to-zero, unified org billing) | Absent | Present, but **as a Nous-hosted SaaS control plane** (`portal.nousresearch.com`), not code in the OSS repo | PARK — not portable | There is nothing to vendor here beyond the attribution row above. "Getting" this feature for a self-hosted fork isn't a merge question — it's either (a) using Nous's hosted product directly for the bots you want in their cloud, which coexists with this fork rather than replacing anything in it, or (b) this fork's own existing edge-sovereign Cloudflare/Coolify deployment governor (commit `c2873ba`) is the actual self-hosted equivalent, already built, already Pauli-owned. Don't build a shadow control plane to chase parity with a product Nous sells. |

### Revised priority order for this addendum only

1. `agent/portal_tags.py` — diff, adapt, test. Ships alone, today, if wanted.
2. Plugin-host system — scoped as its own bounded slice, validated in
   isolation (this fork's existing desktop test suite must pass unchanged
   with the plugin-host present but zero plugins registered) before
   anything is layered on top.
3. `hermes-bots` plugin — only after (2) is merged and stable, vendored at
   the pinned tag, adapted, its own 40-test suite run against this fork's
   adapted plugin-host.
4. Nous Cloud itself — not a port; a product decision, not an engineering
   task, whenever it's needed.

### Non-goals (extends the section above)

- No plugin-host port that changes any existing desktop app behavior when
  zero plugins are registered — presence alone must be a no-op.
- No rewrite-disguised-as-a-port of Bot Mode against the old desktop shell.
- No self-hosted reimplementation of Nous Portal's provisioning/billing
  control plane.
- No reliance on `scripts/upstream_sync.py`'s commit-replay path for any
  part of this addendum — confirmed non-viable (no merge-base exists).

### Risk

LOW for this addendum as written — audit/specification only, zero code
changes. Risk moves to MEDIUM the moment the plugin-host slice (item 2
above) actually starts, purely because it's upstream's newest and least
battle-tested subsystem; that slice should get its own isolated PR with the
present-but-disabled regression check called out above before Bot Mode is
even attempted.
