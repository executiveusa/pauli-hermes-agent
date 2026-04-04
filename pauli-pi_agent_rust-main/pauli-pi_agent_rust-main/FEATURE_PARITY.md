# Feature Parity: pi_agent_rust vs Pi Agent (TypeScript)

> **Purpose:** Authoritative single-source-of-truth for implementation status.
> **Last Updated:** 2026-02-18 (implementation snapshot refresh)
> **Release Claim Guardrail:** This document is progress evidence only. Strict drop-in replacement language is blocked unless `docs/dropin-certification-verdict.json` reports `overall_verdict = CERTIFIED`.

## Status Legend

| Status | Meaning |
|--------|---------|
| ✅ Implemented | Feature exists, covered by tests |
| 🔶 Partial | Some functionality present, known gaps remain |
| ❌ Missing | In scope but not yet implemented |
| ⬜ Out of Scope | Intentionally excluded from this port |

---

## Executive Summary

| Category | Implemented | Partial | Missing | Out of Scope | Total |
|----------|-------------|---------|---------|--------------|-------|
| **Core Types** | 8 | 0 | 0 | 0 | 8 |
| **Provider Layer** | 18 | 0 | 0 | 9 | 27 |
| **Tools (7 total)** | 7 | 0 | 0 | 0 | 7 |
| **Agent Runtime** | 7 | 0 | 0 | 0 | 7 |
| **Session Management** | 10 | 0 | 0 | 0 | 10 |
| **CLI** | 10 | 0 | 0 | 0 | 10 |
| **Resources & Customization** | 8 | 0 | 0 | 0 | 8 |
| **Extensions Runtime** | 12 | 0 | 0 | 0 | 12 |
| **TUI** | 18 | 0 | 0 | 2 | 20 |
| **Configuration** | 9 | 0 | 0 | 0 | 9 |
| **Authentication** | 8 | 0 | 0 | 0 | 8 |

---

## 1. Core Types (Message/Content/Usage)

| Feature | Status | Rust Location | Tests | Notes |
|---------|--------|---------------|-------|-------|
| Message union (User/Assistant/ToolResult) | ✅ | `src/model.rs:13-19` | Unit | Complete enum with serde |
| UserMessage | ✅ | `src/model.rs:22-27` | Unit | Text or Blocks content |
| AssistantMessage | ✅ | `src/model.rs:38-50` | Unit | Full metadata |
| ToolResultMessage | ✅ | `src/model.rs:53-63` | Unit | Error flag, details |
| ContentBlock enum | ✅ | `src/model.rs:86-93` | Unit | Text/Thinking/Image/ToolCall |
| StopReason enum | ✅ | `src/model.rs:70-79` | Unit | All 5 variants |
| Usage tracking | ✅ | `src/model.rs:145-166` | Unit | Input/output/cache/cost |
| StreamEvent enum | ✅ | `src/model.rs:172-232` | Unit | All 12 event types |

---

## 2. Provider Layer

### 2.1 Provider Trait

| Feature | Status | Rust Location | Tests | Notes |
|---------|--------|---------------|-------|-------|
| Provider trait definition | ✅ | `src/provider.rs:18-31` | - | async_trait based |
| Context struct | ✅ | `src/provider.rs:38-43` | - | System prompt + messages + tools |
| StreamOptions | ✅ | `src/provider.rs:62-72` | - | Temperature, max_tokens, thinking |
| ToolDef struct | ✅ | `src/provider.rs:49-55` | - | JSON Schema parameters |
| Model definition | ✅ | `src/provider.rs:108-121` | - | Cost, context window, etc. |
| ThinkingLevel enum | ✅ | `src/model.rs:239-265` | Unit | 6 levels with budgets |
| CacheRetention enum | ✅ | `src/provider.rs:75-81` | - | None/Short/Long |

### 2.2 Provider Implementations

