# Asupersync Leverage Inventory

- Bead: `bd-xdcrh.1`
- Date: 2026-03-10
- Scope: identify only the places where more `asupersync` leverage is likely to improve cancel-correctness, deadline propagation, quiescent shutdown, or runtime ownership.

This is not a rewrite wishlist. Pi already gets real value from `asupersync` in the runtime bootstrap, provider HTTP/TLS stack, extension budget model, and many tool/test paths. The goal here is to leave behind a precise map of the remaining high-ROI sites so downstream beads can start from concrete code locations instead of rerunning archaeology.

## Decision Legend

- `keep`: current shape already matches the runtime intent, or change would likely be aesthetic only.
- `refactor`: high-confidence improvement with a bounded surface.
- `defer`: plausible future work, but only after higher-value context/thread-ownership fixes or with measured evidence.

## Cluster 1: Extension/session helper surfaces still mint fresh request contexts

- Recommendation: `refactor`
- Why it matters: these helpers are reached from extension hostcall and interactive event flows that already have a meaningful current `Cx` or manager-level timeout. Creating a brand-new request scope at the helper boundary disconnects session locking and immediate-save work from the caller's deadline/cancellation semantics.
- Minimal targeted refactor:
  - Use `AgentCx::for_current_or_request()` or `Cx::current().unwrap_or_else(Cx::for_request)` at trait/helper boundaries where threading `&AgentCx` through the trait is awkward.
  - If a helper already sits on an internal API boundary, prefer accepting `&AgentCx` directly instead of recreating one.
- Exact modules and functions:
  - `src/session.rs`
    - `impl ExtensionSession for SessionHandle::{get_state,get_messages,get_entries,get_branch,set_name,append_message,append_custom_entry,set_model,get_model,set_thinking_level,get_thinking_level,set_label}`
  - `src/interactive/ext_session.rs`
    - `InteractiveExtensionHostActions::append_to_session`
    - `impl ExtensionSession for InteractiveExtensionSession::{get_state,get_messages,get_entries,get_branch,set_name,append_message,append_custom_entry,set_model,get_model,set_thinking_level,get_thinking_level,set_label}`
  - `src/agent.rs`
    - `AgentSessionHostActions::append_to_session`
- Keep unchanged:
  - The trait split between `ExtensionSession` and `ExtensionHostActions`
  - The current idle-vs-streaming enqueue semantics for extension messages
- Downstream beads:
  - `bd-xdcrh.2.1`
  - `bd-xdcrh.5`

## Cluster 2: Agent/RPC helper layers already have a live context but recreate request scope anyway

- Recommendation: `refactor`
- Why it matters: the top-level async flows already have a natural request lifetime, but several nested helpers recreate `AgentCx::for_request()` instead of inheriting that scope. That weakens future deadline/checkpoint work and makes cancellation behavior less explainable than it needs to be.
- Minimal targeted refactor:
  - Create one `AgentCx` near the top of each run/turn path and thread clones or references downward.
  - Keep the existing explicit-`cx` pattern where it already exists, then extend it to the nested helpers below it.
- Exact modules and functions:
  - `src/rpc.rs`
    - `run`
    - `run_prompt_with_retry` already accepts `cx: AgentCx`; preserve that pattern
    - `run_extension_command` already accepts `cx: AgentCx`; preserve that pattern
    - `apply_thinking_level`
    - `apply_model_change`
    - `maybe_auto_compact`
    - `rpc_emit_extension_ui_request`
  - `src/agent.rs`
    - `AgentSession::{set_provider_model,sync_runtime_selection_from_session_header,maybe_compact,apply_compaction_result,compact_synchronous,enable_extensions_with_policy,save_and_index,persist_session,revert_last_user_message}`
  - `src/main.rs`
    - `run` session snapshot/model-history/shutdown-flush lock sections
- Keep unchanged:
  - `src/rpc.rs::run_prompt_with_retry` carrying an explicit `AgentCx` parameter
  - `src/rpc.rs::run_extension_command` carrying an explicit `AgentCx` parameter
  - The current runtime split between CLI/bootstrap and mode-specific execution
- Downstream beads:
  - `bd-xdcrh.2.2`
  - `bd-xdcrh.2.3`
  - `bd-xdcrh.3`
  - `bd-xdcrh.5`

