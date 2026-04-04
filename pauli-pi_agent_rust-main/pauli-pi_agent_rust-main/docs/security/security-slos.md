# Extension Security SLOs, Risk Budgets, and Release Gates

Status: Active
Primary bead: `bd-23sa8` (SEC-1.4)
Cross-references:
- `docs/security/threat-model.md` (SEC-1.1, `bd-3jyg8`)
- `docs/security/invariants.md` (SEC-1.2, `bd-2ezm9`)
- `docs/security/baseline-audit.md` (SEC-1.3, `bd-2nr0q`)
Last updated: 2026-02-14

## 1. Purpose

This document defines measurable security objectives that govern release
decisions for the `pi_agent_rust` extension runtime. It replaces subjective
confidence with numeric targets, each traceable to a threat (T1-T8),
invariant (INV-001-012), or baseline audit gap (G-1-G-7).

All SLOs are designed to be testable by the CI pipeline. Release promotion
and rollback decisions are justified from artifact evidence, not human
judgment alone.

## 2. SLO Definitions

### 2.1 SLO Index

| SLO ID | Name | Target | Measurement | Threat/Invariant |
|---|---|---|---|---|
| SLO-01 | Invariant pass rate | 100% | All INV-001..012 tests pass | INV-001-012 |
| SLO-02 | Scanner detection rate | >= 95% | Forbidden patterns detected / total in corpus | T1, T8 |
| SLO-03 | Scanner false-positive rate | <= 5% | False flags / total scanned extensions | T1 |
| SLO-04 | Policy eval latency (p99) | <= 1ms | Policy decision time, measured in test harness | T4, INV-003 |
| SLO-05 | Hostcall dispatch latency (p99) | <= 10ms | End-to-end dispatch, excluding connector work | T2, T4 |
| SLO-06 | Runtime risk decision latency (p99) | <= 5ms | Risk score + action selection time | T4, INV-009 |
| SLO-07 | Env var blocklist coverage | >= 90% | Known secret patterns blocked / total known | T6, INV-011 |
| SLO-08 | Dangerous cap default-deny rate | 100% | `exec`/`env` denied under default config | T3, INV-007 |
| SLO-09 | Ledger integrity verification | 100% | Hash-chain verification passes on all entries | T5, INV-010 |
| SLO-10 | Runtime risk false-positive rate | <= 10% | Benign extensions flagged / total benign tested | T4 |
| SLO-11 | Runtime risk false-negative rate | <= 5% | Hostile extensions missed / total hostile tested | T4 |
| SLO-12 | Capability mapping completeness | 100% | All hostcall methods have capability mappings | T2, INV-001 |
| SLO-13 | Audit gap closure velocity | >= 1 gap/sprint | Baseline audit gaps (G-*) closed per sprint | All |
| SLO-14 | Runtime overhead budget | <= 3% | Total wall-time overhead from security layers | T4 |

### 2.2 SLO Details

#### SLO-01: Invariant Pass Rate

**Target**: 100% of invariant tests (INV-001 through INV-012) pass in every
CI run. Zero tolerance for invariant regressions.

**Measurement**: `cargo test` results for all test functions listed in
`docs/security/invariants.machine.json`. Each invariant maps to 2-3 test
functions. Total: ~36 test functions.

**Rationale**: Invariants are non-negotiable by definition. A failing
invariant test means a security boundary has been violated. This SLO has
no error budget.

**Breach response**: Immediate build failure. No merge until fixed.

#### SLO-02: Scanner Detection Rate

**Target**: >= 95% of forbidden patterns in the adversarial corpus are
detected by the compatibility scanner.

**Measurement**: Run `CompatibilityScanner` against the adversarial
extension corpus (tracked in `bd-3jpm3`). Count forbidden patterns
detected vs total patterns planted. Patterns include: `process.binding()`,
`process.dlopen()`, `eval()`, `new Function()`, forbidden builtins
(vm, worker_threads, net, tls, etc.).

