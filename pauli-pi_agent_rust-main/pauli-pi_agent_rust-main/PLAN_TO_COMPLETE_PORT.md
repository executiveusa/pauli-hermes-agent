# Plan: Complete pi_agent_rust Port

> **Goal:** 100% feature/functionality coverage with clear conformance harness and benchmarking, fully leveraging asupersync, rich_rust, and charmed_rust.

> **Important:** This document is a historical overview, not the live backlog.
> For the authoritative plan, use Beads:
> - `bv --robot-plan`
> - `bv --robot-priority`
> - `br ready`
> - `br show <id>`

---

## Executive Summary

**Current State:** ~85-90% complete
- Core types, tools, sessions, CLI, interactive TUI: ✅ Implemented
- Multi-provider (Anthropic/OpenAI/Gemini/Azure): ✅ Implemented
- RPC mode (stdin/stdout JSON protocol): ✅ Implemented (see `src/rpc.rs`, `tests/rpc_mode.rs`)
- Conformance harness: ✅ Tools fixture suite + integration tests
- Benchmarks: ✅ Truncation + SSE parsing baseline

**Target State:** Production-ready CLI with:
- Full interactive TUI using charmed_rust (Elm Architecture)
- All streaming via asupersync (cancel-correct, structured concurrency)
- Beautiful output via rich_rust (markup, tables, panels)
- Comprehensive conformance test suite (tools + RPC + extensions) with TypeScript reference capture
- Benchmark harness proving performance targets (including extension hostcall dispatch)

**Primary Remaining Work:**
1. **Extensions runtime** + conformance harness: `bd-btq`, `bd-1e0`, `bd-2i5`, `bd-269`
2. **Themes integration** (apply/switch, `/theme`, settings): `bd-22p`, `bd-qpm`, `bd-3d8`, `bd-ieym`
3. **asupersync capability hardening** (AgentCx, cancel-correctness): `bd-3i7u`, `bd-1xf`

---

## Part 1: Library Integration Strategy

### 1.1 asupersync Integration

**Status (today):** `pi_agent_rust` runs on `asupersync` for runtime + HTTP/TLS and provider streaming (see `src/http/client.rs` + `src/sse.rs`).

**Remaining:** Capability wrapper (`AgentCx`) and deeper context wiring are tracked in `bd-3i7u` and `bd-1xf`.

**Key APIs to Leverage:**
```rust
use asupersync::{Cx, Outcome, Budget, Scope};
use asupersync::http::client::HttpClient;
use asupersync::tls::TlsConnectorBuilder;
use asupersync::database::sqlite::SqliteConnection;
use asupersync::io::{AsyncRead, AsyncWrite, AsyncReadExt, AsyncWriteExt};
```

**Benefits:**
- Structured concurrency (no orphan tasks)
- Cancel-correct operations (bounded cleanup)
- Deterministic testing via LabRuntime
- Batteries-included HTTP/TLS/SQLite

### 1.2 rich_rust Integration

**Purpose:** All non-interactive terminal output

**Usage Patterns:**
```rust
use rich_rust::prelude::*;

// Console for formatted output
let console = Console::new();
console.print("[bold green]✓[/] Tool executed successfully");

// Tables for structured data
let mut table = Table::new().title("Session Info");
table.add_row_cells(["Tokens", &format!("{}", usage.total_tokens)]);
console.print_renderable(&table);

// Panels for boxed content
let panel = Panel::from_text(&response).title("Assistant").width(80);
console.print_renderable(&panel);

// Progress bars for long operations
let bar = ProgressBar::new().width(40);
bar.set_progress(0.5);

// Markdown rendering (requires "markdown" feature)
let md = Markdown::new(&response_text);
console.print_renderable(&md);

// Syntax highlighting (requires "syntax" feature)
let syntax = Syntax::new(&code, "rust");
console.print_renderable(&syntax);
```

**Add Features:**
```toml
rich_rust = { path = "../rich_rust", features = ["syntax", "markdown", "full"] }
```

