# Fuzzing in pi_agent_rust

This directory contains `cargo-fuzz` harnesses and seed corpora for coverage-guided fuzzing.

## Requirements

- Rust nightly toolchain (project default)
- `cargo-fuzz` installed
- `rch` available for CPU-heavy runs (required in this repo)

Install `cargo-fuzz` if needed:

```bash
cargo install cargo-fuzz
```

## Directory Layout

- `fuzz/Cargo.toml`: fuzz package and target registrations
- `fuzz/fuzz_targets/*.rs`: libFuzzer harness implementations
- `fuzz/corpus/<target>/`: seed corpora per target
- `fuzz/artifacts/<target>/`: crash artifacts and reproducers

## Current Targets

- `fuzz_smoke`: infrastructure smoke test
- `fuzz_sse_parser`: SSE parser chunking invariant (`feed`/`flush`)
- `fuzz_sse_stream`: byte-level UTF-8 + SSE processing
- `fuzz_session_jsonl`: session JSONL open/decode paths
- `fuzz_session_entry`: standalone `SessionEntry` serde paths
- `fuzz_message_deser`: message/content deserialization entry points
- `fuzz_message_roundtrip`: serde round-trip invariants
- `fuzz_tool_paths`: path resolution/normalization behavior
- `fuzz_grep_pattern`: grep pattern handling (regex/literal)
- `fuzz_edit_match`: edit matching/replacement behavior
- `fuzz_provider_event`: provider `process_event()` flows (Anthropic/OpenAI/Gemini/Cohere/OpenAI Responses/Azure/Vertex)

## Running Fuzzers

Always run heavy fuzz commands through `rch`.

Set high-capacity temporary paths first:

```bash
export CARGO_TARGET_DIR="/data/tmp/pi_agent_rust/${USER:-agent}"
export TMPDIR="/data/tmp/pi_agent_rust/${USER:-agent}/tmp"
mkdir -p "$TMPDIR"
cd fuzz
```

Single-target smoke run (60s):

```bash
rch exec -- cargo fuzz run fuzz_sse_parser -- -max_total_time=60
```

Quick multi-target sweep:

```bash
for t in \
  fuzz_sse_parser fuzz_sse_stream fuzz_session_jsonl fuzz_session_entry \
  fuzz_message_deser fuzz_message_roundtrip fuzz_tool_paths fuzz_grep_pattern \
  fuzz_edit_match fuzz_provider_event
  do
    rch exec -- cargo fuzz run "$t" -- -max_total_time=30
  done
```

## Coverage Dashboard (`bd-1akey`)

Use `fuzz/generate-coverage.sh` for local coverage snapshots and trend tracking.
The script runs `cargo fuzz coverage` per target, extracts llvm-cov percentages
when available, and writes machine-readable report artifacts.

```bash
# Single target coverage snapshot
./fuzz/generate-coverage.sh --target=fuzz_sse_parser --runs=0 --require-rch

# Multiple targets with a small additional run budget
./fuzz/generate-coverage.sh --target=fuzz_sse_parser --target=fuzz_provider_event --runs=25 --require-rch
```

Generated artifacts (default paths):
- `fuzz/reports/fuzz_coverage_*.json` (schema: `pi.fuzz.coverage_report.v1`)
- `fuzz/reports/fuzz_coverage_history.jsonl` append-only trend history
- `fuzz/reports/fuzz_coverage_*_llvm_summary.json` per-target llvm summary export
- `fuzz/reports/fuzz_coverage_*.log` run log

Coverage report fields include per-target line/function/region percentages and
target status, suitable for CI ingestion and historical trend analysis.

CI also runs a dedicated dashboard job in `.github/workflows/fuzz.yml`
(`fuzz-coverage-dashboard`) that emits Markdown plus JSON artifacts with line
and branch coverage breakdowns.

## Phase 1 Validation Suite (`bd-26ecm`)

Use the P1 validator to run the curated Phase 1 proptest matrix and emit a
structured summary report with per-function status, timing, and case counts.

```bash
# Default: require >=2000 aggregate generated cases
./scripts/validate_fuzz_p1.sh

# Raise the aggregate case threshold
./scripts/validate_fuzz_p1.sh --min-cases=2500
```

`validate_fuzz_p1.sh` uses `rch exec --` automatically when available.