**Rationale**: The scanner is the first line of defense at B1 (package
source boundary). A 95% threshold accounts for legitimate obfuscation
edge cases (minified code, complex comment patterns) while requiring
strong baseline coverage.

**Breach response**: Add detection rules for missed patterns. Block
release until rate is restored.

#### SLO-03: Scanner False-Positive Rate

**Target**: <= 5% of scanned extensions produce false forbidden/flagged
classifications.

**Measurement**: Run scanner against the known-good conformance corpus
(60 official + 58 community extensions from `bd-150s`, `bd-2ru2`). Count
extensions that produce `forbidden` or `flagged` entries for code patterns
that are actually safe (e.g., string literals containing "eval", comments
mentioning process.binding).

**Rationale**: High false-positive rates erode trust and lead operators to
ignore scanner output. The 5% budget allows for edge cases in complex
comment/string handling without compromising signal quality.

**Breach response**: Refine detection regex patterns. Add comment-stripping
edge case tests.

#### SLO-04: Policy Eval Latency (p99)

**Target**: <= 1ms for `ExtensionPolicy::evaluate_for()` to return a
`PolicyCheck`.

**Measurement**: Benchmark `evaluate_for()` with worst-case inputs (large
per-extension override maps, many capabilities). p99 over 10,000 iterations.

**Rationale**: Policy evaluation is on the critical path of every hostcall.
Sub-millisecond latency ensures security checks do not create perceptible
delay in extension execution.

**Breach response**: Profile the evaluation path. Optimize hash lookups or
data structures. Consider caching.

#### SLO-05: Hostcall Dispatch Latency (p99)

**Target**: <= 10ms for end-to-end dispatch (validation + capability
derivation + policy check + risk evaluation + logging), excluding the
connector's actual work (tool execution, HTTP request, etc.).

**Measurement**: Benchmark `dispatch_host_call_shared()` with mock
connectors that return immediately. p99 over 1,000 iterations.

**Rationale**: The security overhead of the hostcall dispatch pipeline must
be bounded. 10ms accounts for the full 6-stage pipeline including tracing.

**Breach response**: Profile dispatch stages. Identify bottleneck
(validation, policy, risk, logging). Optimize the slowest stage.

#### SLO-06: Runtime Risk Decision Latency (p99)

**Target**: <= 5ms for `evaluate_runtime_risk()` to return an action.

**Measurement**: Benchmark risk evaluation with a full 128-entry sliding
window and Bayesian posterior computation. p99 over 1,000 iterations.

**Rationale**: The risk controller's `decision_timeout_ms` default is 50ms.
The SLO target of 5ms provides a 10x margin before the fail-closed timeout
would trigger.

**Breach response**: If decision latency approaches 50ms, the controller
falls back to deny (fail-closed). Optimize posterior calculation or reduce
window size.

#### SLO-07: Env Var Blocklist Coverage

**Target**: >= 90% of a canonical list of known secret environment variable
names are blocked by `is_env_var_allowed()`.

**Measurement**: Maintain a canonical list of 100+ known secret variable
names (covering major cloud providers, databases, CI systems, AI APIs).
Test each against `is_env_var_allowed()`. Count blocked / total.

**Canonical list sources**: OWASP secret patterns, CIS benchmarks, major
cloud provider documentation (AWS, GCP, Azure, Anthropic, OpenAI).

**Rationale**: The deny-list approach (baseline audit G-3) inherently
cannot reach 100% coverage. The 90% target acknowledges this limitation
while requiring strong baseline coverage of known patterns.

**Breach response**: Add missing patterns to `BLOCKED_EXACT`, `BLOCKED_SUFFIXES`,
or `BLOCKED_PREFIXES`. Consider adding an optional allow-list mode for
high-security deployments.

#### SLO-08: Dangerous Capability Default-Deny Rate

**Target**: 100%. Under the default configuration (`StandardProfile` with
no overrides), `exec` and `env` capabilities must be denied for all
extensions.

