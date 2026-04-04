# Performance Budgets

> Generated: 2026-03-12T02:21:00Z

## Summary

| Metric | Value |
|---|---|
| Total budgets | 13 |
| CI-enforced | 8 |
| CI-enforced with data | 1 |
| CI-enforced FAIL | 0 |
| CI-enforced NO_DATA | 7 |
| PASS | 4 |
| FAIL | 0 |
| No data | 9 |

| Failing data contracts | 9 |

## Startup

| Budget | Metric | Threshold | Actual | Status | CI |
|---|---|---|---|---|---|
| `startup_version_p95` | p95 latency | 100 ms | - | NO_DATA | Yes |
| `startup_full_agent_p95` | p95 latency | 200 ms | - | NO_DATA | No |

## Extension

| Budget | Metric | Threshold | Actual | Status | CI |
|---|---|---|---|---|---|
| `ext_cold_load_simple_p95` | p95 cold load time | 5 ms | - | NO_DATA | Yes |
| `ext_cold_load_complex_p95` | p95 cold load time | 50 ms | - | NO_DATA | No |
| `ext_load_60_total` | total load time (60 official extensions) | 10000 ms | 6198.0 | PASS | No |

## Tool_call

| Budget | Metric | Threshold | Actual | Status | CI |
|---|---|---|---|---|---|
| `tool_call_latency_p99` | p99 per-call latency | 200 us | - | NO_DATA | Yes |
| `tool_call_throughput_min` | minimum calls/sec | 5000 calls/sec | - | NO_DATA | Yes |

## Event_dispatch

| Budget | Metric | Threshold | Actual | Status | CI |
|---|---|---|---|---|---|
| `event_dispatch_p99` | p99 dispatch latency | 5000 us | 45 | PASS | No |

## Policy

| Budget | Metric | Threshold | Actual | Status | CI |
|---|---|---|---|---|---|
| `policy_eval_p99` | p99 evaluation time | 500 ns | - | NO_DATA | Yes |

## Memory

| Budget | Metric | Threshold | Actual | Status | CI |
|---|---|---|---|---|---|
| `idle_memory_rss` | RSS at idle | 50 MB | 11.3 | PASS | Yes |
| `sustained_load_rss_growth` | RSS growth under 30s sustained load | 5 percent | 0.0 | PASS | No |

## Binary

| Budget | Metric | Threshold | Actual | Status | CI |
|---|---|---|---|---|---|
| `binary_size_release` | release binary size | 22 MB | - | NO_DATA | Yes |

## Protocol

| Budget | Metric | Threshold | Actual | Status | CI |
|---|---|---|---|---|---|
| `protocol_parse_p99` | p99 parse+validate time | 50 us | - | NO_DATA | Yes |

## Failing Data Contracts

- `missing_or_stale_budget_artifact` (`startup_version_p95`): missing artifacts; expected one of [/data/projects/pi_agent_rust/target/criterion/startup/version/warm/new/estimates.json]
  - Remediation: Regenerate benchmark artifacts in the same CI/perf run before evaluating budgets.
- `missing_or_stale_budget_artifact` (`ext_cold_load_simple_p95`): missing artifacts; expected one of [/data/projects/pi_agent_rust/target/criterion/ext_load_init/load_init_cold/hello/new/estimates.json]
  - Remediation: Regenerate benchmark artifacts in the same CI/perf run before evaluating budgets.
- `missing_or_stale_budget_artifact` (`tool_call_latency_p99`): missing artifacts; expected one of [/data/projects/pi_agent_rust/target/perf/perf/pijs_workload_perf.jsonl, /data/projects/pi_agent_rust/target/perf/release/pijs_workload_release.jsonl, /data/projects/pi_agent_rust/target/perf/debug/pijs_workload_debug.jsonl, /data/projects/pi_agent_rust/target/perf/pijs_workload.jsonl]
  - Remediation: Regenerate benchmark artifacts in the same CI/perf run before evaluating budgets.