```bash
# Require remote execution
./scripts/validate_fuzz_p1.sh --require-rch

# Force local execution
./scripts/validate_fuzz_p1.sh --no-rch
```

Report files are written to `fuzz/reports/`:
- `p1_validation_*.json` final summary report
- `p1_validation_*.jsonl` per-test machine-readable event stream
- `p1_validation_*.log` suite-level execution log

The JSON summary includes:
- total proptest functions executed
- total aggregate generated cases and threshold verification
- per-function pass/fail status, time, configured/recorded case count, log path
- overall pass/fail summary suitable for CI consumption

The terminal output prints colored `PASS`/`FAIL` indicators for quick human review.

## Phase 2 Validation Suite (`bd-1uny7`)

Use the dedicated validation runner to build all harnesses, run each for a bounded smoke duration,
and emit a structured JSON report.

```bash
# Default: 60s per target
./scripts/validate_fuzz_p2.sh

# Faster local smoke while iterating
./scripts/validate_fuzz_p2.sh --time=15

# Limit to specific targets
./scripts/validate_fuzz_p2.sh --target=fuzz_sse_parser --target=fuzz_tool_paths --time=30
```

`validate_fuzz_p2.sh` runs `cargo fuzz` commands through `rch exec --` automatically when `rch`
is available. Override behavior explicitly when needed:

```bash
# Require remote execution and fail if rch is unavailable
./scripts/validate_fuzz_p2.sh --require-rch

# Force local execution
./scripts/validate_fuzz_p2.sh --no-rch
```

When `CARGO_TARGET_DIR`/`TMPDIR` are not already set, the script auto-assigns
an isolated per-run target/temp directory (preferring `/dev/shm`) to reduce
Cargo lock contention in multi-agent sessions.

Report files are written to `fuzz/reports/p2_validation_*.json` and include:
- build status/exit code/time and build log path
- per-target status (`pass`/`fail`/`crashed`), exit code, duration, corpus growth, artifact growth
- summary counters for pass/fail/crash totals and aggregate timings

## FUZZ-V3 Unified Orchestration (`bd-cz006`)

Use the master orchestrator when you want a single command that runs both phases,
emits a JSONL event stream, and writes a unified summary report.

```bash
# Full pipeline: P1 + P2 (60s per fuzz target)
./scripts/fuzz_e2e.sh

# Quick mode: P1 + short P2 smoke (10s, defaults to fuzz_smoke target)
./scripts/fuzz_e2e.sh --quick

# Deep mode: P1 + long P2 burn-in (1800s per target)
./scripts/fuzz_e2e.sh --deep

# Regenerate unified report/events from latest P1/P2 reports
./scripts/fuzz_e2e.sh --report
```

Useful options:
- `--p1-min-cases=N` override P1 aggregate case threshold
- `--p2-time=SECONDS` override P2 per-target budget
- `--target=NAME` restrict P2 target set (repeatable)
- `--no-rch` / `--require-rch` forward execution policy to phase scripts
- `--output=PATH` and `--events=PATH` customize output file locations

Generated artifacts (default paths):
- `fuzz/reports/fuzz_e2e_*.json` unified pipeline report
- `fuzz/reports/fuzz_e2e_*.jsonl` per-step event log (`pipeline_start`, phase start/end, report generation, pipeline end)
- `fuzz/reports/fuzz_e2e_*.log` operator-facing execution log

Unified report structure includes:
- `phases.P1_proptest` (embedded P1 report)
- `phases.P2_libfuzzer` (embedded P2 report)
- `phase_runs` metadata (phase exit/time/report paths)
- summarized verdict fields (`overall_status`, totals, `exit_code`)

Master script exit codes:
- `0`: all phases pass
- `1`: P1 failures
- `2`: P2 crashes/failures
- `3`: both P1 and P2 failures
- `4`: P2 build/infrastructure failure

## Seed Corpus Management

General rule: each corpus should contain diverse valid inputs plus known edge/failure shapes.

Current corpus directories:

- `fuzz_sse_parser`, `fuzz_sse_stream`
- `fuzz_session_jsonl`, `fuzz_session_entry`
- `fuzz_message_deser`, `fuzz_message_roundtrip`
- `fuzz_tool_paths`, `fuzz_grep_pattern`, `fuzz_edit_match`
- `fuzz_provider_event`
- forward-looking seed sets for pending harnesses: `fuzz_config`, `fuzz_extension_payload`