**Measurement**: Construct default `ExtensionPolicy` and call
`evaluate_for("exec", None)` and `evaluate_for("env", None)`. Both must
return `Deny`.

**Rationale**: This is the most critical security property (INV-007). Any
failure means dangerous capabilities are implicitly available.

**Breach response**: Immediate build failure. Fix `ExtensionPolicy::default()`
or `PolicyProfile::to_policy()`.

#### SLO-09: Ledger Integrity Verification

**Target**: 100%. Hash-chain verification succeeds for all ledger entries
after any sequence of appends, including ring-buffer truncation.

**Measurement**: Existing tests `shared_dispatch_runtime_risk_ledger_is_tamper_evident`,
`..._replay_reconstructs_decision_path`, `..._verifies_after_ring_buffer_truncation`.

**Rationale**: Tamper-evident logging is the foundation of incident response.
Any chain break indicates either a bug or active tampering.

**Breach response**: Immediate build failure. Investigate hash computation
or entry serialization for determinism issues.

#### SLO-10: Runtime Risk False-Positive Rate

**Target**: <= 10% of benign extensions trigger `Harden` or `Deny` actions
when the runtime risk controller is enabled.

**Measurement**: Run the conformance corpus (60 official + 58 community
extensions) through a simulated session with the runtime risk controller
enabled. Count extensions that receive `Harden`/`Deny`/`Terminate` actions
for normal operations.

**Rationale**: A 10% false-positive budget is generous during initial
calibration. It allows the risk controller to be enabled by default
(addressing baseline audit G-7) without blocking legitimate workloads.
The budget should tighten to <= 5% after calibration (tracked in
`bd-3i9da`, `bd-cu17q`).

**Breach response**: Tune risk score thresholds and Bayesian priors.
Adjust per-capability base scores. Add conformal prediction residual
compensation.

#### SLO-11: Runtime Risk False-Negative Rate

**Target**: <= 5% of hostile extensions evade detection when the runtime
risk controller is enabled.

**Measurement**: Run the adversarial corpus (`bd-3jpm3`) through a
simulated session. Count hostile extensions that do NOT trigger `Deny`
or `Terminate` for dangerous hostcall patterns (burst exec, env probing,
HTTP exfiltration).

**Rationale**: A missed hostile extension is a direct security failure.
The 5% budget allows for sophisticated evasion techniques while requiring
strong baseline detection.

**Breach response**: Add adversarial patterns to the corpus. Tune risk
scoring. Investigate false-negative root causes (too-low base scores,
insufficient window size, posterior convergence issues).

#### SLO-12: Capability Mapping Completeness

**Target**: 100%. Every `HostcallKind` variant and tool name in
`required_capability_for_host_call_static()` has a mapping. No `None`
returns for valid method+params combinations.

**Measurement**: Enumerate all `HostcallKind` variants and tool names.
Call `required_capability_for_host_call_static()` for each. Verify
non-None return.

**Rationale**: An unmapped hostcall method means a capability check is
skipped, creating a policy bypass (T2).

**Breach response**: Immediate build failure. Add mapping for the new
method/tool. Add test coverage.

#### SLO-13: Audit Gap Closure Velocity

**Target**: >= 1 baseline audit gap (G-*) closed per sprint (2-week
cycle).

**Measurement**: Count gaps from `docs/security/baseline-audit.md` that
transition from "Open" to "Closed" each sprint. A gap is closed when
its remediation code is merged and its verification test passes in CI.

**Rationale**: The baseline audit identified 7 gaps. At 1/sprint
velocity, all gaps would be addressed within 2 quarters. This prevents
security debt from accumulating indefinitely.

**Breach response**: Escalate to project lead. Re-prioritize gap
remediation in sprint planning.

#### SLO-14: Runtime Overhead Budget

**Target**: <= 3% total wall-time overhead from security layers (policy
evaluation + runtime risk + logging) compared to a no-security baseline.

**Measurement**: Benchmark a standardized hostcall workload (100
read/write/tool calls) with security layers enabled vs disabled. Measure
total wall-time difference as percentage.