| Provider | Status | Rust Location | Tests | Notes |
|----------|--------|---------------|-------|-------|
| **Anthropic** | ✅ | `src/providers/anthropic.rs` | Unit | Full streaming + thinking + tools |
| **OpenAI** | ✅ | `src/providers/openai.rs` | Unit | Full streaming + tool use |
| **Google Gemini** | ✅ | `src/providers/gemini.rs` | 4 | Full streaming + tool use |
| **Azure OpenAI** | ✅ | `src/providers/azure.rs` | 4 | Full streaming + tool use |
| Amazon Bedrock | ⬜ | - | - | Low priority |
| Google Vertex | ⬜ | - | - | Low priority |
| GitHub Copilot | ⬜ | - | - | OAuth complexity |
| XAI | ⬜ | - | - | Low priority |
| Groq | ⬜ | - | - | Low priority |
| Cerebras | ⬜ | - | - | Low priority |
| OpenRouter | ⬜ | - | - | Low priority |
| Mistral | ⬜ | - | - | Low priority |
| Custom providers | ⬜ | - | - | Defer |

### 2.3 Streaming Implementation

| Feature | Status | Location | Notes |
|---------|--------|----------|-------|
| SSE parsing (Anthropic) | ✅ | `anthropic.rs` | `asupersync` HTTP stream (`src/http/client.rs`) + `src/sse.rs` |
| SSE parser module | ✅ | `src/sse.rs` | Custom parser for asupersync migration |
| Text delta streaming | ✅ | `anthropic.rs:339-352` | Real-time text |
| Thinking delta streaming | ✅ | `anthropic.rs:354-367` | Extended thinking |
| Tool call streaming | ✅ | `anthropic.rs:368-382` | JSON accumulation |
| Usage updates | ✅ | `anthropic.rs:430-448` | Token counts |
| Error event handling | ✅ | `anthropic.rs:258-266` | API errors |

---

## 3. Built-in Tools

| Tool | Status | Rust Location | Tests | Conformance Tests |
|------|--------|---------------|-------|-------------------|
| **read** | ✅ | `src/tools.rs` | 4 | ✅ test_read_* |
| **bash** | ✅ | `src/tools.rs` | 3 | ✅ test_bash_* |
| **edit** | ✅ | `src/tools.rs` | 3 | ✅ test_edit_* |
| **write** | ✅ | `src/tools.rs` | 2 | ✅ test_write_* |
| **grep** | ✅ | `src/tools.rs` | 3 | ✅ test_grep_* |
| **find** | ✅ | `src/tools.rs` | 2 | ✅ test_find_* |
| **ls** | ✅ | `src/tools.rs` | 3 | ✅ test_ls_* |

### 3.1 Tool Feature Details

| Feature | read | bash | edit | write | grep | find | ls |
|---------|------|------|------|-------|------|------|-----|
| Basic operation | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Truncation (head/tail) | ✅ | ✅ | - | - | ✅ | ✅ | ✅ |
| Image support | ✅ | - | - | - | - | - | - |
| Streaming updates | - | ✅ | - | - | - | - | - |
| Line numbers | ✅ | - | - | - | ✅ | - | - |
| Fuzzy matching | - | - | ✅ | - | - | - | - |
| Path resolution | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| ~ expansion | ✅ | - | ✅ | ✅ | ✅ | ✅ | ✅ |
| macOS screenshot paths | ✅ | - | - | - | - | - | - |

### 3.2 Truncation Constants

| Constant | Value | Used By |
|----------|-------|---------|
| DEFAULT_MAX_LINES | 2000 | read, bash, grep |
| DEFAULT_MAX_BYTES | 50KB | read, bash, grep, find, ls |
| GREP_MAX_LINE_LENGTH | 500 | grep |

---

## 4. Agent Runtime

