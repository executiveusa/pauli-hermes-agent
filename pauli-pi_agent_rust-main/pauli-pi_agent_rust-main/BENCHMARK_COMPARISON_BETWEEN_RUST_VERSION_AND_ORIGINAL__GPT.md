# BENCHMARK_COMPARISON_BETWEEN_RUST_VERSION_AND_ORIGINAL__GPT

Generated: 2026-02-19
Workspace: `/data/projects/pi_agent_rust`

## 0) Post-Hardening Status Update (2026-02-17)

This report now includes a post-hardening extension-compatibility checkpoint.

- Extension conformance matrix is now fully green locally: `224/224` passed, `0` failed, `0` skipped.
- This supersedes earlier partial-compatibility snapshots in this document that referenced `223` corpus entries with non-zero failures.
- Validation commands run in this update cycle:
  - `cargo test --test ext_conformance_generated --features ext-conformance -- conformance_sharded_matrix --nocapture --exact`
  - `cargo check --all-targets`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt --check`

## 0.1) Post-Compaction Secure-Path Verification (2026-02-18)

- Security/perf guardrail: feature/security defaults were restored and preserved (no benchmark-goosing default flips):
  - `DUAL_EXEC_DEFAULT_SAMPLE_PPM` restored to original default (`25_000`) in `src/extension_dispatcher.rs`.
  - AMAC `from_env()` default behavior restored to enabled-by-default semantics in `src/hostcall_amac.rs`.
- Fast-path optimization in dispatcher remains security-preserving:
  - Capability deny checks still execute before hostcall completion on the fast path.
  - No policy/risk/quota controls were removed.
- Native runtime hot-path cleanup landed in active scaffold (`src/extensions.rs`):
  - lock-scope tightening and drop timing fixes for `RwLock` guards,
  - allocation/clone reductions in tool/command/shortcut/provider-stream flows,
  - clippy-driven correctness/style rewrites (`option_if_let_else`, `manual_let_else`, `significant_drop_tightening`).
- Gate status from this pass:
  - `cargo fmt --check` ✅
  - `cargo check --all-targets` ✅
  - `cargo clippy --all-targets -- -D warnings` ✅
  - `cargo test --test event_loop_conformance --test lab_runtime_extensions --test extensions_event_wiring` ✅ (`12 + 15 + 133` tests passed)

## 0.2) Live-Provider Release-Binary Checkpoint (2026-02-19)

- Dev-firstset gate (debug binaries, required before any new release build in this flow):
  - artifact: `tests/ext_conformance/reports/release_binary_e2e/ollama_firstset_dev_20260219_jobs10_timeout600.json`
  - run: `release-e2e-20260219T032439Z`
  - provider/model: `ollama` / `qwen2.5:0.5b`
  - result: `20/20` pass (`0` fail, `0` timeout)
- Full optimized release-binary sweep (after dev gate passed):
  - artifact: `tests/ext_conformance/reports/release_binary_e2e/ollama_full_release_20260219_jobs10_timeout600.json`
  - run: `release-e2e-20260219T033502Z`
  - provider/model: `ollama` / `qwen2.5:0.5b`
  - result: `224/224` pass (`0` fail, `0` timeout)
- This lane is compatibility-focused (real `pi` binary + live provider path) rather than paid-provider throughput benchmarking.

## 0.3) Full Perf Orchestrator Checkpoint (2026-02-19)

- Full-suite orchestration run:
  - command: `./scripts/perf/orchestrate.sh --profile full --skip-build --no-rch --output-dir /data/tmp/pi_agent_rust/codex/perf/full_local_skipbuild_retry_20260219T0650Z`
  - correlation: `fullbench-local-skipbuild-retry-20260219T0650Z`
  - manifest: `/data/tmp/pi_agent_rust/codex/perf/full_local_skipbuild_retry_20260219T0650Z/manifest.json`
- Run summary:
  - suites: `11` total, `9` pass, `2` fail, `0` skip
  - duration: `1,601,650ms`
  - passing suites:
    - `perf_comparison`
    - `perf_bench_harness`
    - `perf_baseline_variance`
    - `ext_bench_harness`
    - `bench_schema`
    - `bench_scenario`
    - `criterion_tools`
    - `criterion_system`
    - `criterion_extensions`
  - failing suites:
    - `perf_budgets` (`exit=101`): strict fail-closed budget contract due missing/stale evidence artifacts at expected canonical paths (criterion/pijs/release-binary inputs).
    - `perf_regression` (`exit=101`): `binary_size_check` failed because release binary path was not present for strict mode (`/data/tmp/pi_agent_rust/codex/perf/release/pi`).
- Interpretation:
  - The two failures are evidence-path/precondition failures, not a demonstrated runtime-latency regression.
  - In the same run, the measured startup guards in `perf_regression` remained green (`--help` P95 `3.8ms`, `--version` P95 `3.6ms`).

## 1) Lede (Do Not Bury This)

1. Fresh secure-path reruns (2026-02-18) plus a full orchestrator checkpoint (2026-02-19) continue to show the same trendline: against the last validated legacy baselines, Rust wins the measured `1M`/`5M` matched-state and realistic workload totals in this report’s harness.
2. Rust remains **much smaller in memory footprint** in matched-state and realistic flows, with substantial RSS advantages retained.
3. Extension compatibility is fully passing in both local matrix conformance and live-provider release-binary execution:
   - Matrix: `224/224` pass (`0` fail, `0` skipped)
   - Release-binary E2E (ollama, optimized binaries): `224/224` pass (`0` fail, `0` timeout)
4. Rust has significantly expanded first-class capability surface versus legacy coding-agent CLI (commands, policy explainers, provider metadata/control, risk/quota/security instrumentation).
5. The largest practical optimization target remains session append/save behavior at high token-volume and large histories; this is the best lever for major speed gains.
6. Startup/readiness latency strongly favors Rust in this snapshot: fresh Rust `--help`/`--version` means are ~`3.02ms`/`2.77ms`; direct legacy reruns are currently blocked, but prior validated legacy means remain ~`1.0s` (Node) and ~`0.73s` (Bun).
7. Extension micro-harness inversion remains achieved on the real native runtime lane: in fresh `pijs_workload` release runs, native runtime is `~17.74x` faster per call than QuickJS (`0.4678us` vs `8.2980us`).

## 1.1) Refresh Delta (2026-02-18)

Freshly re-measured in this run:
- `pijs_workload` release 3-lane comparison (`quickjs`, `native-rust-runtime`, `native-rust-preview`) at `50,000` iterations
- Rust release startup/readiness (`hyperfine` for `--help` and `--version`) and one-shot `/usr/bin/time` footprints
- Rust long-session benchmarks via `session_workload_bench`:
  - matched-state (`prepare` + `workload`) at `1M` and `5M` target tokens
  - realistic (`prepare-realistic` + `workload-realistic`) at `1M` and `5M` target tokens
- Rust extension workload matrix via `ext_workloads` with hotspot matrix + trace artifacts
- targeted extension/runtime suites (`event_loop_conformance`, `extensions_event_wiring`, `lab_runtime_extensions`)
- local strict quality gates (`cargo fmt --check`, `cargo check --all-targets`, `cargo clippy --all-targets -- -D warnings`)

Reused (existing in-repo evidence, unchanged methodology):
- LOC and callable inventory (Rust + legacy scopes)
- CLI diff (`pi --help` vs legacy `dist/cli.js --help`)
- provider ID diff (Rust canonical table vs legacy runtime provider registry)
- cross-runtime startup compare tables (Node/Bun) from prior validated run
- one-shot startup footprint snapshots (`/usr/bin/time` RSS/user/sys)
- long-session realistic latency matrix
- matched-state 10-message append footprint matrix
- realistic 1M/5M footprint matrix
- extension workload microbench (`ext_workloads` and `bench_legacy_extension_workloads.mjs`)
- historical 223-extension vendored conformance + failure taxonomy (kept for baseline context; superseded by current `224/224` matrix status above)

Build/regeneration note:
- `cargo build --release --bin pi` succeeds in this run and was used for fresh Rust startup numbers.
- Direct legacy Node/Bun reruns in this workspace are currently blocked by missing dependency surfaces in `legacy_pi_mono_code/pi-mono`:
  - Node startup: `ERR_MODULE_NOT_FOUND` for `chalk`
  - Bun startup: missing `node_modules/chalk`
  - legacy extension workloads (Node): missing `@mariozechner/jiti`
  - legacy extension workloads (Bun): missing `@sinclair/typebox` / `proper-lockfile`
  so legacy comparison rows continue to use prior validated artifacts where direct reruns were not possible.

---

## 2) Scope and Comparison Modes

### 2.1 Apples-to-Apples Scope
- Rust target: this repo (`pi_agent_rust`)
- Legacy target: `legacy_pi_mono_code/pi-mono/packages/coding-agent`

### 2.2 Apples-to-Oranges Scope (Full Legacy Runtime Context)
- Legacy aggregate target: `packages/{ai,agent,coding-agent,tui}`
- Purpose: include behavior that legacy offloads into sibling packages (provider stack, UI/runtime services), so comparisons are not unfairly narrow.

### 2.3 Benchmarks Included
- Matched-state long-session benchmark: resume + append the same 10 user messages.
- Realistic E2E benchmark: resume + append + extension-like activity + slash-like state changes + forking + exports + compactions.
- Extension microbench: real extension loading and real tool/event dispatch.
- Extension corpus conformance: full vendored/unvendored compatibility reports.
- Release-binary live-provider extension E2E: dev-firstset gate (`20` cases) followed by full optimized sweep (`224` cases).
- These suites are intended to function as a practical system-level regression harness, not just synthetic microbench snapshots.

### 2.4 Provider API Cost Control
- This report does **not** use paid external API calls for the benchmark matrices.
- It does include live-provider extension compatibility runs using local `ollama` (`qwen2.5:0.5b`) to validate non-mocked runtime behavior.
- No paid-provider throughput benchmark is included here.
- If provider-call throughput benchmarks are added, use `ollama` first for cost control.

---

## 3) Codebase Scale and Complexity

## 3.1 LOC (Production vs Test)

Method: `tokei` scoped by language (`Rust` for Rust repo, `TypeScript` for legacy scopes).

| Scope | Production LOC | Test LOC |
|---|---:|---:|
| Rust (`src`, Rust only) | 224,348 | 224,212 (`tests`, Rust only) |
| Legacy coding-agent only (`src/test`, TS only) | 27,412 | 8,871 |
| Legacy full stack (`ai+agent+coding-agent+tui`, TS only) | 55,313 | 21,779 |

Ratios:
- Rust vs legacy coding-agent: prod `8.18x`, test `25.27x`
- Rust vs legacy full-stack: prod `4.06x`, test `10.29x`

## 3.2 Function/Callable Inventory

Method note:
- Rust callable count here uses `fn` token signature inventory (`\\bfn\\s+...`) plus test attribute inventory; this remains approximate for macros/trait forms.
- Legacy callable count here uses TypeScript AST traversal (function declarations, methods, constructors, accessors, variable-assigned arrow/function expressions); still an approximation of executable behavior.

Rust (signature inventory):
- `src` function signatures: `10,417`
- `tests` function signatures: `9,459`
- test attributes total: `11,976` (`src=5,474`, `tests=6,502`)

Legacy AST callable inventory:
- coding-agent `src`: `1,315`
- coding-agent `test`: `93`
- full stack `src`: `1,907`
- full stack `test`: `247`

## 3.3 Test Coverage Baseline (Rust)

From `docs/coverage-baseline-map.json`:
- Line coverage: `79.08%` (`95,706 / 121,018`)
- Function coverage: `78.01%` (`8,545 / 10,954`)
- Branch coverage: `51.95%` (documented lower-bound due llvm-cov export SIGSEGV on subset of files)

---

## 4) Verified Feature/Functionality Delta

This section lists **verified Rust-first-class surfaces** missing from legacy coding-agent CLI in this workspace snapshot.

## 4.1 CLI Surface Delta (Direct Help Diff)

Rust-only top-level commands:
- `doctor`
- `help`
- `info`
- `migrate`
- `search`
- `update-index`

Rust-only flags:
- `--extension-policy`
- `--explain-extension-policy`
- `--repair-policy`
- `--explain-repair-policy`
- `--list-providers`
- `--theme-path`
- `--session-durability`
- `--no-migrations`

Legacy-only flags:
- `--plan`

## 4.2 Rust-Only Major Capability Areas (with complexity hints)

| Capability area | Primary Rust implementation | Approx LOC | Approx fn count |
|---|---|---:|---:|
| Extension runtime + policy + host integration | `src/extensions.rs` | 38,379 | 1,517 |
| QuickJS bridge + hostcall plumbing + runtime adapters | `src/extensions_js.rs` | 19,284 | 449 |
| Dispatcher for protocol/hostcall integration | `src/extension_dispatcher.rs` | 11,745 | 404 |
| Provider canonical metadata + alias routing | `src/provider_metadata.rs` | 2,645 | 60 |
| Extension index/search/info/update pipeline | `src/extension_index.rs` | 1,469 | 98 |
| Environment + compatibility diagnostics (`doctor`) | `src/doctor.rs` | 1,475 | 69 |
| Runtime risk ledger/replay/calibration tooling | `src/extensions.rs`, `src/bin/ext_runtime_risk_ledger.rs` | large integrated surface | integrated |
| Per-extension quota enforcement engine | `src/extensions.rs` | integrated in core runtime | integrated |

## 4.3 Provider Breadth Delta

- Rust canonical provider IDs: `87`
- Rust alias IDs: `34`
- Legacy provider IDs (runtime `@mariozechner/pi-ai` `getProviders()`): `22`
- Exact canonical ID overlap (Rust vs legacy set): `16`
- Rust canonical IDs not in legacy exact-ID set: `71`
- Legacy-only exact IDs vs Rust canonical set: `6` (`azure-openai-responses`, `google-antigravity`, `google-gemini-cli`, `kimi-coding`, `openai-codex`, `vercel-ai-gateway`)

Complete Rust canonical IDs absent from legacy exact-ID set appear in **Appendix B**.

## 4.4 Comprehensive Rust-Only Functionality Inventory (Current Snapshot)

Verified as first-class in Rust CLI/runtime and not exposed equivalently in legacy coding-agent CLI snapshot:

1. Extension policy selection and explanation:
- `--extension-policy`, `--explain-extension-policy` (`src/extensions.rs`)

2. Extension auto-repair policy selection and explanation:
- `--repair-policy`, `--explain-repair-policy` (`src/extensions.rs`)

3. Provider registry introspection:
- `--list-providers` plus canonical/alias metadata layer (`src/provider_metadata.rs`, `src/models.rs`)

4. Extension index lifecycle commands:
- `search`, `info`, `update-index` command path (`src/extension_index.rs`, CLI wiring in `src/main.rs`)

5. Environment diagnostics command:
- `doctor` (`src/doctor.rs`)

6. Session durability and migration controls:
- `--session-durability`, `--no-migrations`, `migrate` surface (`src/main.rs`, session/store modules)

7. Large integrated extension runtime controls:
- capability-gated hostcall dispatch, policy gating, quota/risk instrumentation, runtime shims (`src/extensions.rs`, `src/extensions_js.rs`, `src/extension_dispatcher.rs`)

8. Runtime risk ledger and replay/calibration tooling:
- integrated in extension runtime plus dedicated tooling entrypoints (`src/extensions.rs`, `src/bin/ext_runtime_risk_ledger.rs`)

9. Expanded provider footprint:
- 87 canonical providers + 34 aliases in Rust vs 22 provider IDs in legacy runtime (Appendix B)

10. First-class benchmark and conformance executables in repo:
- `src/bin/ext_full_validation.rs`
- `src/bin/ext_workloads.rs`
- `src/bin/session_workload_bench.rs`

Complexity anchors for major Rust-only surfaces are listed in **Section 4.2** and **Appendix C**.

---

## 5) Benchmark Methodology (Realistic + Extreme)

All major benchmark classes were run with identical workload structure per runtime where possible.

## 5.1 Realistic E2E Workload Semantics

Realistic mode executes:
- resume/open existing long session
- append new user+assistant turns
- insert tool-result messages
- extension custom-entry activity
- slash-like state changes (model, thinking level, session info, labels)
- compaction entries
- fork simulation (`branch` summary operations)
- export generation (HTML)
- final save/index update

Parameters for realistic matrix:
- `messages=5000`
- `append=10`
- `compactions=12`
- `extension_ops=40`
- `slash_ops=40`
- `forks=8`
- `exports=2`
- token levels: `100k`, `200k`, `500k`, `1M`, `5M`
- runs per cell: `3`

---

## 6) Performance Results

Interpretation note:
- Sections 6.1-6.3 include the prior validated cross-runtime baseline matrix.
- Section 6.3.1 adds fresh 2026-02-18 Rust reruns and compares them to those prior validated legacy baselines where direct legacy reruns were blocked.

## 6.0 Cold Startup / Readiness (time-to-response)

Command-level readiness benchmark (`hyperfine`, no network calls):

| Probe | Rust mean | Legacy Node mean | Legacy Bun mean | Node/Rust | Bun/Rust |
|---|---:|---:|---:|---:|---:|
| `--help` | 3.02 ms | 1,045.10 ms | 726.28 ms | 346.22x | 240.60x |
| `--version` | 2.77 ms | 1,024.75 ms | 729.70 ms | 370.17x | 263.59x |

One-shot baseline footprint snapshot (`/usr/bin/time`):

| Probe | Runtime | RSS KB | User s | Sys s | Elapsed |
|---|---|---:|---:|---:|---:|
| `--help` | rust | 6,912 | 0.00 | 0.00 | 0:00.00 |
| `--help` | legacy_node | 156,720 | 1.11 | 0.20 | 0:01.02 |
| `--help` | legacy_bun | 195,820 | 0.91 | 0.22 | 0:00.71 |
| `--version` | rust | 6,400 | 0.00 | 0.00 | 0:00.00 |
| `--version` | legacy_node | 156,560 | 1.11 | 0.21 | 0:01.03 |
| `--version` | legacy_bun | 194,624 | 0.96 | 0.20 | 0:00.73 |

Interpretation:
- For initial CLI readiness, Rust is dramatically faster and materially lighter in baseline process footprint.
- These probes isolate startup/path initialization; they do not include session resume or extension workload execution.

## 6.1 Realistic E2E Latency (p50, ms)

| Runtime | Token level | Open | Append/Ops | Save | Total |
|---|---:|---:|---:|---:|---:|
| legacy_bun | 100k | 24.63 | 143.84 | 0.00 | 168.47 |
| legacy_node | 100k | 47.20 | 220.70 | 0.00 | 267.91 |
| rust | 100k | 36.84 | 219.06 | 64.64 | 320.71 |
| legacy_bun | 200k | 29.55 | 196.99 | 0.00 | 226.70 |
| legacy_node | 200k | 58.77 | 303.60 | 0.00 | 362.37 |
| rust | 200k | 40.42 | 397.48 | 113.92 | 552.70 |
| legacy_bun | 500k | 39.01 | 375.75 | 0.00 | 415.27 |
| legacy_node | 500k | 76.68 | 607.04 | 0.00 | 684.64 |
| rust | 500k | 51.22 | 925.65 | 250.27 | 1,226.71 |
| legacy_bun | 1M | 50.83 | 649.51 | 0.00 | 700.52 |
| legacy_node | 1M | 119.76 | 1,117.65 | 0.00 | 1,238.67 |
| rust | 1M | 68.86 | 1,846.67 | 482.81 | 2,401.35 |
| legacy_bun | 5M | 155.63 | 2,801.90 | 0.00 | 2,959.42 |
| legacy_node | 5M | 396.41 | 5,578.20 | 0.00 | 5,974.67 |
| rust | 5M | 204.35 | 9,266.76 | 2,359.30 | 11,828.14 |

Rust total p50 ratio vs legacy:

| Token level | Rust/Node | Rust/Bun |
|---|---:|---:|
| 100k | 1.20x | 1.90x |
| 200k | 1.53x | 2.44x |
| 500k | 1.79x | 2.95x |
| 1M | 1.94x | 3.43x |
| 5M | 1.98x | 4.00x |

Key interpretation:
- Rust open phase is competitive and often better than Node at higher scale.
- Current Rust bottleneck is append/save behavior under large long-session churn.

## 6.2 Matched-State Synthetic Benchmark (Same Session State, Resume + 10)

This is the direct “same state then add same 10 messages” comparison.

| Runtime | Token level | Open ms | Append ms | Save ms | Total ms | RSS KB | User s | Sys s | FS out |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| rust | 1M | 68.78 | 254.76 | 432.47 | 756.01 | 32,092 | 0.86 | 0.04 | 112 |
| legacy_node | 1M | 126.37 | 170.57 | 0.00 | 296.94 | 167,752 | 0.76 | 0.16 | 0 |
| legacy_bun | 1M | 52.85 | 90.51 | 0.00 | 143.36 | 184,492 | 0.31 | 0.17 | 0 |
| rust | 5M | 210.30 | 1,282.08 | 2,124.94 | 3,617.31 | 129,836 | 3.84 | 0.38 | 112 |
| legacy_node | 5M | 399.61 | 1,395.80 | 0.00 | 1,795.41 | 411,372 | 1.95 | 0.63 | 0 |
| legacy_bun | 5M | 156.24 | 405.62 | 0.00 | 561.86 | 481,852 | 0.56 | 0.42 | 0 |

Ratios at matched state:
- 1M: Rust latency `2.55x` Node, `5.27x` Bun; Rust memory is `5.23x` smaller than Node and `5.75x` smaller than Bun.
- 5M: Rust latency `2.01x` Node, `6.44x` Bun; Rust memory is `3.17x` smaller than Node and `3.71x` smaller than Bun.

## 6.3 Realistic Footprint (Same Realistic Ops, 1M/5M)

| Runtime | Token level | Open ms | Append/Ops ms | Save ms | Total ms | RSS KB | User s | Sys s | FS out | Wall |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| rust | 1M | 163.91 | 2,654.88 | 505.34 | 3,324.14 | 76,240 | 3.31 | 0.21 | 112 | 0:03.36 |
| legacy_node | 1M | 199.10 | 1,508.55 | 0.00 | 1,707.65 | 820,380 | 1.44 | 1.16 | 0 | 0:02.26 |
| legacy_bun | 1M | 92.71 | 810.69 | 0.00 | 903.40 | 875,092 | 0.63 | 0.79 | 0 | 0:01.21 |
| rust | 5M | 674.47 | 13,224.33 | 2,460.56 | 16,359.37 | 274,832 | 15.79 | 1.06 | 112 | 0:16.40 |
| legacy_node | 5M | 793.81 | 8,018.42 | 0.00 | 8,812.23 | 2,173,096 | 4.77 | 5.57 | 0 | 0:09.54 |
| legacy_bun | 5M | 325.28 | 3,882.84 | 0.00 | 4,208.12 | 3,057,908 | 1.67 | 3.42 | 0 | 0:04.75 |

Interpretation:
- In this baseline table, Rust latency is slower in realistic E2E.
- Memory remains much smaller (`~7.9x` to `~11.5x` lower RSS in these realistic runs).

## 6.3.1 Secure-Path Refresh (2026-02-18, New Rust Reruns)

Fresh Rust reruns (release binaries, same harness family) produced the following:

Matched-state (`resume + append same 10`):

| Runtime | Token level | Open ms | Append ms | Save ms | Total ms | RSS KB | FS out |
|---|---:|---:|---:|---:|---:|---:|---:|
| rust (fresh) | 1M | 17.49 | 47.54 | 22.63 | 87.66 | 45,140 | 15,496 |
| rust (fresh) | 5M | 58.03 | 233.92 | 73.57 | 365.52 | 146,040 | 77,224 |

Realistic (`append + compactions + extension/slash/fork/export ops`):

| Runtime | Token level | Open ms | Append/Ops ms | Save ms | Total ms | RSS KB | FS out |
|---|---:|---:|---:|---:|---:|---:|---:|
| rust (fresh) | 1M | 17.59 | 210.35 | 22.36 | 250.29 | 67,572 | 88,840 |
| rust (fresh) | 5M | 58.68 | 1,225.95 | 97.49 | 1,382.12 | 268,844 | 438,104 |

Comparison against prior validated legacy baselines in this report:

- Matched 1M: Rust total `87.66ms` vs Node `296.94ms` and Bun `143.36ms` (`Rust/Node=0.295x`, `Rust/Bun=0.611x`).
- Matched 5M: Rust total `365.52ms` vs Node `1,795.41ms` and Bun `561.86ms` (`Rust/Node=0.204x`, `Rust/Bun=0.651x`).
- Realistic 1M: Rust total `250.29ms` vs Node `1,238.67ms` and Bun `700.52ms` (`Rust/Node=0.202x`, `Rust/Bun=0.357x`).
- Realistic 5M: Rust total `1,382.12ms` vs Node `5,974.67ms` and Bun `2,959.42ms` (`Rust/Node=0.231x`, `Rust/Bun=0.467x`).

Important comparability note:
- Direct legacy reruns in this workspace were blocked by missing runtime dependencies (see Section 1.1).
- Therefore, the ratios above use fresh Rust reruns against prior validated legacy baselines from this report.

## 6.4 Session Save Hotpath Optimization Update (2026-02-17)

Applied optimization:
- Changed `Session::should_full_rewrite` in `src/session.rs` so compaction entries no longer unconditionally force full-file rewrite.
- Full rewrite triggers retained: first save, header dirty, checkpoint interval, and defensive persisted-count mismatch.
- Updated save-path unit test to assert compaction path uses incremental append:
  - `session::tests::test_compaction_entry_uses_incremental_append`

Why this lever:
- Realistic workloads include compaction events.
- Previous policy converted those workloads into repeated O(file-size) rewrites.

Measured before/after (same realistic benchmark parameters, release build):

| Token level | Metric | Before | After | Delta | Speedup |
|---|---|---:|---:|---:|---:|
| 1M | `save_ms` | 26.42 | 31.31 | +4.89 | 0.84x |
| 1M | `total_ms` | 275.59 | 264.00 | -11.59 | 1.04x |
| 5M | `save_ms` | 359.63 | 114.46 | -245.17 | 3.14x |
| 5M | `total_ms` | 1,636.38 | 1,412.39 | -223.99 | 1.16x |

Resource counters (`/usr/bin/time`):

| Token level | Metric | Before | After | Delta |
|---|---|---:|---:|---:|
| 1M | `elapsed_s` | 0.36 | 0.27 | -0.09 |
| 1M | `rss_kb` | 68,800 | 67,852 | -948 |
| 5M | `elapsed_s` | 1.76 | 1.43 | -0.33 |
| 5M | `rss_kb` | 268,692 | 268,788 | +96 |

Interpretation:
- This materially cuts the large-session save bottleneck (especially at 5M tokens).
- Aggregate runtime is still dominated by append/operation phase at high token counts, so this is a substantial but not final optimization.

---

## 7) Extension Runtime Design and Compatibility Status

## 7.1 Rust Extension Architecture (Deep-Dive)

Rust extension handling is centered on a capability-gated QuickJS host runtime with explicit hostcall dispatch and policy enforcement.

Core properties:
- Connector model instead of ambient Node/Bun authority (`tool`, `exec`, `http`, `session`, `ui`, `events`, `log`).
- Policy-first dispatch (`allow/prompt/deny`) with explainable profiles and CLI explainers.
- Deterministic event-loop bridge (microtask drain + host completion scheduling discipline).
- Structured lifecycle controls and bounded execution regions.
- Compatibility shims for high-value Node/Bun surfaces rather than full runtime emulation.
- Runtime risk scoring + hash-chained ledger + replay/calibration artifacts.
- Per-extension quota enforcement integrated into shared hostcall dispatch.

Design/implementation emphasis areas:
- `src/extensions.rs`
- `src/extensions_js.rs`
- `src/extension_dispatcher.rs`
- `EXTENSIONS.md` (runtime contract + conformance process)

## 7.2 Real Extension Execution Benchmarks (Rust vs Legacy)

Benchmark artifacts used:
- Rust: `.tmp_windyelk/ext_workloads_rust_gpt.jsonl`
- Legacy Node: `.tmp_windyelk/ext_workloads_legacy_node_gpt.jsonl`
- Legacy Bun runtime: `.tmp_windyelk/ext_workloads_legacy_bun_gpt.jsonl`

| Scenario | Extension | Rust | Legacy (Node) | Legacy (Bun runtime) | Rust/Node | Rust/Bun |
|---|---|---:|---:|---:|---:|---:|
| `ext_load_init/load_init_cold` | hello | 7.96 ms (p50) | 22.29 ms (p50) | 22.25 ms (p50) | 0.36x | 0.36x |
| `ext_load_init/load_init_cold` | pirate | 7.74 ms (p50) | 11.97 ms (p50) | 19.01 ms (p50) | 0.65x | 0.41x |
| `ext_tool_call/hello` | hello | 16.80 us/call | 1.37 us/call | 0.87 us/call | 12.26x slower | 19.37x slower |
| `ext_event_hook/before_agent_start` | pirate | 17.51 us/call | 1.71 us/call | 1.00 us/call | 10.27x slower | 17.52x slower |

Interpretation:
- Rust cold-load is now clearly competitive/faster on these representative extensions.
- Per-call dispatch overhead remains materially higher in Rust and is still a primary extension-runtime optimization target.

### 7.2.1 Incremental Optimization Update (2026-02-17)

After targeted extension-hotpath changes in `src/extensions.rs` (context payload cache reuse, `Arc<Value>` context transfer across runtime command channel, reduced task-id allocation overhead, and `await_js_task` fast-path handling), we re-ran release `ext_workloads`.

Artifacts:
- `.tmp_codex/ext_workloads_after_arc_release.jsonl`
- `.tmp_codex/ext_workloads_after_arc_release_matrix.json`
- `.tmp_codex/ext_workloads_after_arc_release_trace.jsonl`
Repeated samples:
- `.tmp_codex/ext_workloads_release_rep1.jsonl`
- `.tmp_codex/ext_workloads_release_rep2.jsonl`
- `.tmp_codex/ext_workloads_release_rep3.jsonl`

Updated Rust-only deltas vs the previous values in this report:

| Scenario | Prior Rust (report baseline) | Updated Rust (release) | Change |
|---|---:|---:|---:|
| `ext_load_init/load_init_cold` (hello, p50) | 7.96 ms | 6.93 ms | 1.15x faster (`~13.0%`) |
| `ext_load_init/load_init_cold` (pirate, p50) | 7.74 ms | 6.48 ms | 1.19x faster (`~16.3%`) |
| `ext_tool_call/hello` | 16.80 us/call | 11.88 us/call | 1.41x faster (`~29.3%`) |
| `ext_event_hook/before_agent_start` | 17.51 us/call | 15.02 us/call | 1.17x faster (`~14.2%`) |

Replication note:
- 3 immediate repeated release runs showed some host-contention variance (tool-call `~12.18-13.25us`, event-hook `~15.41-17.05us`), but still materially better than the prior baseline.

### 7.2.2 QuickJS vs Native-Rust Preview (internal micro-harness)

We re-ran `pijs_workload` to isolate runtime-engine overhead for a minimal tool roundtrip:

| Runtime engine | Command | Result |
|---|---|---:|
| QuickJS | `target/release/pijs_workload --iterations 50000 --runtime-engine quickjs` | `per_call_us_f64 = 8.29795126` (`per_call_ns_f64 = 8297.95126`) |
| Native Rust runtime (real handle path) | `target/release/pijs_workload --iterations 50000 --runtime-engine native-rust-runtime` | `per_call_us_f64 = 0.46778008` (`per_call_ns_f64 = 467.78008`) |
| Native Rust preview | `target/release/pijs_workload --iterations 50000 --runtime-engine native-rust-preview` | `per_call_us_f64 = 0.00777448` (`per_call_ns_f64 = 7.77448`) |

Important caveat:
- `native-rust-preview` is synthetic and not parity-complete.
- `native-rust-runtime` is the real runtime-handle path and is now `~17.74x` faster per call than QuickJS in this harness.
- Preview is still far faster (`~1067.33x` vs QuickJS), indicating additional headroom beyond the current real-runtime implementation.
- This micro-harness does not supersede the larger realistic session benchmarks in Section 6; it isolates extension-call runtime overhead only.

### 7.2.2.1 Fresh Extension Workload Rerun (2026-02-18)

Fresh Rust rerun artifacts:
- `target/perf/secure_path_refresh_20260218/rust_extension_workloads.jsonl`
- `target/perf/secure_path_refresh_20260218/rust_ext_hotspot_matrix.json`
- `target/perf/secure_path_refresh_20260218/rust_ext_trace.jsonl`

Fresh Rust extension metrics:
- `ext_load_init/load_init_cold` (`hello`): `p50=7.02ms`, `p95=8.52ms`
- `ext_load_init/load_init_cold` (`pirate`): `p50=6.75ms`, `p95=6.78ms`
- `ext_tool_call/hello`: `13.10us/call` (`~76.3k calls/s`)
- `ext_event_hook/before_agent_start`: `15.37us/call` (`~65.1k calls/s`)
- `ext_hostcall_bridge/long_session_real_corpus` (8 real extensions): `15.53us/call` (`~64.4k calls/s`)

Compared to prior validated legacy extension baselines in this report:
- Cold load remains faster in Rust (`hello`: `0.315x` Node, `0.316x` Bun; `pirate`: `0.564x` Node, `0.355x` Bun).
- Per-call extension dispatch remains slower (`tool_call`: `~9.56x` Node, `~15.06x` Bun; `event_hook`: `~8.99x` Node, `~15.37x` Bun).

Direct legacy extension reruns are currently blocked in this workspace:
- Node22 run: missing `@mariozechner/jiti`
- Bun run: missing `@sinclair/typebox` / `proper-lockfile`

### 7.2.3 QuickJS Removal Program (performance inversion path)

To actually invert the extension overhead (Rust faster than legacy per-call), the benchmark data supports a staged replacement:

1. Native runtime tier for hot-path hooks/tools first (`tool_call`, `tool_result`, high-frequency event hooks).
2. Keep QuickJS only as explicit compatibility/test harness infrastructure during migration (production runtime selection is now native-mandatory in this tree).
3. Introduce ahead-of-time extension lowering (manifest + typed hostcall IR) so dispatch bypasses JS marshalling for validated extensions.
4. Preserve existing policy/quota/risk guardrails in native dispatcher, but move them to pre-validated typed structs to eliminate repeated JSON decoding.
5. Gate rollout behind existing conformance corpus and perf SLI gates:
   - no regression in vendored pass rate,
   - `ext_tool_call/hello` and `ext_event_hook/before_agent_start` must beat current legacy baselines.

Near-term measurable target from current data:
- Drive `ext_tool_call/hello` from ~`11.9-12.3us` to `<1.3us` and `ext_event_hook/before_agent_start` from ~`15.0-15.5us` to `<1.7us` while maintaining conformance.

## 7.3 Corpus Conformance (Current 2026-02-18)

Sources:
- `tests/ext_conformance/reports/sharded/shard_0_report.json` (`generated_at=2026-02-18T23:43:48Z`)
- `tests/ext_conformance/reports/scenario_conformance.json` (`generated_at=2026-02-18T23:11:57Z`)
- `tests/ext_conformance/reports/parity/triage.json` (`generated_at=2026-02-18T23:12:13Z`)
- `tests/ext_conformance/reports/release_binary_e2e/ollama_firstset_dev_20260219_jobs10_timeout600.json` (`runId=release-e2e-20260219T032439Z`)
- `tests/ext_conformance/reports/release_binary_e2e/ollama_full_release_20260219_jobs10_timeout600.json` (`runId=release-e2e-20260219T033502Z`)

Vendored matrix status:
- manifest count: `224`
- tested: `224`
- pass: `224`
- fail: `0`
- skip: `0`
- overall pass rate: `100%`

Scenario conformance suite:
- total: `25`
- pass: `25`
- fail: `0`
- error: `0`
- skip: `0`

Parity triage sample:
- total: `25`
- match: `22`
- mismatch: `0`
- skip: `3`
- rust_error: `0`
- ts_error: `0`

## 7.3.1 Release-Binary Live-Provider E2E (2026-02-19)

Execution order used for this checkpoint:
1. Dev-firstset gate on debug binaries (`max_cases=20`) to validate behavior before release build.
2. Full optimized release-binary sweep on the complete vendored manifest.

Results:
- Dev-firstset gate (`ollama_firstset_dev_20260219_jobs10_timeout600.json`):
  - total: `20`
  - pass: `20`
  - fail: `0`
  - timeout: `0`
- Full release sweep (`ollama_full_release_20260219_jobs10_timeout600.json`):
  - total: `224`
  - pass: `224`
  - fail: `0`
  - timeout: `0`
  - missing_extension: `0`
  - process_error: `0`

Interpretation:
- The extension compatibility claim now has both harness-level conformance evidence and real `target/release/pi` live-provider execution evidence.
- This is strong runtime compatibility validation; it is not a throughput/latency benchmark for provider quality.

## 7.4 Historical Baseline (2026-02-14, Superseded)

The following TSV is retained for audit history from the older `223`-entry baseline run and is superseded by the current `224/224` matrix status above.

Columns: `id`, `status`, `verdict`, `failure_category`, `reason`, `suggested_fix`

```tsv
agents-mikeastock/extensions	fail	harness_gap	registration_mismatch	Observed registration output diverges from manifest expectations.	Refresh expected snapshot from TS oracle and re-validate.
community/nicobailon-interview-tool	fail	extension_problem	extension_load_error	Extension expects local assets/files unavailable at runtime.	Bundle required assets or extend missing_asset auto-repair policy.
community/prateekmedia-lsp	fail	harness_gap	registration_mismatch	Observed registration output diverges from manifest expectations.	Refresh expected snapshot from TS oracle and re-validate.
doom-overlay	fail	needs_review	extension_load_error	Extension load failure could not be cleanly mapped to limitation vs extension bug.	Inspect failure dossier and reproduce command.
npm/@verioussmith/pi-openrouter	pending	needs_review		Vendored candidate is missing from VALIDATED_MANIFEST.json.	Regenerate or repair VALIDATED_MANIFEST.json.
npm/agentsbox	pending	needs_review		Vendored candidate is missing from VALIDATED_MANIFEST.json.	Regenerate or repair VALIDATED_MANIFEST.json.
npm/aliou-pi-linkup	fail	harness_gap	registration_mismatch	Observed registration output diverges from manifest expectations.	Refresh expected snapshot from TS oracle and re-validate.
npm/aliou-pi-synthetic	fail	harness_gap	registration_mismatch	Observed registration output diverges from manifest expectations.	Refresh expected snapshot from TS oracle and re-validate.
npm/lsp-pi	fail	harness_gap	registration_mismatch	Observed registration output diverges from manifest expectations.	Refresh expected snapshot from TS oracle and re-validate.
npm/marckrenn-pi-sub-bar	fail	harness_gap	registration_mismatch	Observed registration output diverges from manifest expectations.	Refresh expected snapshot from TS oracle and re-validate.
npm/marckrenn-pi-sub-core	fail	harness_gap	registration_mismatch	Observed registration output diverges from manifest expectations.	Refresh expected snapshot from TS oracle and re-validate.
npm/mitsupi	fail	harness_gap	registration_mismatch	Observed registration output diverges from manifest expectations.	Refresh expected snapshot from TS oracle and re-validate.
npm/oh-my-pi-anthropic-websearch	pending	needs_review		Vendored candidate is missing from VALIDATED_MANIFEST.json.	Regenerate or repair VALIDATED_MANIFEST.json.
npm/oh-my-pi-exa	pending	needs_review		Vendored candidate is missing from VALIDATED_MANIFEST.json.	Regenerate or repair VALIDATED_MANIFEST.json.
npm/oh-my-pi-lsp	pending	needs_review		Vendored candidate is missing from VALIDATED_MANIFEST.json.	Regenerate or repair VALIDATED_MANIFEST.json.
npm/oh-my-pi-pi-git-tool	pending	needs_review		Vendored candidate is missing from VALIDATED_MANIFEST.json.	Regenerate or repair VALIDATED_MANIFEST.json.
npm/oh-my-pi-subagents	pending	needs_review		Vendored candidate is missing from VALIDATED_MANIFEST.json.	Regenerate or repair VALIDATED_MANIFEST.json.
npm/pi-amplike	fail	harness_gap	registration_mismatch	Observed registration output diverges from manifest expectations.	Refresh expected snapshot from TS oracle and re-validate.
npm/pi-bash-confirm	fail	harness_gap	registration_mismatch	Observed registration output diverges from manifest expectations.	Refresh expected snapshot from TS oracle and re-validate.
npm/pi-extensions	fail	harness_gap	registration_mismatch	Observed registration output diverges from manifest expectations.	Refresh expected snapshot from TS oracle and re-validate.
npm/pi-messenger	fail	needs_review	extension_load_error	Extension load failure could not be cleanly mapped to limitation vs extension bug.	Inspect failure dossier and reproduce command.
npm/pi-package-test	fail	harness_gap	registration_mismatch	Observed registration output diverges from manifest expectations.	Refresh expected snapshot from TS oracle and re-validate.
npm/pi-search-agent	fail	harness_gap	registration_mismatch	Observed registration output diverges from manifest expectations.	Refresh expected snapshot from TS oracle and re-validate.
npm/pi-shell-completions	fail	needs_review	extension_load_error	Extension load failure could not be cleanly mapped to limitation vs extension bug.	Inspect failure dossier and reproduce command.
npm/shitty-extensions	fail	harness_gap	registration_mismatch	Observed registration output diverges from manifest expectations.	Refresh expected snapshot from TS oracle and re-validate.
npm/tmustier-pi-arcade	fail	harness_gap	registration_mismatch	Observed registration output diverges from manifest expectations.	Refresh expected snapshot from TS oracle and re-validate.
npm/vaayne-agent-kit	fail	needs_review	extension_load_error	Extension load failure could not be cleanly mapped to limitation vs extension bug.	Inspect failure dossier and reproduce command.
npm/vaayne-pi-mcp	fail	needs_review	extension_load_error	Extension load failure could not be cleanly mapped to limitation vs extension bug.	Inspect failure dossier and reproduce command.
third-party/aliou-pi-extensions	fail	harness_gap	registration_mismatch	Observed registration output diverges from manifest expectations.	Refresh expected snapshot from TS oracle and re-validate.
third-party/ben-vargas-pi-packages	fail	harness_gap	registration_mismatch	Observed registration output diverges from manifest expectations.	Refresh expected snapshot from TS oracle and re-validate.
third-party/charles-cooper-pi-extensions	fail	harness_gap	registration_mismatch	Observed registration output diverges from manifest expectations.	Refresh expected snapshot from TS oracle and re-validate.
third-party/kcosr-pi-extensions	fail	harness_gap	registration_mismatch	Observed registration output diverges from manifest expectations.	Refresh expected snapshot from TS oracle and re-validate.
third-party/marckrenn-pi-sub	fail	harness_gap	registration_mismatch	Observed registration output diverges from manifest expectations.	Refresh expected snapshot from TS oracle and re-validate.
third-party/openclaw-openclaw	fail	harness_gap	registration_mismatch	Observed registration output diverges from manifest expectations.	Refresh expected snapshot from TS oracle and re-validate.
third-party/pasky-pi-amplike	fail	harness_gap	registration_mismatch	Observed registration output diverges from manifest expectations.	Refresh expected snapshot from TS oracle and re-validate.
third-party/w-winter-dot314	fail	harness_gap	registration_mismatch	Observed registration output diverges from manifest expectations.	Refresh expected snapshot from TS oracle and re-validate.
```

## 7.5 Current Gap Status

1. Current vendored conformance matrix has no outstanding non-pass entries (`224/224` pass).
2. Current scenario suite has no outstanding failures (`25/25` pass).
3. Differential parity triage currently shows `0` mismatches in sampled cases (`22` match, `3` skip).
4. Remaining work is regression prevention: keep the matrix/scenario/parity lanes as release-gate checks and investigate any future drift immediately.

---

## 8) Test Surface Comparison (Unit + E2E)

Rust:
- Rust test files: `257`
- Rust e2e-prefixed test files: `35`
- Rust test attributes: `11,976`

Legacy proxies (regex callsite counts):
- coding-agent test files: `49`
- coding-agent test callsites: `604` (`it(` + `test(` occurrences)
- full stack test files (`ai+agent+coding-agent+tui`): `107`
- full stack test callsites: `1,413` (`it(` + `test(` occurrences)

Note: legacy tree in this workspace does not provide an equivalent consolidated coverage JSON artifact like `docs/coverage-baseline-map.json` for direct percentage parity.

---

## 9) Security / Reliability / asupersync Impact

## 9.1 Security
- Rust extension path is capability-gated and auditable per hostcall.
- Policy explainers + explicit deny/prompt/allow semantics are first-class.
- Risk and quota controls are integrated and test-instrumented.

## 9.2 Reliability and Correctness
- Structured concurrency foundation (`asupersync`) reduces async lifecycle ambiguity.
- Deterministic cancellation/resource scoping improves robustness of long-lived CLI sessions.
- Hash-chained risk ledger + replay/calibration tooling improve post-incident reproducibility.

## 9.3 asupersync “Correct-by-Design” Impact
- Work is scoped to explicit lifetimes, which reduces hidden background-task leakage and orphaned async work.
- Cancellation becomes a first-class control flow primitive instead of a best-effort convention, reducing stuck-session and shutdown race risk.
- Deterministic runtime patterns make failure reproduction and forensic replay more credible (especially with extension hostcall/risk ledgers).
- The primary tradeoff is a stricter execution model that can add engineering/coordination overhead versus loosely structured async graphs.
- In this refreshed snapshot, correctness/controllability gains remain, and large-session latency has improved sharply on the secure path versus prior baselines; extension per-call overhead is still the main remaining latency gap.

## 9.4 Performance Trade in This Snapshot
- Rust startup/readiness remains decisively faster.
- Fresh Rust long-session/matched-state reruns are faster than the report’s prior validated legacy baselines at `1M` and `5M`.
- Rust still wins memory footprint substantially.
- Remaining major gap: extension per-call dispatch overhead relative to legacy Node/Bun extension runtimes.

---

## 10) Extreme Optimization Priorities (To Reach Next 5-10x)

These are the highest expected-value targets from measured bottlenecks:

1. Session append/save hot path:
- Minimize repeated full-history serialization work.
- Introduce incremental persistence for large session files.
- Reduce allocation churn and copy amplification in append/update routines.

2. JSON parse/serialize fast path:
- Eliminate avoidable intermediate `Value` transforms in hot loops.
- Prefer typed deserialization in critical paths.
- Use zero-copy/borrowed parsing where safe and measurable.

3. Extension per-call overhead:
- Reduce hostcall marshalling overhead and temporary allocations.
- Batch or precompute invariant policy/risk metadata for high-frequency calls.
- Optimize hot connector dispatch paths (`tool`/`events`).

4. Multi-core and locality:
- Partition expensive analysis and indexing work off the foreground session loop.
- Improve cache locality in session entry scans/index updates.
- Keep save/index updates append-oriented rather than full-rebuild when possible.

5. Regression guardrails:
- Keep the realistic 100k/200k/500k/1M/5M matrix as a blocking perf CI track.
- Track p50/p95, RSS, and FS I/O deltas per commit series.

---

## 11) Appendix A — Historical Vendored Extension List Snapshot (223, 2026-02-14)

Columns: `id`, `sourceTier`, `candidateStatus`, `conformanceStatus`, `verdict`, `conformanceFailureCategory`, `classificationReason`, `suggestedFix`

```tsv
agents-mikeastock/extensions	agents-mikeastock	vendored	fail	harness_gap	registration_mismatch	Observed registration output diverges from manifest expectations.	Refresh expected snapshot from TS oracle and re-validate.
antigravity-image-gen	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
auto-commit-on-exit	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
bash-spawn-hook	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
bookmark	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
claude-rules	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/ferologics-notify	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/hjanuschka-clipboard	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/hjanuschka-cost-tracker	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/hjanuschka-flicker-corp	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/hjanuschka-funny-working-message	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/hjanuschka-handoff	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/hjanuschka-loop	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/hjanuschka-memory-mode	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/hjanuschka-oracle	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/hjanuschka-plan-mode	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/hjanuschka-resistance	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/hjanuschka-speedreading	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/hjanuschka-status-widget	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/hjanuschka-ultrathink	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/hjanuschka-usage-bar	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/jyaunches-canvas	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/mitsuhiko-answer	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/mitsuhiko-control	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/mitsuhiko-cwd-history	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/mitsuhiko-files	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/mitsuhiko-loop	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/mitsuhiko-notify	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/mitsuhiko-review	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/mitsuhiko-todos	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/mitsuhiko-uv	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/mitsuhiko-whimsical	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/nicobailon-interactive-shell	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/nicobailon-interview-tool	community	vendored	fail	extension_problem	extension_load_error	Extension expects local assets/files unavailable at runtime.	Bundle required assets or extend missing_asset auto-repair policy.
community/nicobailon-mcp-adapter	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/nicobailon-powerline-footer	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/nicobailon-rewind-hook	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/nicobailon-subagents	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/ogulcancelik-ghostty-theme-sync	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/prateekmedia-checkpoint	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/prateekmedia-lsp	community	vendored	fail	harness_gap	registration_mismatch	Observed registration output diverges from manifest expectations.	Refresh expected snapshot from TS oracle and re-validate.
community/prateekmedia-permission	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/prateekmedia-ralph-loop	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/prateekmedia-repeat	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/prateekmedia-token-rate	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/qualisero-background-notify	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/qualisero-compact-config	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/qualisero-pi-agent-scip	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/qualisero-safe-git	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/qualisero-safe-rm	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/qualisero-session-color	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/qualisero-session-emoji	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/tmustier-agent-guidance	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/tmustier-arcade-mario-not	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/tmustier-arcade-picman	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/tmustier-arcade-ping	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/tmustier-arcade-spice-invaders	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/tmustier-arcade-tetris	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/tmustier-code-actions	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/tmustier-files-widget	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/tmustier-ralph-wiggum	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/tmustier-raw-paste	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/tmustier-tab-status	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
community/tmustier-usage-extension	community	vendored	pass	pass		Extension passed conformance without requiring repair.	
confirm-destructive	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
custom-compaction	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
custom-footer	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
custom-header	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
custom-provider-anthropic	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
custom-provider-gitlab-duo	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
custom-provider-qwen-cli	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
dirty-repo-guard	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
doom-overlay	official-pi-mono	vendored	fail	needs_review	extension_load_error	Extension load failure could not be cleanly mapped to limitation vs extension bug.	Inspect failure dossier and reproduce command.
dynamic-resources	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
event-bus	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
file-trigger	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
git-checkpoint	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
handoff	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
hello	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
inline-bash	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
input-transform	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
interactive-shell	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
mac-system-theme	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
message-renderer	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
modal-editor	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
model-status	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
notify	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/@verioussmith/pi-openrouter	npm-registry	vendored		needs_review		Vendored candidate is missing from VALIDATED_MANIFEST.json.	Regenerate or repair VALIDATED_MANIFEST.json.
npm/agentsbox	npm-registry	vendored		needs_review		Vendored candidate is missing from VALIDATED_MANIFEST.json.	Regenerate or repair VALIDATED_MANIFEST.json.
npm/aliou-pi-extension-dev	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/aliou-pi-guardrails	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/aliou-pi-linkup	npm-registry	vendored	fail	harness_gap	registration_mismatch	Observed registration output diverges from manifest expectations.	Refresh expected snapshot from TS oracle and re-validate.
npm/aliou-pi-processes	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/aliou-pi-synthetic	npm-registry	vendored	fail	harness_gap	registration_mismatch	Observed registration output diverges from manifest expectations.	Refresh expected snapshot from TS oracle and re-validate.
npm/aliou-pi-toolchain	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/benvargas-pi-ancestor-discovery	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/benvargas-pi-antigravity-image-gen	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/benvargas-pi-synthetic-provider	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/checkpoint-pi	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/imsus-pi-extension-minimax-coding-plan-mcp	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/juanibiapina-pi-extension-settings	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/juanibiapina-pi-files	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/juanibiapina-pi-gob	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/lsp-pi	npm-registry	vendored	fail	harness_gap	registration_mismatch	Observed registration output diverges from manifest expectations.	Refresh expected snapshot from TS oracle and re-validate.
npm/marckrenn-pi-sub-bar	npm-registry	vendored	fail	harness_gap	registration_mismatch	Observed registration output diverges from manifest expectations.	Refresh expected snapshot from TS oracle and re-validate.
npm/marckrenn-pi-sub-core	npm-registry	vendored	fail	harness_gap	registration_mismatch	Observed registration output diverges from manifest expectations.	Refresh expected snapshot from TS oracle and re-validate.
npm/mitsupi	npm-registry	vendored	fail	harness_gap	registration_mismatch	Observed registration output diverges from manifest expectations.	Refresh expected snapshot from TS oracle and re-validate.
npm/ogulcancelik-pi-sketch	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/oh-my-pi-anthropic-websearch	npm-registry	vendored		needs_review		Vendored candidate is missing from VALIDATED_MANIFEST.json.	Regenerate or repair VALIDATED_MANIFEST.json.
npm/oh-my-pi-basics	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/oh-my-pi-exa	npm-registry	vendored		needs_review		Vendored candidate is missing from VALIDATED_MANIFEST.json.	Regenerate or repair VALIDATED_MANIFEST.json.
npm/oh-my-pi-lsp	npm-registry	vendored		needs_review		Vendored candidate is missing from VALIDATED_MANIFEST.json.	Regenerate or repair VALIDATED_MANIFEST.json.
npm/oh-my-pi-pi-git-tool	npm-registry	vendored		needs_review		Vendored candidate is missing from VALIDATED_MANIFEST.json.	Regenerate or repair VALIDATED_MANIFEST.json.
npm/oh-my-pi-subagents	npm-registry	vendored		needs_review		Vendored candidate is missing from VALIDATED_MANIFEST.json.	Regenerate or repair VALIDATED_MANIFEST.json.
npm/permission-pi	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/pi-agentic-compaction	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/pi-amplike	npm-registry	vendored	fail	harness_gap	registration_mismatch	Observed registration output diverges from manifest expectations.	Refresh expected snapshot from TS oracle and re-validate.
npm/pi-annotate	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/pi-bash-confirm	npm-registry	vendored	fail	harness_gap	registration_mismatch	Observed registration output diverges from manifest expectations.	Refresh expected snapshot from TS oracle and re-validate.
npm/pi-brave-search	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/pi-command-center	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/pi-ephemeral	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/pi-extensions	npm-registry	vendored	fail	harness_gap	registration_mismatch	Observed registration output diverges from manifest expectations.	Refresh expected snapshot from TS oracle and re-validate.
npm/pi-ghostty-theme-sync	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/pi-interactive-shell	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/pi-interview	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/pi-mcp-adapter	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/pi-md-export	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/pi-mermaid	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/pi-messenger	npm-registry	vendored	fail	needs_review	extension_load_error	Extension load failure could not be cleanly mapped to limitation vs extension bug.	Inspect failure dossier and reproduce command.
npm/pi-model-switch	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/pi-moonshot	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/pi-multicodex	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/pi-notify	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/pi-package-test	npm-registry	vendored	fail	harness_gap	registration_mismatch	Observed registration output diverges from manifest expectations.	Refresh expected snapshot from TS oracle and re-validate.
npm/pi-poly-notify	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/pi-powerline-footer	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/pi-prompt-template-model	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/pi-repoprompt-mcp	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/pi-review-loop	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/pi-screenshots-picker	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/pi-search-agent	npm-registry	vendored	fail	harness_gap	registration_mismatch	Observed registration output diverges from manifest expectations.	Refresh expected snapshot from TS oracle and re-validate.
npm/pi-session-ask	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/pi-shadow-git	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/pi-shell-completions	npm-registry	vendored	fail	needs_review	extension_load_error	Extension load failure could not be cleanly mapped to limitation vs extension bug.	Inspect failure dossier and reproduce command.
npm/pi-skill-palette	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/pi-subdir-context	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/pi-super-curl	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/pi-telemetry-otel	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/pi-threads	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/pi-voice-of-god	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/pi-wakatime	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/pi-watch	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/pi-web-access	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/qualisero-pi-agent-scip	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/ralph-loop-pi	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/repeat-pi	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/shitty-extensions	npm-registry	vendored	fail	harness_gap	registration_mismatch	Observed registration output diverges from manifest expectations.	Refresh expected snapshot from TS oracle and re-validate.
npm/tmustier-pi-arcade	npm-registry	vendored	fail	harness_gap	registration_mismatch	Observed registration output diverges from manifest expectations.	Refresh expected snapshot from TS oracle and re-validate.
npm/token-rate-pi	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/vaayne-agent-kit	npm-registry	vendored	fail	needs_review	extension_load_error	Extension load failure could not be cleanly mapped to limitation vs extension bug.	Inspect failure dossier and reproduce command.
npm/vaayne-pi-mcp	npm-registry	vendored	fail	needs_review	extension_load_error	Extension load failure could not be cleanly mapped to limitation vs extension bug.	Inspect failure dossier and reproduce command.
npm/vaayne-pi-subagent	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/vaayne-pi-web-tools	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/vpellegrino-pi-skills	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/walterra-pi-charts	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/walterra-pi-graphviz	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
npm/zenobius-pi-dcp	npm-registry	vendored	pass	pass		Extension passed conformance without requiring repair.	
overlay-qa-tests	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
overlay-test	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
permission-gate	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
pirate	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
plan-mode	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
preset	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
protected-paths	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
qna	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
question	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
questionnaire	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
rainbow-editor	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
rpc-demo	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
sandbox	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
send-user-message	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
session-name	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
shutdown-command	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
snake	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
space-invaders	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
ssh	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
status-line	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
subagent	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
summarize	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
system-prompt-header	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
third-party/aliou-pi-extensions	third-party-github	vendored	fail	harness_gap	registration_mismatch	Observed registration output diverges from manifest expectations.	Refresh expected snapshot from TS oracle and re-validate.
third-party/ben-vargas-pi-packages	third-party-github	vendored	fail	harness_gap	registration_mismatch	Observed registration output diverges from manifest expectations.	Refresh expected snapshot from TS oracle and re-validate.
third-party/charles-cooper-pi-extensions	third-party-github	vendored	fail	harness_gap	registration_mismatch	Observed registration output diverges from manifest expectations.	Refresh expected snapshot from TS oracle and re-validate.
third-party/cv-pi-ssh-remote	third-party-github	vendored	pass	pass		Extension passed conformance without requiring repair.	
third-party/graffioh-pi-screenshots-picker	third-party-github	vendored	pass	pass		Extension passed conformance without requiring repair.	
third-party/graffioh-pi-super-curl	third-party-github	vendored	pass	pass		Extension passed conformance without requiring repair.	
third-party/jyaunches-pi-canvas	third-party-github	vendored	pass	pass		Extension passed conformance without requiring repair.	
third-party/kcosr-pi-extensions	third-party-github	vendored	fail	harness_gap	registration_mismatch	Observed registration output diverges from manifest expectations.	Refresh expected snapshot from TS oracle and re-validate.
third-party/limouren-agent-things	third-party-github	vendored	pass	pass		Extension passed conformance without requiring repair.	
third-party/lsj5031-pi-notification-extension	third-party-github	vendored	pass	pass		Extension passed conformance without requiring repair.	
third-party/marckrenn-pi-sub	third-party-github	vendored	fail	harness_gap	registration_mismatch	Observed registration output diverges from manifest expectations.	Refresh expected snapshot from TS oracle and re-validate.
third-party/michalvavra-agents	third-party-github	vendored	pass	pass		Extension passed conformance without requiring repair.	
third-party/ogulcancelik-pi-sketch	third-party-github	vendored	pass	pass		Extension passed conformance without requiring repair.	
third-party/openclaw-openclaw	third-party-github	vendored	fail	harness_gap	registration_mismatch	Observed registration output diverges from manifest expectations.	Refresh expected snapshot from TS oracle and re-validate.
third-party/pasky-pi-amplike	third-party-github	vendored	fail	harness_gap	registration_mismatch	Observed registration output diverges from manifest expectations.	Refresh expected snapshot from TS oracle and re-validate.
third-party/qualisero-pi-agent-scip	third-party-github	vendored	pass	pass		Extension passed conformance without requiring repair.	
third-party/raunovillberg-pi-stuffed	third-party-github	vendored	pass	pass		Extension passed conformance without requiring repair.	
third-party/rytswd-direnv	third-party-github	vendored	pass	pass		Extension passed conformance without requiring repair.	
third-party/rytswd-questionnaire	third-party-github	vendored	pass	pass		Extension passed conformance without requiring repair.	
third-party/rytswd-slow-mode	third-party-github	vendored	pass	pass		Extension passed conformance without requiring repair.	
third-party/vtemian-pi-config	third-party-github	vendored	pass	pass		Extension passed conformance without requiring repair.	
third-party/w-winter-dot314	third-party-github	vendored	fail	harness_gap	registration_mismatch	Observed registration output diverges from manifest expectations.	Refresh expected snapshot from TS oracle and re-validate.
third-party/zenobi-us-pi-dcp	third-party-github	vendored	pass	pass		Extension passed conformance without requiring repair.	
timed-confirm	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
titlebar-spinner	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
todo	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
tool-override	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
tools	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
trigger-compact	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
truncated-tool	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
widget-placement	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
with-deps	official-pi-mono	vendored	pass	pass		Extension passed conformance without requiring repair.	
```

## 12) Appendix B — Rust Canonical Provider IDs Not in Legacy Exact-ID Set (71)

```text
302ai
abacus
aihubmix
alibaba
alibaba-cn
azure-openai
bailing
baseten
berget
chutes
cloudflare-ai-gateway
cloudflare-workers-ai
cohere
cortecs
deepinfra
deepseek
fastrouter
fireworks
firmware
friendli
github-models
gitlab
helicone
iflowcn
inception
inference
io-net
jiekou
kimi-for-coding
llama
lmstudio
lucidquery
minimax-cn-coding-plan
minimax-coding-plan
moark
modelscope
moonshotai
moonshotai-cn
morph
nano-gpt
nebius
nova
novita-ai
nvidia
ollama
ollama-cloud
ovhcloud
perplexity
poe
privatemode-ai
requesty
sap-ai-core
scaleway
siliconflow
siliconflow-cn
stackit
submodel
synthetic
togetherai
upstage
v0
venice
vercel
vivgrid
vultr
wandb
xiaomi
zai-coding-plan
zenmux
zhipuai
zhipuai-coding-plan
```

## 13) Appendix C — Feature Complexity Tables Used in This Report

### 13.1 Rust Feature Complexity Table

```tsv
file	loc	fn_count
src/extensions.rs	38379	1517
src/extensions_js.rs	19284	449
src/extension_dispatcher.rs	11745	404
src/provider_metadata.rs	2645	60
src/extension_index.rs	1469	98
src/doctor.rs	1475	69
src/session.rs	7294	334
src/session_index.rs	1648	88
src/cli.rs	1330	107
src/main.rs	3632	120
src/providers/mod.rs	2127	105
src/providers/openai.rs	1903	75
src/providers/anthropic.rs	1792	63
src/providers/gemini.rs	1317	59
src/providers/azure.rs	1030	39
src/providers/cohere.rs	1551	51
src/providers/vertex.rs	801	39
src/providers/bedrock.rs	1048	42
src/providers/gitlab.rs	375	22
src/providers/copilot.rs	424	22
src/bin/ext_full_validation.rs	1635	28
src/bin/ext_workloads.rs	4537	120
src/bin/session_workload_bench.rs	435	18
```

### 13.2 Legacy Feature Complexity Table

```tsv
file	loc	callables
packages/coding-agent/src/core/extensions/index.ts	132	0
packages/coding-agent/src/core/extensions/wrapper.ts	85	4
packages/coding-agent/src/core/extensions/runner.ts	615	37
packages/coding-agent/src/core/session-manager.ts	1011	61
packages/coding-agent/src/core/model-registry.ts	432	21
packages/coding-agent/src/cli/args.ts	286	3
packages/coding-agent/src/main.ts	619	16
packages/ai/src/providers/register-builtins.ts	62	2
packages/ai/src/providers/openai-responses.ts	222	8
packages/ai/src/providers/openai-completions.ts	699	15
packages/ai/src/providers/anthropic.ts	637	15
packages/ai/src/providers/google.ts	408	9
packages/ai/src/providers/google-vertex.ts	435	11
packages/ai/src/providers/amazon-bedrock.ts	547	15
packages/ai/src/providers/azure-openai-responses.ts	212	9
packages/ai/src/providers/google-gemini-cli.ts	862	16
packages/ai/src/providers/openai-codex-responses.ts	356	13
```

## 14) Appendix D — Primary Raw Artifacts

- Realistic E2E latency + matched-state + footprint matrices (baseline dataset reused in this report)
  - `BENCHMARK_COMPARISON_BETWEEN_RUST_VERSION_AND_ORIGINAL__CODEX.md` (source tables and matrix outputs)
  - `.bench/pi_session_bench/after_round2_runs.jsonl`
  - `.bench/pi_session_bench/after_round3_runs.jsonl`
  - `.bench/pi_session_bench/after_round4_runs.jsonl`
  - `.bench/pi_session_bench/after_round5_runs.jsonl`
- Extension execution microbench
  - `.tmp_windyelk/ext_workloads_rust_gpt.jsonl`
  - `.tmp_windyelk/ext_workloads_legacy_node_gpt.jsonl`
  - `.tmp_windyelk/ext_workloads_legacy_bun_gpt.jsonl`
- Cold-start readiness and footprint probes
  - `/tmp/startup_help_compare.json`
  - `/tmp/startup_version_compare.json`
- Extension conformance corpus outputs
  - `tests/ext_conformance/reports/pipeline/full_validation_report.compat2.json`
  - `tests/ext_conformance/reports/pipeline/full_validation_report.compat2.md`
- Provider inventory/parity artifacts
  - `docs/provider-canonical-id-table.json`
  - `docs/provider-parity-reconciliation-report.json`
  - `/tmp/provider_diff.json`
  - `/tmp/help_diff.json`
  - `.tmp_windyelk/rust_provider_extra.txt`
  - `.tmp_windyelk/provider_overlap.txt`
  - `.tmp_windyelk/legacy_provider_extra.txt`
- Coverage and test-surface artifacts
  - `docs/coverage-baseline-map.json`
  - `docs/TEST_COVERAGE_MATRIX.md`
  - `/tmp/ts_counts_coding_agent.json`
  - `/tmp/ts_counts_fullstack.json`
  - `.tmp_windyelk/pi_rust_tokei.json`
  - `.tmp_windyelk/pi_legacy_coding_agent_tokei.json`
  - `.tmp_windyelk/pi_legacy_fullstack_tokei.json`