## Cluster 3: Session persistence and discovery still rely on raw threads plus oneshot rendezvous

- Recommendation: `refactor`
- Why it matters: the raw-thread wrappers do isolate blocking work, but the waiting side also recreates fresh request contexts. That means outer cancellation and shutdown budgets do not naturally propagate into the wait path, and worker ownership lives outside the runtime even when the caller is already async.
- Minimal targeted refactor:
  - Migrate the JSONL scan/open/save wrappers to `asupersync::runtime::spawn_blocking_io` or a small runtime-owned blocking helper.
  - Accept an inherited `&AgentCx` or `&Cx` for the wait path instead of constructing a new request context internally.
  - For SQLite helpers, pass the caller context through instead of minting a new one inside the helper.
- Exact modules and functions:
  - `src/session.rs`
    - `Session::resume_with_picker`
    - `Session::continue_recent_in_dir`
    - `Session::open_v2_with_diagnostics`
    - `Session::open_jsonl_with_diagnostics`
    - `Session::save_inner`
    - `scan_sessions_on_disk`
  - `src/session_sqlite.rs`
    - `load_session`
    - `load_session_meta`
    - `save_session`
    - `append_entries`
- Keep unchanged:
  - JSONL atomic temp-file persist behavior
  - Session index reconciliation heuristics
  - SQLite schema/layout choices
- Downstream beads:
  - `bd-xdcrh.2.1`
  - `bd-xdcrh.4.1`
  - `bd-xdcrh.5`

## Cluster 4: Background compaction is async work running on a thread-owned runtime

- Recommendation: `refactor`
- Why it matters: compaction is provider/network work, not plain blocking I/O. Today it launches on a dedicated OS thread that builds a fresh current-thread runtime per attempt. That preserves foreground responsiveness, but it also gives up structured task ownership and direct cancellation inheritance.
- Minimal targeted refactor:
  - Keep the two-phase worker state machine and quota logic.
  - Rehost the actual compaction future on the existing multi-thread runtime behind an owned task handle and explicit shutdown/abort path.
- Exact modules and functions:
  - `src/compaction_worker.rs`
    - `CompactionWorkerState::start`
    - `run_compaction_thread`
  - `src/agent.rs`
    - `AgentSession::maybe_compact`
    - `AgentSession::compact_synchronous`
    - `AgentSession::apply_compaction_result`
- Keep unchanged:
  - Cooldown/attempt-count quota policy
  - The non-blocking "apply finished result, then maybe launch next compaction" model
- Downstream beads:
  - `bd-xdcrh.4.2`
  - `bd-xdcrh.3`
  - `bd-xdcrh.5`

## Cluster 5: Pipe pumps and subprocess cleanup threads are mostly acceptable as-is

- Recommendation: `keep` for now, with a narrow `defer` for measurement-based follow-up
- Why it matters: these threads are tied to blocking OS pipes and short-lived child-process cleanup. Replacing them preemptively with generic runtime blocking lanes is not obviously better, and several of these surfaces already combine timeouts with explicit kill/cleanup logic correctly.
- Exact modules and functions:
  - `src/tools.rs`
    - `BashTool::execute`
    - `GrepTool::execute`
    - `FindTool::execute`
    - `ProcessGuard::drop`
  - `src/extensions.rs`
    - extension exec/tool child-process pump paths
  - `src/extension_dispatcher.rs`
    - extension exec/command child-process pump paths
- Keep unchanged:
  - `src/tools.rs::{EditTool::execute,WriteTool::execute,HashlineEditTool::execute}` using `spawn_blocking_io` for atomic writes
  - `AgentCx::for_current_or_request()` timer usage in tool polling loops
  - Current `TERM -> grace -> KILL` and process-tree cleanup semantics
- Defer only if evidence appears:
  - thread proliferation under parallel subprocess load
  - shutdown hangs or orphan cleanup regressions
  - measurable contention that runtime-owned blocking lanes would clearly solve
- Downstream beads:
  - `bd-xdcrh.4.3`

## Cluster 6: Long-lived loop/checkpoint work should wait until inherited-context fixes land