**Rationale**: Security layers must not create perceptible performance
degradation. The 3% budget covers the policy check (~1ms), risk scoring
(~5ms amortized over many calls), and structured logging overhead.

**Breach response**: Profile overhead sources. Optimize hot paths.
Consider lazy evaluation for non-dangerous capabilities.

## 3. Risk Budgets

### 3.1 Error Budget Definitions

| Budget | Scope | Allowance | Reset Period |
|---|---|---|---|
| Invariant violations | INV-001..012 | 0 (zero tolerance) | Per build |
| Scanner false positives | Conformance corpus | 5% of corpus size | Per release |
| Scanner false negatives | Adversarial corpus | 5% of corpus size | Per release |
| Runtime risk FP | Benign corpus | 10% (initial), 5% (post-calibration) | Per release |
| Runtime risk FN | Adversarial corpus | 5% of corpus size | Per release |
| Policy latency budget | p99 of eval_for | 1ms | Per benchmark run |
| Overhead budget | Total security overhead | 3% wall-time | Per benchmark run |

### 3.2 Budget Exhaustion Rules

When an error budget is exhausted:

1. **Zero-tolerance budgets** (SLO-01, SLO-08, SLO-09, SLO-12): Build
   fails immediately. No merge permitted. Fix is blocking.

2. **Detection budgets** (SLO-02, SLO-03, SLO-10, SLO-11): Release is
   blocked until the budget is restored. Investigation root cause analysis
   is required before re-release.

3. **Performance budgets** (SLO-04, SLO-05, SLO-06, SLO-14): Warning at
   80% of budget. Release blocked at 100%. Performance regression must be
   addressed before promotion.

4. **Velocity budgets** (SLO-13): Warning if no gap closed in current
   sprint. Escalation if 2 consecutive sprints without closure.

## 4. CI Gate Matrix

### 4.1 Gate Definitions

| Gate | Phase | Trigger | Blocking | SLOs Checked |
|---|---|---|---|---|
| G-CI-1 | Pre-merge | Every PR | Yes | SLO-01, SLO-08, SLO-12 |
| G-CI-2 | Post-merge | Every merge to `main` | Yes | SLO-01, SLO-08, SLO-09, SLO-12 |
| G-CI-3 | Nightly | Scheduled (daily) | Advisory | SLO-02, SLO-03, SLO-07 |
| G-CI-4 | Pre-release | Before version tag | Yes | All SLOs |
| G-CI-5 | Post-release | After deployment | Advisory | SLO-10, SLO-11, SLO-14 |

### 4.2 Gate Details

#### G-CI-1: Pre-Merge Gate

**Trigger**: Every pull request targeting `main`.

**Tests run**:
```bash
cargo fmt --check
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo test --lib --test capability_policy_scoped \
                 --test extensions_policy_negative \
                 --test config_edge_cases
```

**SLOs verified**:
- SLO-01: All invariant tests pass (INV-001..012 test functions).
- SLO-08: Default deny for dangerous capabilities verified.
- SLO-12: Capability mapping completeness (all hostcall kinds mapped).

**Pass criteria**: All tests pass. Zero clippy warnings. Format check clean.

**Fail action**: PR merge blocked. Author must fix and re-push.

#### G-CI-2: Post-Merge Gate

**Trigger**: Every merge commit to `main`.

**Tests run**:
```bash
cargo test
```

**SLOs verified**:
- SLO-01, SLO-08, SLO-12 (from G-CI-1, full test suite).
- SLO-09: Ledger integrity tests (hash-chain, replay, truncation).

**Pass criteria**: Full test suite passes.

**Fail action**: Revert merge or fix-forward within 4 hours.

#### G-CI-3: Nightly Gate

**Trigger**: Scheduled daily run (e.g., 02:00 UTC).

**Tests run**:
```bash
# Scanner corpus tests
cargo test conformance
cargo test compatibility_scanner

# Env var blocklist coverage test
cargo test env_var_blocklist_coverage
```