| Feature | Status | Rust Location | Tests | Notes |
|---------|--------|---------------|-------|-------|
| Agent struct | ✅ | `src/agent.rs` | Unit | Provider + tools + config |
| Agent loop | ✅ | `src/agent.rs` | - | Tool iteration limit |
| Tool execution | ✅ | `src/agent.rs` | Unit | Error handling |
| Event callbacks | ✅ | `src/agent.rs` | - | 9 event types |
| Stream processing | ✅ | `src/agent.rs` | - | Delta handling |
| Context building | ✅ | `src/agent.rs` | - | System + history + tools |
| Abort handling | ✅ | `src/agent.rs`, `src/main.rs`, `src/interactive.rs` | - | Ctrl+C cancels in-flight requests |

---

## 5. Session Management

| Feature | Status | Rust Location | Tests | Notes |
|---------|--------|---------------|-------|-------|
| Session struct | ✅ | `src/session.rs` | - | Header + entries + path |
| SessionHeader | ✅ | `src/session.rs` | - | Version 3 |
| JSONL persistence | ✅ | `src/session.rs` | - | Save/load |
| Entry types (7) | ✅ | `src/session.rs` | - | Message, ModelChange, etc. |
| Tree structure | ✅ | `src/session.rs` | 7 | Full parent/child navigation |
| CWD encoding | ✅ | `src/session.rs` | 1 | Session directory naming |
| Entry ID generation | ✅ | `src/session.rs` | - | 8-char hex |
| Continue previous | ✅ | `src/session.rs` | - | Most recent by mtime |
| Session picker UI | ✅ | `src/session_picker.rs` | 3 | TUI picker with bubbletea |
| Branching/navigation | ✅ | `src/session.rs` | 7 | navigate_to, create_branch_from, list_leaves, branch_summary |

---

## 6. CLI

| Feature | Status | Rust Location | Tests | Notes |
|---------|--------|---------------|-------|-------|
| Argument parsing | ✅ | `src/cli.rs` | - | Clap derive |
| Subcommands | ✅ | `src/cli.rs`, `src/main.rs` | - | Install, Remove, Update, List, Config |
| @file arguments | ✅ | `src/cli.rs` | - | File inclusion |
| Message arguments | ✅ | `src/cli.rs` | - | Positional text |
| Tool selection | ✅ | `src/cli.rs` | - | --tools flag |
| Model listing | ✅ | `src/main.rs` | - | Table output |
| Session export | ✅ | `src/main.rs` | - | HTML export |
| Print mode | ✅ | `src/main.rs` | - | Single-shot mode |
| RPC mode | ✅ | `src/main.rs`, `src/rpc.rs` | `tests/rpc_mode.rs` | Headless stdin/stdout JSON protocol (prompt/steer/follow_up/state/stats/model/thinking/compact/bash/fork) |
| Package management | ✅ | `src/package_manager.rs`, `src/main.rs` | Unit | install/remove/update/list + settings updates + startup auto-install + resource resolution |

---

## 6A. Resources & Customization

| Feature | Status | Rust Location | Tests | Notes |
|---------|--------|---------------|-------|-------|
| Skills loader + validation | ✅ | `src/resources.rs` | Unit | Agent Skills frontmatter + diagnostics |
| Skills prompt inclusion | ✅ | `src/main.rs` | Unit | Appends `<available_skills>` if `read` tool enabled |
| Skill command expansion (`/skill:name`) | ✅ | `src/resources.rs`, `src/interactive.rs` | Unit | Expands to `<skill ...>` block |
| Prompt template loader | ✅ | `src/resources.rs` | Unit | Global/project + explicit paths |
| Prompt template expansion (`/name args`) | ✅ | `src/resources.rs`, `src/interactive.rs` | Unit | `$1`, `$@`, `$ARGUMENTS`, `${@:N}` |
| Package resource discovery | ✅ | `src/resources.rs` | Unit | Reads `package.json` `pi` field or defaults |
| Themes discovery | ✅ | `src/theme.rs`, `src/interactive.rs` | Unit + `tests/tui_state.rs` | Loader + /theme switching |
| Themes hot reload | ✅ | `src/interactive.rs` | `tests/tui_state.rs` | `/reload` re-resolves and reapplies current theme |

## 6B. Extensions Runtime

