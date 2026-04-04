# Changelog

This repo tracks work in Beads (`br`) under `.beads/`. This `CHANGELOG.md` is a
human-facing summary generated from **closed issues**.

## Updating the changelog

1. Generate grouped entries from Beads:
   - `br changelog --since YYYY-MM-DD`
   - or, once tags exist: `br changelog --since-tag vX.Y.Z`
2. Move the generated entries into the appropriate release section.
3. Keep edits deterministic: same inputs → same output.

## Unreleased

Generated on 2026-02-04 from issues closed since 2026-02-03.

### Chore

- [P3] bd-1ano Docs: rpc.md (stdin/stdout protocol)
- [P3] bd-zu7a Docs: session.md (JSONL format + deletion + branching)

### Feature

- [P0] bd-gi3 Workstream: HTTP Migration to asupersync
- [P2] bd-346 Workstream: Session Index Integration

### Task

- [P0] bd-2ke Impl: QuickJS promise bridge + job draining semantics
- [P0] bd-8mm Impl: Deterministic event loop scheduler (asupersync)
- [P0] bd-37z Spec: Hostcall ABI + capability manifest
- [P0] bd-123 Spec: PiJS runtime contract + event loop semantics
- [P0] bd-1pf Set up VCR-style test infrastructure for provider streaming
- [P0] bd-1iy Migrate Gemini provider to asupersync HTTP
- [P0] bd-1vo Migrate OpenAI provider to asupersync HTTP
- [P0] bd-37l Migrate Anthropic provider to asupersync HTTP
- [P0] bd-pwz Implement SSE streaming adapter for asupersync
- [P0] bd-9sa Implement asupersync HTTP client wrapper
- [P1] bd-aywz Unblock clippy -D warnings (error_hints + session_index tests)
- [P1] bd-3uuf Unit tests: session_index core behaviors + lock/error paths
- [P1] bd-2tu3 Interactive: implement legacy app actions (clear/exit/suspend/externalEditor)
- [P1] bd-x85o Interactive: /tree full navigator UI + branch summarization
- [P1] bd-2jdz Interactive: implement /new (start new session without restart)
- [P1] bd-2knt Interactive: implement in-app /resume session picker
- [P1] bd-1rfa Editor: Shift+Enter newline, Enter submit (legacy multiline)
- [P1] bd-28t8 Autocomplete: UI dropdown + selection/insertion
- [P1] bd-28rk Autocomplete: core provider (commands/prompts/skills/files/paths)
- [P1] bd-340x Message queue: keybindings + editor interactions
- [P1] bd-3v08 Message queue: core data model + delivery boundaries
- [P1] bd-gze Interactive: route KeyMsg -> action -> behavior
- [P1] bd-3qm Keybindings: load ~/.pi/agent/keybindings.json + merge
- [P1] bd-103 Keybindings: parse key strings + normalize to KeyId
- [P1] bd-cru Keybindings: define action catalog + default bindings
- [P1] bd-ocv Unit tests: error type conversions + display invariants
- [P1] bd-2hr Spec: Unified JS+WASM capability model
- [P1] bd-w83 Testing: Event loop conformance + determinism
- [P1] bd-3sf Spec: Extension manifest + capability inference
- [P1] bd-3bs Tooling: Compatibility scanner + evidence ledger
- [P1] bd-2ki Tooling: Compatibility rewrite map + extc contract
- [P1] bd-2ds Connector: HTTP/network minimal API (policy-gated)
- [P1] bd-2rl Connector: Filesystem minimal API (capability-scoped)
- [P1] bd-3ev Benchmark suite: startup/memory/streaming budgets
- [P1] bd-iub Spec refresh: extensions/resources + parity audit
- [P1] bd-ah1 Implement session persistence cycle tests
- [P1] bd-ivj Migrate Azure OpenAI provider to asupersync HTTP
- [P1] bd-261 Unit tests: extension protocol parsing + schema validation
- [P2] bd-33jg Docs: FEATURE_PARITY.md - refresh statuses + bead references
- [P2] bd-30zd Docs: README - correct asupersync vs tokio migration status
- [P2] bd-1ve4 CI: add cross-platform build/test matrix (linux/macOS/windows)
- [P2] bd-4uap Unit tests: session_picker listing + ordering + formatting
- [P2] bd-xr4f Interactive: /reload applies reloaded resources to state
- [P2] bd-lqec Interactive: /fork creates new session file from current branch
- [P2] bd-1bzn Session picker: delete sessions (Ctrl+D) using trash when available
- [P2] bd-11pg Tests: message queue semantics (steering/follow-up)
- [P2] bd-k7ke Message queue: wire steering_mode/follow_up_mode config
- [P2] bd-l6lx Tests: keybindings parser + matching + /hotkeys
- [P2] bd-1dd3 Interactive: render /hotkeys from active keybindings
- [P2] bd-331 Dev UX: diagnostics + trace viewer for extensions
- [P2] bd-2rj Session index integration tests (persist/index/list/continue)
- [P2] bd-15n Update --continue flag to use SQLite index
- [P2] bd-1mh Update session picker to use SQLite index
- [P2] bd-3nz Wire session indexing into session.persist()
- [P2] bd-3tu Crates publishing workflow for pi_agent_rust + libs
- [P3] bd-24rp Docs: AGENTS.md - remove stale tokio migration statements
- [P3] bd-jw5q Docs: PLAN_TO_COMPLETE_PORT.md - reconcile checklist with reality
- [P3] bd-1iz5 Error UX: define hint taxonomy + mapping (Error -> hints)
- [P3] bd-3d8 Implement /theme slash command
- [P3] bd-qpm Apply theme colors to TUI components
- [P3] bd-37a Implement theme file format and loader