Recommended source material for new seeds:

- `tests/fixtures/provider_responses/*.json`
- `tests/fixtures/vcr/*.json`
- `tests/conformance/fixtures/*.json`
- `tests/ext_conformance/mock_specs/*.json`
- representative `tests/**/*.jsonl` logs

When adding seeds:

1. Keep files small and focused (one scenario per seed file).
2. Mix parseable and malformed payloads.
3. Preserve provenance in filenames where possible (`provider_case_event`, `fixture_case`, etc.).
4. Re-run at least a short smoke fuzz pass for affected targets.

## Crash Corpus Management

Crash lifecycle is managed by `scripts/fuzz_crash_manage.sh` and follows a structured pipeline:

```
artifacts/<target>/crash-*  ──triage──>  crashes/<target>/<cat>-NNN.bin  ──regress──>  regression/<target>/<cat>-NNN.bin
  (transient, gitignored)                 (committed, tracked)                          (committed, regression)
```

### Directory Layout

- `fuzz/artifacts/<target>/`: Raw crash inputs from libFuzzer (gitignored, transient)
- `fuzz/crashes/<target>/`: Minimized, categorized crashes with JSON metadata (committed)
- `fuzz/regression/<target>/`: Fixed crashes preserved as regression inputs (committed)

### Crash Categories

| Category | Description |
|----------|-------------|
| `oom` | Memory exhaustion from unbounded allocation |
| `stack-overflow` | Deep recursion without limit |
| `panic-unwrap` | `.unwrap()` on `None`/`Err` |
| `panic-index` | Array/slice index out of bounds |
| `panic-assertion` | `assert!` or `debug_assert!` failure |
| `timeout` | Infinite loop or catastrophic backtracking |
| `logic-error` | Incorrect behavior found by harness assertions |
| `unknown` | Uncategorized or pending investigation |

### Crash Triage Workflow

1. **Discover**: Run fuzzer, check for new artifacts:

```bash
./scripts/fuzz_crash_manage.sh triage
```

2. **Reproduce**: Confirm the crash is deterministic:

```bash
rch exec -- cargo fuzz run <target> fuzz/artifacts/<target>/crash-<hash>
```

3. **Minimize**: Reduce crash input to minimal reproducer:

```bash
./scripts/fuzz_crash_manage.sh minimize <target> fuzz/artifacts/<target>/crash-<hash>
```

4. **Categorize and store**: Move to tracked crashes with metadata:

```bash
./scripts/fuzz_crash_manage.sh store <target> fuzz/artifacts/<target>/crash-<hash> \
    --category=panic-unwrap --description="Description of the crash"
```

5. **Fix**: Address root cause in source code.

6. **Regression**: Move fixed crash to regression corpus:

```bash
./scripts/fuzz_crash_manage.sh regress <target> panic-unwrap-001.bin --bead=bd-XXXX
```

7. **Verify**: Re-run smoke fuzzing and confirm no regression:

```bash
rch exec -- cargo fuzz run <target> -- -max_total_time=60
```

### Crash Report

Generate a summary of all crashes (open and resolved):

```bash
# Human-readable
./scripts/fuzz_crash_manage.sh report

# Machine-readable JSON (pi.fuzz.crash_report.v1)
./scripts/fuzz_crash_manage.sh report --format=json
```

### Metadata Sidecar Format

Each stored crash has a `.json` sidecar with schema `pi.fuzz.crash_metadata.v1`:

```json
{
  "schema": "pi.fuzz.crash_metadata.v1",
  "target": "fuzz_sse_parser",
  "category": "panic-unwrap",
  "original_artifact": "crash-28de6b...",
  "stored_as": "panic-unwrap-001.bin",
  "description": "SSE parser crash on malformed input",
  "stored_at": "2026-02-15T03:45:53Z",
  "size_bytes": 185,
  "status": "open"
}
```

When resolved via `regress`, the metadata gains `resolved_at` and `bead` fields.

## Multi-Agent Coordination

Before editing harnesses/corpus in shared sessions:

- reserve paths via MCP Agent Mail (`file_reservation_paths`)
- announce start/completion in the bead thread
- release reservations after completion

This prevents corpus and harness collisions when multiple agents fuzz in parallel.