- Recommendation: `defer`, then revisit with focused tests
- Why it matters: several loops already use timer-driver-aware `now` calculations, but explicit checkpoint/cancellation discipline is hard to evaluate cleanly while the surrounding helper layers are still recreating fresh request contexts.
- Exact modules and functions to revisit after Clusters 1-4:
  - `src/rpc.rs`
    - `run_prompt_with_retry` retry backoff loop
    - `maybe_auto_compact`
    - `rpc_emit_extension_ui_request`
  - `src/tools.rs`
    - `BashTool::execute`
    - `GrepTool::execute`
    - `FindTool::execute`
  - `src/compaction_worker.rs`
    - `CompactionWorkerState::try_recv`
- Keep unchanged right now:
  - timer-driver-aware `now` lookups in `src/tools.rs` and `src/http/client.rs`
  - extension manager timeout clamping via `ExtensionManager::effective_timeout`
- Downstream beads:
  - `bd-xdcrh.3`
  - `bd-xdcrh.3.1`
  - `bd-xdcrh.5`

## `bd-xdcrh.3.1` Checkpoint Boundary Map

This section turns Cluster 6 into an implementation-ready map. The rule is simple: only add an
explicit checkpoint where cancellation can happen between complete iterations or immediately before
the next blocking wait. Do not checkpoint in the middle of a state transition that mutates session,
tool-output, or UI-delivery state in a way that would leave a partial effect behind.

### 1. Shell/process polling loops

- `src/tools.rs::run_bash_command`
  - Why it matters: this is a long-lived poll/drain loop that can run for minutes while a child
    process is alive and streaming output.
  - Safe checkpoint location: after the current `rx.try_recv()` drain finishes and after the
    timeout / terminate-deadline decision is computed, immediately before the `sleep(now, tick)`
    that begins the next poll iteration.
  - Safe checkpoint location: in the post-exit drain loop, only on the `TryRecvError::Empty`
    branch before the next `sleep(now, tick)`.
  - Unsafe region: inside `ingest_bash_chunk(...)`, because that function updates total byte/line
    counts, truncation state, and optional spill-to-temp-file state as one logical unit.
  - Unsafe region: between `terminate_process_group_tree(pid)` and the follow-up grace-period kill;
    cancellation there could leave shutdown state ambiguous.
  - Expected behavior after insertion: cancellation stops future polling promptly, but never drops a
    half-ingested chunk or interrupts the terminate-then-kill escalation once it has started.

- `src/rpc.rs::run_bash_rpc`
  - Why it matters: same shape as `run_bash_command`, but used by the RPC shell path with explicit
    abort handling and bounded in-memory/tail-file state.
  - Safe checkpoint location: at the bottom of the main `exit_code = loop { ... }` body, after the
    current `rx.try_recv()` drain and child-status check, immediately before `sleep(wall_now(), tick)`.
  - Safe checkpoint location: in the remaining-output drain loop, only on the `TryRecvError::Empty`
    branch before the next `sleep(...)`.
  - Unsafe region: inside `ingest_bash_rpc_chunk(...)`, for the same reason as the tool path.
  - Unsafe region: after `abort_rx.try_recv().is_ok()` has triggered and the code has committed to
    `guard.kill()`. Finish the kill path once started.
  - Expected behavior after insertion: cancelled RPC shell work stops polling quickly, but tail-file
    spill state and process-tree cleanup remain internally consistent.

- `src/tools.rs::GrepTool::execute`
  - Why it matters: the main loop repeatedly drains bounded stdout/stderr channels from ripgrep and
    waits for process exit; the post-exit join loop can also run for a while under backpressure.
  - Safe checkpoint location: after `drain_rg_stdout(...)` / `drain_rg_stderr(...)` finish for the
    current iteration and immediately before the `sleep(now, tick)` on the `Ok(None)` branch.
  - Safe checkpoint location: in the `while !stdout_thread.is_finished() || !stderr_thread.is_finished()`
    loop, after the current drain pass and before `sleep(wall_now(), Duration::from_millis(1))`.
  - Unsafe region: inside `drain_rg_stdout(...)` while it is consuming JSON lines into the match
    accumulator; stop between batches, not mid-batch.
  - Unsafe region: after the code decides the match limit was reached and commits to `guard.kill()`
    plus channel draining. That termination sequence should remain indivisible.
  - Expected behavior after insertion: cancellation stops waiting for more ripgrep output, but does
    not leave the match accumulator or explicit process termination in a half-applied state.