- `missing_or_stale_budget_artifact` (`tool_call_throughput_min`): missing artifacts; expected one of [/data/projects/pi_agent_rust/target/perf/perf/pijs_workload_perf.jsonl, /data/projects/pi_agent_rust/target/perf/release/pijs_workload_release.jsonl, /data/projects/pi_agent_rust/target/perf/debug/pijs_workload_debug.jsonl, /data/projects/pi_agent_rust/target/perf/pijs_workload.jsonl]
  - Remediation: Regenerate benchmark artifacts in the same CI/perf run before evaluating budgets.
- `missing_or_stale_budget_artifact` (`policy_eval_p99`): missing artifacts; expected one of [/data/projects/pi_agent_rust/target/criterion/ext_policy/evaluate]
  - Remediation: Regenerate benchmark artifacts in the same CI/perf run before evaluating budgets.
- `missing_or_stale_budget_artifact` (`binary_size_release`): missing artifacts; expected one of [/data/projects/pi_agent_rust/target/release/pi, /data/projects/pi_agent_rust/target/perf/pi]
  - Remediation: Regenerate benchmark artifacts in the same CI/perf run before evaluating budgets.
- `missing_or_stale_budget_artifact` (`protocol_parse_p99`): missing artifacts; expected one of [/data/projects/pi_agent_rust/target/criterion/ext_protocol/parse_and_validate]
  - Remediation: Regenerate benchmark artifacts in the same CI/perf run before evaluating budgets.
- `missing_or_stale_e2e_matrix_evidence` (`global`): all candidate artifacts are stale/invalid (>24.00h): /data/projects/pi_agent_rust/tests/perf/reports/extension_benchmark_stratification.json (551.26h old)
  - Remediation: Generate fresh extension_benchmark_stratification.json in the current perf run.
- `missing_or_stale_phase1_matrix_validation_evidence` (`global`): missing artifacts; expected one of [/data/projects/pi_agent_rust/target/perf/results/phase1_matrix_validation.json, /data/projects/pi_agent_rust/tests/perf/reports/phase1_matrix_validation.json]
  - Remediation: Generate fresh phase1_matrix_validation.json in the current perf run.

## Measurement Methodology

- **`startup_version_p95`**: hyperfine: `pi --version` (10 runs, 3 warmup)
- **`startup_full_agent_p95`**: hyperfine: `pi --print '.'` with full init (10 runs, 3 warmup)
- **`ext_cold_load_simple_p95`**: criterion: load_init_cold for simple single-file extensions (10 samples)
- **`ext_cold_load_complex_p95`**: criterion: load_init_cold for multi-registration extensions (10 samples)
- **`ext_load_60_total`**: conformance runner: sequential load of all 60 official extensions
- **`tool_call_latency_p99`**: pijs_workload: 2000 iterations x 1 tool call, perf profile
- **`tool_call_throughput_min`**: pijs_workload: 2000 iterations x 10 tool calls, perf profile
- **`event_dispatch_p99`**: criterion: event_hook dispatch for before_agent_start (100 samples)
- **`policy_eval_p99`**: criterion: ext_policy/evaluate with various modes and capabilities
- **`idle_memory_rss`**: sysinfo: measure RSS after startup, before any user input
- **`sustained_load_rss_growth`**: stress test: 15 extensions, 50 events/sec for 30 seconds
- **`binary_size_release`**: ls -la target/release/pi (stripped)
- **`protocol_parse_p99`**: criterion: ext_protocol/parse_and_validate for host_call and log messages

## CI Enforcement

CI-enforced budgets are checked on every PR. A budget violation blocks the PR from merging. Non-CI budgets are informational and checked in nightly runs.

```bash
# Run budget checks
cargo test --test perf_budgets -- --nocapture

# Generate full budget report
cargo test --test perf_budgets generate_budget_report -- --nocapture
```
