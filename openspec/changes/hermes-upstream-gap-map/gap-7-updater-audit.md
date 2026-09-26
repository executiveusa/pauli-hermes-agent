# Gap #7 audit — Updater/rollback reconciliation

Companion to `proposal.md`. Covers the 7th and final item in the gap-map:
"Older/uncertain | Newer upstream | PORT after dependency audit | High
priority but broad blast radius; do after bounded execution slices."

Source: `NousResearch/hermes-agent @ v2026.8.16.2` vs.
`executiveusa/pauli-hermes-agent @ origin/main` at audit time.

## Structural finding

Upstream extracted self-update logic into `hermes_cli/update_cmd.py`
(6,694 lines) plus `hermes_cli/update_lock.py` and
`hermes_cli/subcommands/update.py`. This fork has no equivalent split —
the same behavior lives inline in `hermes_cli/main.py`
(`cmd_update`, `_cmd_update_impl`, `_cmd_update_check`, `_cmd_update_pip`,
`_update_via_zip`, `_run_pre_update_backup`, `_resolve_update_branch`,
`_update_node_dependencies`, `_invalidate_update_cache`, and more). There
is no "missing file" to vendor here — this gap is reconciling two
independently-evolved implementations of the same live self-update
system, not porting a module.

Per the task's required approach, no wholesale port or restructuring was
attempted. The audit below is behavior-by-behavior, using the fork's
existing ~18 update-related test files as the map of what the fork
already covers, and upstream's ~48 update-related test files as the map
of what upstream has fixed since.

## What landed (1 change)

**`_atomic_replace_dir` for the ZIP-fallback directory replace.**
`hermes_cli/main.py::_update_via_zip` (the Windows fallback path used when
git file I/O is broken by AV/NTFS filter drivers) did
`shutil.rmtree(dst); shutil.copytree(src, dst)` for each top-level
directory it replaces. A copy that fails partway — plausible precisely
because this path only runs when file I/O is already flaky — leaves `dst`
deleted with nothing copied back (upstream issue #49145, observed as
`ui-tui/` vanishing and breaking the TUI). This is a byte-for-byte match
between the fork's code and the bug upstream's `_atomic_replace_dir`
fixes, and the fix is a single self-contained helper plus a two-line call
site change — no restructuring of the surrounding loop or control flow.