### 1.3 charmed_rust Integration

**Purpose:** Full interactive TUI using Elm Architecture

**Architecture:**
```rust
use bubbletea::{Program, Model, Message, Cmd, KeyMsg};
use lipgloss::{Style, Border, Position};
use bubbles::{textinput::TextInput, spinner::Spinner, viewport::Viewport};
use glamour::{render as render_markdown, Style as MdStyle};

struct PiTui {
    // Editor state
    input: TextInput,
    history: Vec<String>,
    history_index: Option<usize>,

    // Display state
    viewport: Viewport,
    spinner: Spinner,
    status: StatusLine,

    // Agent state
    messages: Vec<Message>,
    streaming: bool,
    current_response: String,
    thinking: Option<String>,

    // Session state
    session: Session,
    config: Config,
}

impl Model for PiTui {
    fn init(&mut self) -> Cmd {
        Cmd::batch(vec![
            TextInput::blink(),
            self.spinner.tick(),
        ])
    }

    fn update(&mut self, msg: Message) -> Cmd {
        // Handle keyboard input, API responses, tool results
    }

    fn view(&self) -> String {
        // Render using lipgloss layout
        let layout = lipgloss::join_vertical(Position::Left, &[
            self.render_header(),
            self.render_messages(),
            self.render_status(),
            self.render_input(),
        ]);
        layout
    }
}
```

**Add Dependencies:**
```toml
bubbletea = { path = "../charmed_rust", package = "bubbletea" }
lipgloss = { path = "../charmed_rust", package = "lipgloss" }
bubbles = { path = "../charmed_rust", package = "bubbles" }
glamour = { path = "../charmed_rust", package = "glamour" }
```

---

## Part 2: Implementation Phases

### Phase 1: Fix Existing Issues ✅ COMPLETE

**1.1 Fix Failing Fixture Tests**
- [x] Investigated detail field serialization in bash, edit, read, write tools
- [x] Fixed `details` field expectations in fixtures
- [x] Fixed bash exit code bug (legacy race; no tokio dependency now)
- [x] All 67 fixture cases pass

**1.2 Clean Up Existing Code**
- [x] Run `cargo clippy --all-targets` - warnings addressed
- [x] Run `cargo fmt` - consistent formatting
- [x] Run `cargo test` - all 67 tests pass

### Phase 2: Library Dependencies ✅ COMPLETE

**2.1 Update Cargo.toml** ✅
```toml
# Interactive TUI (charmed_rust - Elm Architecture)
bubbletea = { path = "../charmed_rust/crates/bubbletea" }
lipgloss = { path = "../charmed_rust/crates/lipgloss" }
bubbles = { path = "../charmed_rust/crates/bubbles" }
glamour = { path = "../charmed_rust/crates/glamour" }
```

**2.2 Create Wrapper Types**
- 🔶 `AgentCx` - Capability context for agent operations (tracked in `bd-3i7u`)
- [x] `RichConsole` - Wrapper for rich_rust Console with Pi-specific methods (`PiConsole`)
- [x] `TuiApp` - bubbletea Model implementation (`src/interactive.rs`)

### Phase 3: Interactive TUI ✅ COMPLETE

**3.1 Core TUI Structure**
- [x] `src/interactive.rs` - Main Model implementation (PiApp)
- [x] TextInput with history navigation (up/down)
- [x] Message display with markdown rendering (glamour)
- [x] Status footer with token counts and cost
- [x] Spinner while processing
- [x] Tool execution status display
- [x] Slash command system

**3.2 Editor Component**
- [x] Multi-line text input via bubbles TextArea
- [x] History navigation (up/down)
- [x] Ctrl+C to cancel/quit
- [x] Esc to quit when idle
- 🔶 Completions popup (slash commands, file paths) - tracked in `bd-1iwi` (see also `bd-3dr9`)
- ⬜ Shift+Enter for newline, Enter to submit - deferred (TextArea already handles this)