**SLOs verified**:
- SLO-02: Scanner detection rate against adversarial corpus.
- SLO-03: Scanner false-positive rate against conformance corpus.
- SLO-07: Env var blocklist coverage against canonical secret list.

**Pass criteria**: Detection >= 95%, FP <= 5%, blocklist >= 90%.

**Fail action**: Create issue. Fix in next sprint. Does not block merges.

#### G-CI-4: Pre-Release Gate

**Trigger**: Before tagging a version release.

**Tests run**:
```bash
# Full test suite
cargo test

# Performance benchmarks
cargo bench --bench security_overhead

# Adversarial corpus (requires bd-3jpm3 corpus)
cargo test adversarial_corpus

# All nightly checks
```

**SLOs verified**: All 14 SLOs.

**Pass criteria**: All SLOs within targets. No zero-tolerance violations.
Performance within budget.

**Fail action**: Release blocked. Fix all violations. Re-run gate.

#### G-CI-5: Post-Release Gate

**Trigger**: After deployment to production/staging.

**Measurements**:
- Runtime risk FP/FN rates from telemetry (SLO-10, SLO-11).
- Security overhead from production metrics (SLO-14).

**Pass criteria**: SLOs within targets over 7-day observation window.

**Fail action**: If FP > 10% or FN > 5%, trigger rollback evaluation.
If overhead > 3%, investigate regression.

### 4.3 Gate-to-SLO Mapping

| SLO | G-CI-1 | G-CI-2 | G-CI-3 | G-CI-4 | G-CI-5 |
|---|---|---|---|---|---|
| SLO-01 Invariants | X | X | | X | |
| SLO-02 Scanner detection | | | X | X | |
| SLO-03 Scanner FP | | | X | X | |
| SLO-04 Policy latency | | | | X | |
| SLO-05 Dispatch latency | | | | X | |
| SLO-06 Risk latency | | | | X | |
| SLO-07 Env blocklist | | | X | X | |
| SLO-08 Dangerous deny | X | X | | X | |
| SLO-09 Ledger integrity | | X | | X | |
| SLO-10 Risk FP | | | | X | X |
| SLO-11 Risk FN | | | | X | X |
| SLO-12 Cap mapping | X | X | | X | |
| SLO-13 Gap velocity | | | | X | |
| SLO-14 Overhead | | | | X | X |

## 5. Rollout Phases and Promotion Criteria

### 5.1 Phase Definitions

| Phase | Environment | Duration | Promotion Criteria |
|---|---|---|---|
| Alpha | Dev/test only | >= 1 week | G-CI-1, G-CI-2 pass. All zero-tolerance SLOs met. |
| Beta | Internal staging | >= 2 weeks | G-CI-3 nightly clean. SLO-10 FP <= 10%. SLO-14 overhead <= 3%. |
| RC | Staged rollout (10%) | >= 1 week | G-CI-5 telemetry within targets. No rollback triggers. |
| GA | Full release | Indefinite | All SLOs sustained. G-CI-4 clean. |

### 5.2 Promotion Decision Table

| From | To | Required Artifacts | Approval |
|---|---|---|---|
| Alpha | Beta | G-CI-1 + G-CI-2 pass logs. Invariant test results. | Automated (CI) |
| Beta | RC | G-CI-3 nightly results (7 days clean). Benchmark results. | Project lead |
| RC | GA | G-CI-5 telemetry report (7 days). No rollback events. | Project lead + security review |

### 5.3 Abort Criteria

Rollout is aborted (rolled back to previous phase) if any of these occur:

| Condition | Threshold | Action |
|---|---|---|
| Invariant test failure | Any single failure | Immediate abort. Fix required. |
| Dangerous cap allowed under default config | Any instance | Immediate abort. Fix required. |
| Ledger hash-chain break | Any instance | Immediate abort. Investigate tampering vs bug. |
| Runtime risk FN rate | > 10% (2x budget) | Abort RC/GA. Return to Beta for calibration. |
| Runtime risk FP rate | > 20% (2x budget) | Abort RC/GA. Return to Beta for calibration. |
| Security overhead | > 5% wall-time | Abort RC. Profile and optimize before retry. |
| Scanner detection rate | < 90% | Block release. Add detection rules. |