| Feature | Status | Rust Location | Tests | Notes |
|---------|--------|---------------|-------|-------|
| Extension discovery (paths + packages) | ✅ | `src/package_manager.rs`, `src/resources.rs` | Unit | Resolves `extensions/` sources from settings/auto-discovery/packages/CLI |
| Extension protocol (v1) + JSON schema | ✅ | `src/extensions.rs`, `docs/schema/extension_protocol.json` | Unit + `tests/extensions_manifest.rs` | `ExtensionMessage::parse_and_validate` + schema compilation tests |
| Compatibility scanner (Node API audit) | ✅ | `src/extensions.rs`, `src/package_manager.rs` | `tests/ext_conformance_artifacts.rs` | Emits compat ledgers when `PI_EXT_COMPAT_SCAN` is enabled |
| Capability manifest + policy | ✅ | `src/extensions.rs` | Unit + `tests/extensions_manifest.rs` | `strict/prompt/permissive` + scoped manifests (`pi.ext.cap.v1`) |
| FS connector (scoped, anti-escape) | ✅ | `src/extensions.rs` | Unit | Path traversal + symlink escape hardening |
| HTTP connector (policy-gated) | ✅ | `src/connectors/http.rs` | Unit | TLS/allowlist/denylist/size/timeouts |
| PiJS runtime (QuickJS) | ✅ | `src/extensions_js.rs` | Unit + `tests/event_loop_conformance.rs` | Deterministic scheduler + Promise bridge + budgets/timeouts |
| Promise hostcall bridge (pi.* → queue → completion) | ✅ | `src/extensions_js.rs` | Unit | `pi.tool/exec/http/session/ui/events` + `setTimeout/clearTimeout` |
| Hostcall ABI (host_call/host_result protocol) | ✅ | `src/extensions.rs` | Unit | Protocol types + validation exist; end-to-end dispatch wired |
| Extension UI bridge (select/confirm/input/editor) | ✅ | `src/extensions.rs`, `src/interactive.rs`, `src/rpc.rs` | Unit | UI request/response plumbing exists; runtime dispatch wired |
| Extension session API (get_state/messages/set_name) | ✅ | `src/extensions.rs`, `src/interactive.rs` | - | Trait + interactive impl exist; runtime dispatch wired |
| JS extension execution + registration (tools/commands/hooks) | ✅ | `src/extensions_js.rs`, `src/extension_dispatcher.rs`, `src/agent.rs`, `src/interactive.rs` | Unit + E2E | QuickJS runtime loads JS/TS extensions and supports tool/command registration + execution + event hooks |

---

## 7. Configuration

| Feature | Status | Rust Location | Tests | Notes |
|---------|--------|---------------|-------|-------|
| Config loading | ✅ | `src/config.rs` | - | Global + project merge |
| Settings struct | ✅ | `src/config.rs` | - | All fields optional |
| Default accessors | ✅ | `src/config.rs` | - | Fallback values |
| Compaction settings | ✅ | `src/config.rs` | - | enabled, reserve, keep |
| Retry settings | ✅ | `src/config.rs` | - | enabled, max, delays |
| Image settings | ✅ | `src/config.rs` | - | auto_resize, block |
| Terminal settings | ✅ | `src/config.rs` | - | show_images, clear |
| Thinking budgets | ✅ | `src/config.rs` | - | Per-level overrides |
| Environment variables | ✅ | `src/config.rs` | - | PI_CONFIG_PATH/PI_CODING_AGENT_DIR/PI_PACKAGE_DIR/PI_SESSIONS_DIR + provider API keys |

---

## 8. Terminal UI

### 8.1 Non-Interactive Output (rich_rust)