**3.3 Message Display**
- [x] Assistant responses with markdown rendering (glamour)
- [x] Thinking blocks (displayed inline)
- [x] Tool execution status (spinner + tool name)
- [x] Tool results (formatted output)
- ⬜ Images (terminal-dependent) - tracked in `bd-1iwi`

**3.4 Slash Commands** ✅ IMPLEMENTED
- [x] `/help` - Show available commands
- [x] `/model` - Show/switch model
- [x] `/thinking` - Set thinking level
- [x] `/history` - Show input history
- [x] `/clear` - Clear conversation
- [x] `/quit` (`/exit`) - Exit application
- [x] `/export` - Export to HTML

**3.5 Status Line**
- [x] Current model/provider in header
- [x] Token counts (input/output)
- [x] Cost ($X.XX)
- [x] Streaming indicator (spinner)
- [x] Status message for slash commands

**3.6 Agent Integration** ✅ COMPLETE
- [x] PiMsg enum for async agent events
- [x] Wire up agent execution from submit_message()
- [x] Handle streaming events via channel
- [x] Session persistence after each turn

### Phase 4: Provider streaming over asupersync ✅ COMPLETE

**4.1 Create HTTP Module**
- [x] `src/http/mod.rs` - HTTP client abstraction
- [x] `src/http/client.rs` - asupersync-based client
- [x] `src/http/sse.rs` - SSE streaming parser (reuse existing)

**4.2 Migrate Anthropic Provider**
- ✅ Provider streaming uses the asupersync HTTP client (`src/http/client.rs`) + SSE parser (`src/sse.rs`)
- ✅ TLS uses asupersync connector with native roots
- 🔶 Additional cancel-correctness + deterministic LabRuntime coverage tracked in `bd-1xf`

**4.3 Legacy cleanup**
- ✅ No tokio/reqwest dependencies remain in `Cargo.toml`
- 🔶 Remaining capability wiring tracked in `bd-3i7u` (AgentCx)

### Phase 5: Additional Providers ✅ COMPLETE

**5.1 OpenAI Provider** ✅
- [x] `src/providers/openai.rs`
- [x] Chat completions API
- [x] Streaming support
- [x] Function calling (tools)
- [x] Unit tests

**5.2 Google Gemini Provider** ✅
- [x] `src/providers/gemini.rs`
- [x] Generative AI API
- [x] Streaming support
- [x] Tool calling
- [x] Unit tests

**5.3 Azure OpenAI Provider** ✅
- [x] `src/providers/azure.rs`
- [x] Azure-specific endpoints
- [x] Streaming support
- [x] Tool calling
- [x] Unit tests

### Phase 6: Session Enhancements ✅ MOSTLY COMPLETE

**6.1 SQLite Index**
- ⬜ SQLite-based session index + search (deferred; consider adding a dedicated bead if/when needed)
- [x] Fast session listing (via filesystem mtime sort)
- ⬜ Sync-from-JSONL index (deferred)

**6.2 Tree Navigation** ✅
- [x] Branch creation (`create_branch_from`)
- [x] Branch switching (`navigate_to`)
- [x] Visual tree display in TUI (`/tree` command)
- [x] Branch summary support

**6.3 Session Picker UI** ✅
- [x] List recent sessions (`src/session_picker.rs`)
- [x] Search/filter (by directory)
- [x] Preview content (shows model, message count)
- [x] Select and resume (`--resume` flag)

### Phase 7: Conformance Testing (2-3 days)

**7.1 Reference Capture Infrastructure**
- ⬜ Tool reference-capture programs (TS/Go) - deferred; current focus is no-mock Rust coverage (`bd-26s`)

**7.2 Tool Conformance**
- ✅ Tool fixture suite lives in `tests/conformance/fixtures/` and is executed by `tests/conformance_fixtures.rs` (see `FEATURE_PARITY.md`)
- 🔶 Expand fixture coverage / add fuzz-discovered edge cases - track under `bd-26s`

**7.3 Provider Conformance**
- ✅ VCR-backed provider streaming tests (no mocks) - `bd-h7r`, `bd-gd1`
- ⬜ Additional error/rate-limit conformance - track under `bd-26s`