- `src/tools.rs::FindTool::execute`
  - Why it matters: the fd-backed search loop mirrors grep/bashtool polling, just with bulk stdout
    readers instead of per-line JSON parsing.
  - Safe checkpoint location: after the current child-status check and before `sleep(now, tick)`.
  - Unsafe region: none inside the loop body beyond the existing `try_wait` / timeout state update;
    this is one of the cleaner candidates because each iteration already has a single wait boundary.
  - Expected behavior after insertion: cancellation stops future polling with bounded latency and
    does not affect already-buffered stdout/stderr collection.

### 2. Retry / stream / UI-delivery loops

- `src/rpc.rs::run_prompt_with_retry`
  - Why it matters: this retry loop can span multiple model calls and explicit backoff delays.
  - Safe checkpoint location: at the top of the next retry iteration, before creating the next
    `AbortHandle` / `AbortSignal` pair.
  - Safe checkpoint location: after emitting `auto_retry_start` and before entering the backoff
    sleep for `delay_ms`.
  - Unsafe region: while the current attempt owns `abort_handle_slot` or the session mutex and is
    actively running `run_text_with_abort(...)` / `run_with_content_with_abort(...)`.
  - Unsafe region: between a completed attempt and the `abort_handle_slot` cleanup that sets the
    slot back to `None`.
  - Expected behavior after insertion: cancellation prevents the next retry or backoff wait from
    starting, but never strands a stale abort handle or interrupts an in-flight model attempt in an
    internally inconsistent state.

- `src/agent.rs::stream_assistant_response`
  - Why it matters: this is the core streaming event loop for provider output, including abort and
    error synthesis.
  - Safe checkpoint location: right before awaiting the next `stream.next()` / abort-select result.
  - Safe checkpoint location: after a full `StreamEvent` arm completes and the loop is about to
    return to the next iteration.
  - Unsafe region: inside any single `StreamEvent` arm while mutating the partial assistant message,
    especially between `MessageStart` emission and the corresponding state update.
  - Unsafe region: inside the abort-handling branch after the abort message has been synthesized and
    `MessageUpdate` emission has begun.
  - Expected behavior after insertion: cancellation aborts between provider events, not halfway
    through applying one event to the assistant transcript.

- `src/rpc.rs` extension UI request bridge
  - Exact loop: `while let Ok(request) = extension_ui_rx.recv(&cx).await`
  - Why it matters: this queue serializes response-required extension UI requests and performs the
    active/queued transition under a mutex.
  - Safe checkpoint location: immediately before the next `extension_ui_rx.recv(&cx).await`.
  - Safe checkpoint location: after the `ui_state` mutex is released and before `rpc_emit_extension_ui_request(...)`
    is called for the next request.
  - Unsafe region: while holding `ui_state.lock(&cx)` and mutating `active` / `queue`; the dequeue,
    overflow-cancel, and next-active promotion are each single logical transitions.
  - Expected behavior after insertion: cancellation stops accepting new UI requests or delays the
    next emission, but never leaves `active` and `queue` disagreeing about ownership.

- `src/interactive/agent.rs::flush_ui_stream_batcher_with_backpressure`
  - Why it matters: this loop can spend time awaiting channel capacity while flushing a detached
    batch of UI deltas.
  - Conditionally safe checkpoint location: between completed `sender.send(&cx, msg).await` calls,
    but only if we explicitly accept that cancellation may drop the remaining detached tail.
  - Unsafe region: after `pending` has been taken out of the batcher if the caller still expects a
    full flush guarantee. Today the detached batch has no requeue-on-cancel path.
  - Current recommendation: do not add a checkpoint yet. First decide whether shutdown/cancel should
    preserve or drop unsent UI deltas; the safe boundary depends on that policy choice.

### 3. Session and persistence loops