| Feature | Status | Rust Location | Tests | Notes |
|---------|--------|---------------|-------|-------|
| PiConsole wrapper | ✅ | `src/tui.rs` | 3 | rich_rust integration |
| Styled output (markup) | ✅ | `src/tui.rs` | - | Colors, bold, dim |
| Agent event rendering | ✅ | `src/tui.rs` | - | Text, thinking, tools, errors |
| Table rendering | ✅ | `src/tui.rs` | - | Via rich_rust Tables |
| Panel rendering | ✅ | `src/tui.rs` | - | Via rich_rust Panels |
| Rule rendering | ✅ | `src/tui.rs` | - | Horizontal dividers |
| Spinner styles | ✅ | `src/tui.rs` | 1 | Dots, line, simple |

### 8.2 Interactive TUI (charmed_rust/bubbletea)

| Feature | Status | Rust Location | Tests | Notes |
|---------|--------|---------------|-------|-------|
| PiApp Model | ✅ | `src/interactive.rs` | 296+ | Elm Architecture (296 tui_state + 226 lib unit tests) |
| TextInput with history | ✅ | `src/interactive.rs` | - | bubbles TextInput |
| Markdown rendering | ✅ | `src/interactive.rs` | - | glamour Dark style |
| Token/cost footer | ✅ | `src/interactive.rs` | - | Usage tracking |
| Spinner animation | ✅ | `src/interactive.rs` | - | bubbles spinner |
| Tool status display | ✅ | `src/interactive.rs` | - | Running tool indicator |
| Keyboard navigation | ✅ | `src/interactive.rs` | - | Up/Down history, Esc quit |
| Agent integration | ✅ | `src/interactive.rs` | - | Agent events wired; CLI interactive uses PiApp |
| Multi-line editor | ✅ | `src/interactive.rs` | - | TextArea with line wrapping |
| Slash command system | ✅ | `src/interactive.rs` | - | /help, /login, /logout, /clear, /model, /thinking, /exit, /history, /export, /session, /resume, /new, /copy, /name, /hotkeys |
| Viewport scrolling | ✅ | `src/interactive.rs` | - | Viewport with scroll_to_bottom() |
| Image display | ⬜ | - | - | Terminal dependent |
| Autocomplete | ✅ | `src/autocomplete.rs`, `src/interactive.rs` | `tests/tui_state.rs` | Tab-triggered dropdown + path completion |

### 8.3 Interactive Commands (Slash)

| Command | Status | Rust Location | Notes |
|---------|--------|---------------|-------|
| `/help` | ✅ | `src/interactive.rs` | Help text |
| `/clear` | ✅ | `src/interactive.rs` | Clears in-memory conversation view |
| `/model` | ✅ | `src/interactive.rs` | Switch model/provider |
| `/thinking` | ✅ | `src/interactive.rs` | Set thinking level |
| `/history` | ✅ | `src/interactive.rs` | Show input history |
| `/export` | ✅ | `src/interactive.rs` | Export session to HTML |
| `/exit` / `/quit` | ✅ | `src/interactive.rs` | Exit Pi |
| `/login` | ✅ | `src/interactive/commands.rs`, `src/auth.rs` | Anthropic OAuth + OpenAI/Google API key + extension OAuth |
| `/logout` | ✅ | `src/interactive.rs`, `src/auth.rs` | Remove stored credentials |
| `/session` | ✅ | `src/interactive.rs` | Show session info (path/tokens/cost) |
| `/resume` | ✅ | `src/interactive.rs` | Session picker overlay (deletion disabled) |
| `/new` | ✅ | `src/interactive.rs` | Start new in-memory session |
| `/name <name>` | ✅ | `src/interactive.rs` | Set session display name |
| `/copy` | ✅ | `src/interactive.rs` | Clipboard support is feature-gated (`--features clipboard`) |
| `/hotkeys` | ✅ | `src/interactive.rs` | Show keybindings |
| `/scoped-models` | ✅ | `src/interactive/commands.rs` | Pattern matching + persistence to project settings |
| `/settings` | ✅ | `src/interactive.rs` | Shows effective settings + resource counts |
| `/tree` | ✅ | `src/interactive.rs` | List leaves and switch branch by id/index |
| `/fork` | ✅ | `src/interactive.rs` | Forks new session file from user message |
| `/compact [prompt]` | ✅ | `src/interactive.rs`, `src/compaction.rs` | Manual compaction |
| `/share` | ✅ | `src/interactive/share.rs` | HTML export + GitHub Gist upload via `gh` CLI |
| `/reload` | ✅ | `src/interactive.rs`, `src/resources.rs` | Reloads skills/prompts/themes + refreshes autocomplete |
| `/changelog` | ✅ | `src/interactive.rs` | Display changelog entries |