**7.4 Session Format Conformance**
- ✅ Covered by integration tests (`tests/session_conformance.rs`)
- ⬜ Migration tests (older versions) - deferred

### Phase 8: Benchmarking Harness (1-2 days)

**8.1 Benchmark Infrastructure**
- ✅ Existing benches live in `benches/` (see `BENCHMARKS.md`)
- ⬜ Add startup/TUI/streaming micro-benchmarks - defer until needed

**8.2 Performance Targets**
| Metric | Target | Measurement |
|--------|--------|-------------|
| Startup time | <100ms | Cold start to first prompt |
| TUI framerate | 60fps | Continuous rendering benchmark |
| Binary size | <15MB | `cargo build --release && ls -la` |
| Memory (idle) | <30MB | After startup, before first request |
| SSE throughput | >10MB/s | Parse rate for streaming events |

**8.3 CI Integration**
- ⬜ CI benchmark automation + regression detection - tracked in `bd-gqtd`
- ⬜ Bench/status badges in README - `bd-3nrc`

### Phase 9: Polish & Documentation (1-2 days)

**9.1 Error Messages**
- [x] User-friendly error formatting (rich_rust panels)
- 🔶 Actionable suggestions + context-aware hints - `bd-3am2`

**9.2 Documentation**
- [x] README.md with architecture
- 🔶 Rust API docs (rustdoc) - `bd-14od`
- 🔶 Configuration reference + troubleshooting - `bd-3m7f`

**9.3 Release Preparation**
- 🔶 Release engineering (versioning/changelog/cross-compilation/releases) - `bd-gqtd`

### Phase 10: Extensions Runtime (NEW - Primary Remaining Work)

**10.1 PiJS Runtime** (see `EXTENSIONS.md` for full spec)
- 🔶 Tracked in `bd-btq` (PiJS runtime umbrella; see `br show bd-btq`)

**10.2 Extension API**
- 🔶 Tracked under `bd-btq` (wiring work: `bd-2i5`)

**10.3 Extension UI**
- 🔶 UI surface tracked under `bd-btq` (RPC protocol exists; runtime integration pending)

**10.4 Extension Discovery & Loading**
- 🔶 Discovery + install resolution tracked in `bd-1e0`

**10.5 Conformance Testing**
- 🔶 Harness + fixtures tracked in `bd-269` (benchmarks: `bd-1fg`)

### Phase 11: Themes Discovery (NEW)

**11.1 Theme System**
- 🔶 Theme system tracked in `bd-22p` (apply colors `bd-qpm`, `/theme` `bd-3d8`, settings `bd-ieym`)

---

## Part 3: Conformance Test Strategy

### 3.1 Fixture-Based Testing

Each tool has a JSON fixture file with this structure:

```json
{
  "version": "1.1",
  "tool": "read",
  "reference_impl": "typescript",
  "captured_at": "2026-02-02T00:00:00Z",
  "cases": [
    {
      "name": "read_simple_file",
      "description": "Read a simple text file",
      "setup": [
        {"type": "create_file", "path": "test.txt", "content": "Hello\nWorld\n"}
      ],
      "input": {"path": "test.txt"},
      "expected": {
        "is_error": false,
        "content_contains": ["Hello", "World"],
        "content_regex": "^\\s*1→Hello",
        "details": {
          "lines_read": 2,
          "truncated": false
        }
      }
    }
  ]
}
```

### 3.2 Reference Capture Process

1. **TypeScript Reference:**
   ```bash
   cd legacy_pi_mono_code/pi-mono
   pnpm test:capture -- --tool read --output ../fixtures/read_tool.json
   ```

2. **Go Reference (for additional validation):**
   ```bash
   cd tests/conformance/go_reference
   go run ./cmd/capture_read --output ../fixtures/go_read_tool.json
   ```

3. **Rust Implementation Test:**
   ```bash
   cargo test conformance::test_read -- --nocapture
   ```