Ported the single-entry stage-then-swap form only (stage the copy under
`*.hermes-update-staging`, then `os.rename` it into place). Upstream later
generalized this into a two-phase design with rollback across the entire
~90-entry replace loop (issue #76104, `test_update_zip_two_phase.py`) —
that generalization changes the sequencing of the whole ZIP-update flow
and was judged out of scope for a bounded fix; see "Flagged, not landed"
below.

Files: `hermes_cli/main.py` (new `_atomic_replace_dir` helper + call-site
change), `tests/hermes_cli/test_update_zip_atomic_replace.py` (vendored
success-path test + one additional case for `dst` not existing yet).

## Already equivalent or comparable (no action)

The fork's inline implementation turns out to have independently grown
substantial hardening of its own — this is not a "behind and undefended"
implementation:

- **Zip-slip / symlink rejection during ZIP extraction** —
  `_update_via_zip` already validates every member path against the
  extraction root and rejects symlink members via the `external_attr`
  mode bits, matching upstream's `test_update_zip_symlink_reject.py`
  fix exactly (both fork and upstream files are near-identical).
- **Pre-pull SHA capture + rollback on bad code** — `_cmd_update_impl`
  captures `pre_pull_sha`, runs a post-pull syntax guard over
  critical-path files, and does `git reset --hard` back to `pre_pull_sha`
  on failure (`tests/hermes_cli/test_update_post_pull_syntax_guard.py`).
  Comparable in intent to upstream's syntax-guard coverage.
- **Pre-update snapshot** — `_run_pre_update_backup` / quick-snapshot
  before pulling, independent of and in addition to the git-level
  rollback.
- **Auto-stash of local changes** — `_stash_local_changes_if_needed` /
  `.gitignore`-aware autostash, with its own dedicated
  `test_update_autostash.py` (18 tests, comparable scope to upstream's
  file of the same name).
- **Diverged-history handling** — `git pull --ff-only`, falling back to
  `git reset --hard origin/<branch>` on divergence, with a stash
  preserved for recovery either way.
- **Windows concurrent-instance detection** — `_detect_concurrent_hermes_instances`
  and the exe-quarantine retry/reboot-defer path
  (`test_update_concurrent_quarantine.py`) exist in the fork with 18
  tests; upstream's equivalent file has grown different additional
  coverage in a different direction (see below), but the base mechanism
  is present and working on both sides.
- **Stale dashboard / hangup handling** — `test_update_stale_dashboard.py`,
  `test_update_hangup_protection.py` both have fork equivalents.

## Flagged, not landed — genuinely missing, too risky for a bounded PR

These are real gaps. Each would require touching the update flow's overall
sequencing or process-management surface for every user of `hermes
update`, not a localized additive check, so per the task's constraints
they are reported here rather than fixed.

1. **No cross-process update lock (`hermes_cli/update_lock.py` upstream
   equivalent).** Upstream added a marker-file-based mutual-exclusion lock
   because three surfaces can start an update of the same install tree —
   a terminal `hermes update`, the dashboard's Update button (which spawns
   the same command detached), and (upstream only) the desktop app's
   updater. Without a lock, two can run concurrently and rewrite source
   under a live interpreter (upstream's observed failure: an installer
   `git checkout` rewound the tree ~9k commits mid-`npm install` from a
   concurrently-running dashboard-spawned update).
   **This exposure is real in this fork**: `hermes_cli/web_server.py`
   spawns `hermes update` as a detached subprocess from the dashboard
   (`_spawn_hermes_action(["update"], "hermes-update")`), the same
   concurrency shape upstream's incident describes, and the fork has no
   lock of any kind guarding it. This is the single most consequential
   finding in this audit. Fixing it means adding a new lock module and
   wiring every update entry point (CLI, dashboard-spawn, and any future
   desktop path) through it — broad blast radius, exactly what the
   gap-map flagged this item for. **Recommend a dedicated follow-up
   task**, not a fold-in here.

2. **Windows gateway-pause-during-update (`_pause_windows_gateways_for_update`
   / `venv_launcher_ancestors` / leftover-holder nomination).** Upstream's
   `test_update_concurrent_quarantine.py` has grown considerably beyond
   the fork's version of the same file: ancestor-chain walking so a venv
   launcher isn't mistaken for a blocking holder, a "pause kill set" that
   covers the venv-guard abort set, logic to nominate leftover Python
   holders that are all gateway processes as safe to pause, and a hard
   refusal path when even one non-gateway holder remains. None of this
   exists in the fork today (`_pause_windows_gateways_for_update` is not
   defined anywhere in `hermes_cli/main.py`). This is deep,
   Windows-specific process-tree reasoning around what may be killed
   during an update — high value for Windows users but not a change that
   can be dropped in as an isolated check; it interacts with the existing
   concurrent-instance/quarantine path this fork already has. Flagging
   for a dedicated Windows-update-hardening pass.

3. **Orphaned Desktop backend reap (`_orphaned_desktop_backend_pids`).**
   Upstream's `test_update_orphan_backend_reap.py` covers a Desktop-app
   teardown race where the Electron/Tauri shell exits but its Python
   backend process survives, wedging the update's venv-holder guard.
   Requires classifying process trees by supervising-parent liveness.
   This fork's Desktop/GUI surface (if any) was not audited in depth
   here; recommend confirming whether this scenario is even reachable in
   this fork's packaging before deciding whether to port it.