- `src/session.rs::scan_sessions_on_disk`
  - Why it matters: this is the main on-disk session discovery loop and can scan many session files.
  - Candidate safe checkpoint location after worker migration: between completed `for entry in dir_entries`
    iterations, after finishing one file's metadata/load decision and before starting the next.
  - Unsafe region: do not checkpoint while reusing a known entry or halfway through `load_session_meta(...)`
    for one file; each file decision should remain indivisible.
  - Current recommendation: do not insert checkpoints into the current raw-thread body. The right
    first move is to inherit caller context on the wait side and/or move the scan onto
    runtime-managed blocking work (`bd-xdcrh.4.1`), then add cooperative checkpoints between files.

- `src/session.rs::save_inner`
  - Why it matters: this function contains the most obvious persistence loops (`for entry in &entries`
    during full rewrite and `for entry in new_entries` during incremental append serialization).
  - Safe checkpoint location: before spawning the blocking worker and after the worker result is
    fully received and applied to in-memory counters.
  - Unsafe region: the full-rewrite loop writing header + entries into the temp file.
  - Unsafe region: the incremental append serialization loop and the subsequent locked
    `file.write_all(&serialized_buf)` append path.
  - Current recommendation: treat these write phases as intentionally indivisible. A checkpoint in
    the middle would weaken atomicity semantics more than it would improve cancel latency.

- `src/session.rs` picker / recent-session merge loops
  - Exact loops: the `for entry in entries.into_iter().chain(scanned.into_iter())` merges in
    `continue_recent_in_dir` and `resume_with_picker`.
  - Why it matters: these loops can touch many indexed + scanned entries, but they are pure in-memory
    merge logic once the blocking index/scan work is done.
  - Safe checkpoint location: between completed `by_path` merge iterations if these loops ever prove
    large enough to matter under measurement.
  - Current recommendation: defer. The bigger cancellation win is on the blocking worker wait path,
    not in these short in-memory merges.

### 4. Non-candidates or keep-as-is surfaces

- `src/compaction_worker.rs::CompactionWorkerState::try_recv`
  - Not a checkpoint candidate. It is a single non-blocking poll helper, not a long-lived loop.

- `src/rpc.rs` stdio pump threads (`read_line` / stdout writer loops)
  - Keep for now. These are dedicated blocking-I/O threads; if they are ever migrated, the natural
    checkpoint boundary would be after a full line is processed and before the next blocking read or
    backpressure retry, but that is runtime-ownership work first, not checkpoint work first.

### Recommended implementation order for `bd-xdcrh.3.2`

1. `src/tools.rs::run_bash_command`
2. `src/rpc.rs::run_bash_rpc`
3. `src/tools.rs::GrepTool::execute`
4. `src/rpc.rs::run_prompt_with_retry`
5. `src/agent.rs::stream_assistant_response`
6. `src/rpc.rs` extension UI request bridge
7. Revisit `scan_sessions_on_disk` only after the worker/wait-path refactor lands
8. Leave `save_inner` and `flush_ui_stream_batcher_with_backpressure` unchanged until their
   indivisibility / drop-policy questions are answered explicitly

### `bd-xdcrh.3.2` implementation note

The current implementation landed the four highest-confidence sites from this map:

- `src/tools.rs::run_bash_command`
- `src/rpc.rs::run_bash_rpc`
- `src/rpc.rs::run_prompt_with_retry`
- `src/agent.rs::stream_assistant_response`

Two remaining candidates were deferred deliberately rather than forgotten:

- `src/tools.rs::GrepTool::execute` and `src/tools.rs::FindTool::execute`
  - The poll boundaries are mechanically safe, but these tool APIs do not currently expose an
    explicit cancelled result shape. Adding checkpoints there would force a separate contract
    decision about whether cancellation returns partial search results or a new tool error.

- `src/rpc.rs` extension UI request bridge
  - The queue transition under `RpcUiBridgeState` is internally coherent today, but a checkpoint
    after the mutex is released and before emit is only safe if cancellation also specifies whether
    the active request is cancelled, re-queued, or left pending. That ownership policy should land
    with the bridge follow-up, not as an implicit side effect of this bead.

## High-Confidence Keep Surface

These are the places where Pi is already using `asupersync` in the right way and should be treated as baseline patterns, not cleanup targets:

- `src/http/client.rs`
  - `RequestBuilder::send`
  - response timeout paths that consult `Cx::current()` for timer-driver time