### 3.3 Coverage Targets

| Component | Cases | Status |
|-----------|-------|--------|
| read tool | 25+ | 10 done |
| write tool | 15+ | 7 done |
| edit tool | 20+ | 6 done |
| bash tool | 30+ | 12 done |
| grep tool | 25+ | 12 done |
| find tool | 15+ | 6 done |
| ls tool | 15+ | 8 done |
| truncation | 20+ | 9 done |
| SSE parsing | 30+ | 11 done |
| Session format | 25+ | 0 done |
| Provider responses | 20+ | 0 done |
| **Total** | **240+** | **81 done** |

---

## Part 4: Architecture Diagram

```
┌────────────────────────────────────────────────────────────────────┐
│                          pi CLI Binary                              │
├────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────────────────┐│
│  │   CLI       │───▶│   Config    │───▶│      Agent Loop         ││
│  │  (clap)     │    │   Loader    │    │   (tool iteration)      ││
│  └─────────────┘    └─────────────┘    └───────────┬─────────────┘│
│                                                     │              │
│  ┌──────────────────────────────────────────────────┼──────────────┤
│  │                    TUI Layer (charmed_rust)      │              │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────▼───────────┐ │
│  │  │   Editor    │  │   Status    │  │     Message Display     │ │
│  │  │ (bubbles)   │  │   Line      │  │  (glamour markdown)     │ │
│  │  └─────────────┘  └─────────────┘  └─────────────────────────┘ │
│  │  ┌─────────────┐  ┌─────────────┐                              │
│  │  │  Thinking   │  │   Slash     │                              │
│  │  │   Block     │  │  Commands   │                              │
│  │  └─────────────┘  └─────────────┘                              │
│  └─────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────────┤
│  │                 Provider Layer (asupersync HTTP)                │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
│  │  │  Anthropic  │  │   OpenAI    │  │   Google    │             │
│  │  │  Provider   │  │  Provider   │  │  Provider   │             │
│  │  └─────────────┘  └─────────────┘  └─────────────┘             │
│  └─────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────────┤
│  │                    Tool Layer                                    │
│  │  ┌────┐ ┌────┐ ┌────┐ ┌─────┐ ┌────┐ ┌────┐ ┌──┐              │
│  │  │read│ │bash│ │edit│ │write│ │grep│ │find│ │ls│              │
│  │  └────┘ └────┘ └────┘ └─────┘ └────┘ └────┘ └──┘              │
│  └─────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────────┤
│  │              Session Layer (JSONL + SQLite index)               │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
│  │  │   JSONL     │  │   SQLite    │  │    Tree     │             │
│  │  │   Files     │  │   Index     │  │  Navigator  │             │
│  │  └─────────────┘  └─────────────┘  └─────────────┘             │
│  └─────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────────┤
│  │              Output Layer (rich_rust)                           │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
│  │  │  Console    │  │   Tables    │  │   Panels    │             │
│  │  │  (markup)   │  │  Progress   │  │  Spinners   │             │
│  │  └─────────────┘  └─────────────┘  └─────────────┘             │
│  └─────────────────────────────────────────────────────────────────┤
└────────────────────────────────────────────────────────────────────┘
```

---

## Part 5: Timeline Estimate

| Phase | Duration | Status | Dependencies |
|-------|----------|--------|--------------|
| Phase 1: Fix Issues | 1-2 days | ✅ Complete | None |
| Phase 2: Dependencies | 1 day | ✅ Complete | Phase 1 |
| Phase 3: Interactive TUI | 3-5 days | ✅ Complete | Phase 2 |
| Phase 4: Provider streaming (asupersync) | 2-3 days | ✅ Complete | Phase 2 |
| Phase 5: Providers | 2-3 days | ✅ Complete | Phase 4 |
| Phase 6: Sessions | 2 days | ✅ Mostly Complete | Phase 3 |
| Phase 7: Tool conformance | 2-3 days | ✅ Complete (see `FEATURE_PARITY.md`) | Phases 1-6 |
| Phase 8: Benchmarks | 1-2 days | ✅ Complete | Phase 7 |
| Phase 9: Polish | 1-2 days | 🔶 In Progress | Phase 8 |
| **Phase 10: Extensions** | **3-5 days** | 🔶 In Progress (`bd-btq`) | Phase 4 |
| **Phase 11: Themes** | **1-2 days** | 🔶 In Progress (`bd-22p`) | Phase 9 |
| **Remaining** | **~5-8 days** | | |