## 6. Rollback Thresholds

### 6.1 Automatic Rollback Triggers

These conditions trigger automated rollback without human approval:

| Trigger | Detection Method | Rollback Target |
|---|---|---|
| Invariant test failure in production | G-CI-5 monitoring | Previous GA version |
| Capability bypass detected | Runtime risk telemetry | Previous GA version |
| Ledger corruption | Periodic verification job | Previous GA version |

### 6.2 Manual Rollback Triggers

These conditions require human evaluation before rollback:

| Trigger | Detection Method | Evaluation Criteria |
|---|---|---|
| FP rate sustained > 15% for 48h | G-CI-5 telemetry | Assess user impact vs security risk |
| FN rate sustained > 8% for 24h | G-CI-5 telemetry + incident reports | Assess whether hostile extensions are active |
| Performance degradation > 5% | Production metrics | Assess whether degradation is security-related |
| New threat class discovered | External advisory / CVE | Assess exposure and mitigation options |

### 6.3 Rollback Verification

After any rollback:

1. Verify previous version passes G-CI-4 (pre-release gate).
2. Run adversarial corpus to confirm detection rates restored.
3. Record rollback event with: timestamp, trigger, previous version,
   rollback version, root cause (if known).
4. Create post-incident bead for root cause analysis.

## 7. Incident Response Thresholds

### 7.1 Severity Classification

| Severity | Condition | Response Time | Examples |
|---|---|---|---|
| P0 Critical | Security boundary breached | < 1 hour | Capability bypass, secret exfiltration, sandbox escape |
| P1 High | Control degraded | < 4 hours | Scanner detection < 90%, risk FN > 10%, ledger break |
| P2 Medium | SLO budget at risk | < 1 sprint | FP/FN approaching threshold, latency warning |
| P3 Low | Informational | Next sprint | Gap closure velocity below target, minor blocklist gaps |

### 7.2 Escalation Path

```
P3 -> Sprint backlog item
P2 -> Sprint priority item + project lead notification
P1 -> Immediate fix + rollback evaluation + project lead approval
P0 -> Immediate rollback + all-hands investigation + post-mortem required
```

## 8. SLO Review and Evolution

### 8.1 Review Schedule

| Review Type | Frequency | Scope |
|---|---|---|
| SLO target review | Quarterly | Tighten targets based on calibration data |
| Error budget review | Monthly | Assess budget consumption trends |
| Gate effectiveness review | Per release | Evaluate whether gates caught issues |
| Threat model alignment | Per SEC-1.1 update | Ensure SLOs cover new threats |

### 8.2 Planned SLO Target Tightening

| SLO | Current Target | Post-Calibration Target | Timeline |
|---|---|---|---|
| SLO-10 Risk FP | <= 10% | <= 5% | After bd-3i9da, bd-cu17q complete |
| SLO-03 Scanner FP | <= 5% | <= 2% | After bd-21vng complete |
| SLO-14 Overhead | <= 3% | <= 1% | After performance optimization pass |

## 9. Artifact Manifest

| Artifact | Role | Determinism Rule |
|---|---|---|
| `docs/security/security-slos.md` | SEC-1.4 SLO definitions | Hash via `sha256sum docs/security/security-slos.md` |
| `docs/security/threat-model.md` | SEC-1.1 threat baseline | Cross-reference for threat IDs |
| `docs/security/invariants.md` | SEC-1.2 invariant definitions | Cross-reference for INV IDs |
| `docs/security/invariants.machine.json` | Machine-checkable invariant manifest | Test function mappings |
| `docs/security/baseline-audit.md` | SEC-1.3 gap inventory | Cross-reference for G-* IDs |