4. **HEAD-moved gate after `git pull` (`test_update_head_moved_gate.py`).**
   Upstream added a check that compares pre-pull and post-pull HEAD SHA
   after a successful `git pull --ff-only`, so a detached/pinned checkout
   that reports "N new commits," runs the merge cleanly, but ends up on
   the same commit (because a later branch-switch step re-detaches to a
   raw SHA) fails loudly instead of printing "✓ Code updated!" against a
   stale tree. **Partially mitigated already**: this fork's
   `_cmd_update_impl` checks out the target branch (`git checkout
   <branch>`) *before* `git pull`, resolving any detached-HEAD state
   ahead of the pull rather than after — the specific re-detach sequence
   upstream's bug describes doesn't obviously apply to this fork's
   ordering. Not clearly a live bug here, so not ported; noted as a cheap
   belt-and-suspenders check a future pass could add (reuse the fork's
   existing `_capture_head_sha` helper, called once more post-pull).

5. **Cross-process update-marker-aware self-lock deferral
   (`test_update_self_lock.py`, `test_update_secret_import_lock.py`).**
   Upstream defers the dependency-sync step when the updater process has
   already imported a native venv extension (e.g. `cryptography`'s Rust
   bindings via a secrets backend) that would block `uv` from replacing
   the mapped `.pyd` on Windows. Tightly coupled to the update-lock
   module in finding #1 and to this fork's own secrets-backend import
   ordering, which was not audited in depth. Flagging as dependent on #1
   rather than independently actionable.

## Not investigated in depth (lower priority / narrower applicability)

Peeked at but not pursued further given the size of the remaining list and
the bounded scope of this task: `test_update_venv_health.py`,
`test_update_import_guard.py`, `test_update_modified_notice.py`,
`test_update_current_node_repair.py`, `test_update_eol_churn.py`,
`test_update_bootstrap_cache_refresh.py`,
`test_update_behind_count_recovery.py`,
`test_update_apply_shallow_count.py`,
`test_update_cold_start_gateway_liveness.py`,
`test_update_gateway_launcher_refresh.py`,
`test_update_gateway_restart_aborted.py`,
`test_update_fleet_restart_timeout.py`, `test_update_version_report.py`,
`test_update_cron_drain.py`, `test_install_diverged_update.py`,
`test_desktop_update_windows_python_handoff.py`,
`test_desktop_update_windows_timestamp.py`. A dedicated future
update-hardening effort should triage this list the same way this audit
triaged the rest: check whether the fork's inline implementation already
covers the behavior under a different name before assuming it's missing.

## Recommended order for future work

1. Cross-process update lock (finding #1) — highest value, matches an
   exposure this fork actually has via the dashboard spawn path.
2. Windows gateway-pause/quarantine hardening (finding #2) — builds on
   the existing quarantine mechanism the fork already has.
3. Self-lock deferral (finding #5), once #1 exists to build on.
4. Orphaned Desktop backend reap (finding #3), gated on confirming the
   fork's Desktop packaging can actually hit this race.
5. HEAD-moved gate (finding #4) — cheap, low priority given the fork's
   pre-pull checkout already narrows the window.

## Testing performed for the change that did land

- New test file (`test_update_zip_atomic_replace.py`, 2 tests): pass.
- Fork's full existing update-related test suite (18 files, 252 tests):
  250 passed, 2 pre-existing failures
  (`test_update_autostash.py::test_bootstrap_marker_not_autostashed_by_update`,
  `test_uv_tool_update.py::TestCmdUpdatePipUsesUvTool::test_runs_uv_pip_install_when_not_uv_tool`)
  unrelated to this change (`.gitignore`/autostash marker handling, and a
  `uv pip install` flag mismatch respectively). Confirmed byte-identical
  by temporarily restoring the pre-change `hermes_cli/main.py` via
  `git checkout HEAD~1 -- hermes_cli/main.py`, re-running just those two
  tests, and observing the same two failures with the same assertion
  text, before restoring `HEAD`.