---

## Part 6: Success Criteria

### Functional Requirements

- ✅ Core parity status is tracked in `FEATURE_PARITY.md`
- 🔶 Extensions runtime parity is tracked in `bd-btq` (+ children)
- 🔶 Theme parity is tracked in `bd-22p` (+ `bd-qpm`, `bd-3d8`, `bd-ieym`)

### Performance Requirements

- ✅ Size/startup targets are tracked in README + `BENCHMARKS.md`
- 🔶 Extension performance targets + evidence tracked in `bd-20p` (benchmarks: `bd-1fg`)

### Quality Requirements

- ✅ `unsafe` is forbidden (`#![forbid(unsafe_code)]`)
- 🔶 Zero-clippy + full gates are enforced in CI / release workstream (`bd-gqtd`)
- 🔶 Rust API docs (rustdoc) tracked in `bd-14od`

### Conformance Requirements

- ✅ Tool fixtures exist and run in CI (see `FEATURE_PARITY.md` for counts)
- 🔶 Provider streaming conformance via VCR (no mocks): `bd-h7r`, `bd-gd1`
- ⬜ Session format migration fixtures/tests - deferred

---

## Part 7: Files to Create/Modify

### New Files

```
src/tui/mod.rs           # TUI module organization
src/tui/app.rs           # Main Model implementation
src/tui/input.rs         # Multi-line editor
src/tui/messages.rs      # Message display
src/tui/status.rs        # Status line
src/tui/thinking.rs      # Thinking block
src/tui/commands.rs      # Slash commands

src/http/mod.rs          # HTTP abstraction
src/http/client.rs       # asupersync HTTP client

src/providers/openai.rs  # OpenAI provider
src/providers/google.rs  # Google Gemini provider

src/session/index.rs     # SQLite session index
src/session/tree.rs      # Tree navigation

benches/startup.rs       # Startup benchmark
benches/tui_render.rs    # TUI benchmark
benches/tools.rs         # Tool benchmark
benches/streaming.rs     # SSE benchmark

tests/conformance/reference/    # Reference implementations
tests/conformance/fixtures/     # Expanded fixtures
```

### Files to Modify

```
Cargo.toml               # Add charmed_rust, update asupersync features
src/main.rs              # Wire up TUI
src/agent.rs             # Use AgentCx
src/providers/anthropic.rs # Migrate to asupersync HTTP
src/session.rs           # Add SQLite index, tree navigation
src/tools.rs             # Fix detail field serialization
FEATURE_PARITY.md        # Update as features complete
```

---

## Immediate Next Steps

1. ~~**Fix the failing fixture tests** - detail field serialization~~ ✅ Done
2. ~~**Add charmed_rust dependencies** to Cargo.toml~~ ✅ Done
3. ~~**Create TUI module structure** with basic Model implementation~~ ✅ Done
4. ~~**Implement multi-line editor** using bubbles TextInput~~ ✅ Done
5. ~~**Run `cargo check`** to verify everything compiles~~ ✅ Done

**Current Priorities:**

Use Beads for the live queue:
- `bv --robot-plan`
- `bv --robot-priority`
- `br ready`

High-level workstreams:
1. **Extensions runtime** - `bd-btq` (discovery: `bd-1e0`)
2. **Themes** - `bd-22p` (apply colors: `bd-qpm`)
3. **asupersync capability hardening** - `bd-3i7u`, `bd-1xf`
4. **VCR test infrastructure** - `bd-30u`, `bd-h7r`, `bd-gd1`