---

## 9. Authentication

| Feature | Status | Rust Location | Tests | Notes |
|---------|--------|---------------|-------|-------|
| API key from env | ✅ | `src/auth.rs` | - | ANTHROPIC_API_KEY, etc. |
| API key from flag | ✅ | `src/main.rs` | - | --api-key |
| auth.json storage | ✅ | `src/auth.rs` | - | File with 0600 perms |
| File locking | ✅ | `src/auth.rs` | - | Exclusive lock with timeout |
| Key resolution | ✅ | `src/auth.rs` | - | override > auth.json > env |
| Multi-provider keys | ✅ | `src/auth.rs` | - | 12 providers supported |
| OAuth flow | ✅ | `src/auth.rs`, `src/interactive/commands.rs` | Unit | Anthropic PKCE + extension-registered providers |
| Token refresh | ✅ | `src/auth.rs`, `src/main.rs` | Unit | Auto-refresh on startup for all OAuth providers |

---

## 10. Error Handling

| Feature | Status | Rust Location | Tests | Notes |
|---------|--------|---------------|-------|-------|
| Error enum | ✅ | `src/error.rs` | - | thiserror based |
| Config errors | ✅ | `src/error.rs` | - | |
| Session errors | ✅ | `src/error.rs` | - | Including NotFound |
| Provider errors | ✅ | `src/error.rs` | - | Provider + message |
| Auth errors | ✅ | `src/error.rs` | - | |
| Tool errors | ✅ | `src/error.rs` | - | Tool name + message |
| Validation errors | ✅ | `src/error.rs` | - | |
| IO/JSON/HTTP errors | ✅ | `src/error.rs` | - | From impls |

---

## Test Coverage Summary

| Category | Unit Tests | Integration Tests | Fixture Cases | Total |
|----------|------------|-------------------|---------------|-------|
| Core types | 4 | 0 | 0 | 4 |
| Provider (Anthropic) | 2 | 0 | 0 | 2 |
| Provider (OpenAI) | 3 | 0 | 0 | 3 |
| Provider (Gemini) | 4 | 0 | 0 | 4 |
| Provider (Azure) | 4 | 0 | 0 | 4 |
| SSE parser | 11 | 0 | 0 | 11 |
| Tools | 5 | 20 | 122 | 147 |
| CLI flags (fixtures) | 0 | 0 | 17 | 17 |
| TUI (rich_rust) | 3 | 0 | 0 | 3 |
| TUI (interactive lib) | 226 | 0 | 0 | 226 |
| TUI (tui_state integration) | 0 | 296 | 0 | 296 |
| TUI (e2e_tui_perf) | 0 | 103 | 0 | 103 |
| TUI (session picker) | 3 | 0 | 0 | 3 |
| TUI (perf unit: FrameTiming/Cache/Buffers) | 47 | 0 | 0 | 47 |
| Session (branching) | 7 | 0 | 0 | 7 |
| Agent | 2 | 0 | 0 | 2 |
| Conformance infra | 6 | 0 | 0 | 6 |
| Extensions | 2 | 0 | 0 | 2 |
| Other lib tests | 2,800+ | 0 | 0 | 2,800+ |
| **Total (lib)** | **3,319** | - | - | **3,319** |
| **Total (all targets)** | **3,319+** | **399+** | **139** | **3,857+** |

**All tests pass** (`cargo test --lib`: 3,319 pass; `tui_state`: 296 pass; `e2e_tui_perf`: 103 pass)

---

## Conformance Testing Status