- `src/extensions.rs`
  - `cx_with_deadline`
  - `ExtensionManager::{with_budget,set_budget,extension_cx,effective_timeout,dispatch_event*,execute_command,execute_shortcut}`
- `src/tools.rs`
  - `EditTool::execute`
  - `WriteTool::execute`
  - `HashlineEditTool::execute`
  - wide use of `asupersync::fs` and `spawn_blocking_io`
- `src/interactive.rs`
  - the TUI core itself is not the leverage problem; the actionable context issue is in `src/interactive/ext_session.rs`

## Testing Obligations For Downstream Work

Existing strong examples:

- `src/extensions.rs` budget/timeout tests
- `src/http/client.rs` deterministic timeout/stream tests
- `src/tools.rs` broad `asupersync::test_utils::run_test` coverage

Coverage gaps worth filling as code lands:

1. Session/extension-session helper calls should inherit a current deadline instead of silently creating an unbounded request scope.
2. Session JSONL/SQLite worker wrappers should prove cancellation and shutdown behavior when the caller budget expires mid-wait.
3. Compaction worker changes should prove that background work respects shutdown budgets and does not outlive the owning runtime.
4. RPC retry/auto-compaction paths should prove deadline propagation across nested helper calls.

## Recommended Execution Order

1. `bd-xdcrh.2.1`: fix session and extension-session helper inheritance first.
2. `bd-xdcrh.2.2`: thread inherited `AgentCx` through RPC control/model/thinking helpers.
3. `bd-xdcrh.2.3`: thread inherited `AgentCx` through agent and interactive background helpers.
4. `bd-xdcrh.4.1`: move session JSONL/SQLite worker islands under runtime-owned blocking execution.
5. `bd-xdcrh.4.2`: migrate background compaction ownership onto the runtime.
6. `bd-xdcrh.3` and `bd-xdcrh.3.1`: revisit checkpoint boundaries once the inherited-context work is in place.
7. `bd-xdcrh.4.3`: only revisit subprocess pipe pumps if measured evidence says the current thread model is a real problem.
8. `bd-xdcrh.5`: land deterministic tests alongside each slice instead of saving them for the end.
9. `bd-xdcrh.6` and `bd-xdcrh.6.1`: only escalate to broader supervision/AppSpec work if lifecycle hazards remain after the narrow fixes above.

## `bd-xdcrh.6.2` Supervision / AppSpec Decision

- Recommendation: `defer` broad supervision or AppSpec adoption
- Why this is the right call now:
  - Pi already has a usable ownership spine: `src/main.rs` owns the multi-thread `asupersync` runtime, then threads a runtime handle into `AgentSession`, which already concentrates per-session ownership for provider state, session state, extension region, and background compaction.
  - The highest-value worker-island fix already landed in `bd-xdcrh.4.2`, so background compaction is no longer evidence that Pi lacks runtime ownership.
  - The remaining lifecycle edges are mode-specific bridge boundaries, especially interactive and RPC handoff points plus the explicit `ExtensionRegion::shutdown()` contract, not a missing repo-wide supervision framework.
  - The known raw-thread islands are still intentional. RPC stdio loops and subprocess pipe pumps are tied to hard blocking I/O and descendant-pipe behavior where a dedicated thread remains simpler and safer than a generic AppSpec-shaped abstraction.
- What to keep doing instead:
  - make shutdown edges explicit where async work crosses UI or blocking boundaries
  - preserve the existing `main` -> `AgentSession` -> mode-bridge ownership model
  - treat dedicated blocking threads as guilty only if measured failures appear, not because they are aesthetically out of style
- Non-goals:
  - do not introduce a new top-level supervision tree just to mirror `asupersync` concepts more visibly
  - do not wrap existing interactive or RPC flows in a broad AppSpec layer without a concrete failure mode to solve
  - do not use this bead to reopen already-closed worker-lifecycle decisions such as compaction runtime ownership
- Revisit only if new evidence appears:
  - bridge tasks demonstrably outlive their owning mode and cannot be fixed with a local shutdown or abort contract
  - explicit extension shutdown becomes unmanageable across more than one narrow boundary
  - raw-thread islands show measured shutdown hangs, orphan work, or thread proliferation that runtime-owned structure would clearly improve