| Component | Has Fixture Tests | Fixture File | Cases | Status |
|-----------|-------------------|--------------|-------|--------|
| read tool | ✅ Yes | `read_tool.json` | 23 | ✅ All pass |
| write tool | ✅ Yes | `write_tool.json` | 7 | ✅ All pass |
| edit tool | ✅ Yes | `edit_tool.json` | 23 | ✅ All pass |
| bash tool | ✅ Yes | `bash_tool.json` | 34 | ✅ All pass |
| grep tool | ✅ Yes | `grep_tool.json` | 12 | ✅ All pass |
| find tool | ✅ Yes | `find_tool.json` | 6 | ✅ All pass |
| ls tool | ✅ Yes | `ls_tool.json` | 8 | ✅ All pass |
| truncation | ✅ Yes | `truncation.json` | 9 | ✅ All pass |
| Session format | ✅ Yes | `tests/session_conformance.rs` | 28 | ✅ All pass |
| Provider responses | ✅ Yes | `tests/provider_streaming.rs` | 4 | ✅ All pass (VCR) |
| CLI flags | ✅ Yes | `cli_flags.json` | 17 | ✅ All pass |
| **Total** | **11/11** | - | **171** | ✅ |

### Fixture Schema

Fixtures are JSON files in `tests/conformance/fixtures/` with this structure:

```json
{
  "version": "1.0",
  "tool": "tool_name",
  "cases": [
    {
      "name": "test_name",
      "setup": [{"type": "create_file", "path": "...", "content": "..."}],
      "input": {"param": "value"},
      "expected": {
        "content_contains": ["..."],
        "content_regex": "...",
        "details_exact": {"key": "value"}
      }
    }
  ]
}
```

---

## Performance Targets

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| Startup time | <100ms | 13ms (`pi --version`) | ✅ |
| Binary size (release) | <20MB | 8.3MB | ✅ |
| TUI framerate | 60fps | Instrumented (PERF-3: frame timing telemetry) | ✅ |
| Frame budget | <16ms | Enforced (PERF-4: auto-degrades when exceeded) | ✅ |
| Memory (idle) | <50MB | Monitored (PERF-6: RSS-based pressure detection) | ✅ |

### Performance Features (PERF Track — Complete)

| Feature | Bead | Description | Status |
|---------|------|-------------|--------|
| Message render cache | PERF-1 | Per-message memoization with generation-based invalidation | ✅ |
| Incremental prefix | PERF-2 | Streaming fast path: cached prefix + append-only tail | ✅ |
| Frame timing telemetry | PERF-3 | Microsecond-precision instrumentation of view()/update() | ✅ |
| Frame budget + degradation | PERF-4 | Auto-degrade rendering when frames exceed 16ms budget | ✅ |
| Memory pressure detection | PERF-6 | RSS monitoring, progressive collapse at thresholds | ✅ |
| Buffer pre-allocation | PERF-7 | Reusable render buffers, capacity hints, zero-copy paths | ✅ |
| Criterion benchmarks | PERF-8 | Benchmark suite for all critical rendering paths | ✅ |
| CI regression gate | PERF-9 | Fail CI on >20% performance regression | ✅ |
| Cross-platform fallbacks | PERF-CROSS | Graceful degradation when /proc unavailable (macOS/Windows) | ✅ |

---

## Next Steps (Priority Order)

1. ~~**Complete print mode** - Non-interactive single response~~ ✅ Done
2. ~~**Add OpenAI provider** - Second provider implementation~~ ✅ Done
3. ~~**Implement auth.json** - Credential storage~~ ✅ Done (src/auth.rs)
4. ~~**Session picker UI** - Basic TUI for --resume~~ ✅ Done (src/session_picker.rs)
5. ~~**Branching/navigation** - Tree operations~~ ✅ Done (src/session.rs)
6. ~~**Benchmark harness** - Performance validation~~ ✅ Done (benches/tools.rs, BENCHMARKS.md)
7. ~~**Conformance fixtures** - TypeScript reference capture~~ ✅ Done (tests/conformance/)
