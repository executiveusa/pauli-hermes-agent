# Security Operator Quick Reference

Quick command and API reference for daily security operations.
For detailed procedures, see the Incident Response Runbook and Policy Tuning Guide.

## Environment Variables

```bash
# Master switches
PI_EXTENSION_RISK_ENABLED=true      # Enable runtime risk controller
PI_EXTENSION_RISK_ENFORCE=true       # Enable enforcement (false = shadow mode)
PI_EXTENSION_RISK_FAIL_CLOSED=true   # Deny on controller errors

# Tuning
PI_EXTENSION_RISK_ALPHA=0.01         # Type-I error budget (1e-6..0.5)
PI_EXTENSION_RISK_WINDOW=128         # Sliding window size (8..4096)
PI_EXTENSION_RISK_LEDGER_LIMIT=2048  # Max ledger entries (32..20000)
PI_EXTENSION_RISK_DECISION_TIMEOUT_MS=50  # Decision budget ms (1..2000)

# Policy profile
PI_EXTENSION_POLICY=standard         # safe | standard | permissive
```

## Rollout Phases

| Phase | `enforce` flag | Description |
|-------|---------------|-------------|
| `shadow` | `false` | Score + telemetry only, no blocking |
| `log_only` | `false` | Log would-be actions, no blocking |
| `enforce_new` | `true` | Enforce for newly loaded extensions |
| `enforce_all` | `true` | Full enforcement |

### Phase Operations (Programmatic API)

```rust
// Read current phase
let state: RolloutState = manager.rollout_state();
println!("Phase: {}, Enforce: {}", state.phase, state.enforce);

// Advance to next phase
let changed: bool = manager.advance_rollout();

// Set explicit phase (forward or backward)
manager.set_rollout_phase(RolloutPhase::Shadow);

// Configure rollback triggers
manager.set_rollback_trigger(RollbackTrigger {
    max_false_positive_rate: 0.05,
    max_error_rate: 0.10,
    window_size: 100,
    max_latency_ms: 200,
});

// Record a decision for rollback evaluation
let rollback_triggered: bool = manager.record_rollout_decision(
    latency_ms,    // decision latency
    was_error,     // controller error?
    was_fp,        // operator-flagged false positive?
);
```

## Enforcement States

```
Allow → Harden → Prompt → Deny → Terminate
  0        1        2       3        4
```

- **Allow:** Normal operation, no restrictions
- **Harden:** Dangerous capabilities blocked, safe ones allowed
- **Prompt:** User confirmation required before proceeding
- **Deny:** Call blocked entirely
- **Terminate:** Extension quarantined (3+ consecutive unsafe)

## Risk Ledger Operations

```rust
// Export ledger
let ledger = manager.runtime_risk_ledger_artifact();

// Verify hash chain integrity
let report = verify_runtime_risk_ledger_artifact(&ledger);
assert!(report.valid);

// Export telemetry
let telemetry = manager.runtime_hostcall_telemetry_artifact();
```

## Security Alerts

```rust
// Read alert stream
let alerts: Vec<SecurityAlert> = manager.security_alerts();

// Each alert contains:
// - schema, ts_ms, sequence_id
// - extension_id, capability, method
// - action_taken, reason, risk_score
```

## Kill-Switch Operations

```rust
// Activate kill-switch for an extension
manager.set_kill_switch("extension-id", true, "incident-2024-001");

// Deactivate
manager.set_kill_switch("extension-id", false, "cleared-after-investigation");

// Check trust state
let trust = manager.trust_state("extension-id");
```

## Score Band Thresholds (by profile)

| | Safe | Balanced | Permissive |
|---|------|----------|------------|
| Harden | 0.30 | 0.40 | 0.55 |
| Prompt | 0.50 | 0.60 | 0.70 |
| Deny | 0.65 | 0.75 | 0.85 |
| Terminate | 0.80 | 0.90 | 0.95 |

## Rollback Trigger Defaults

| Threshold | Value | Action when breached |
|-----------|-------|---------------------|
| FP rate | > 5% | Auto-rollback to Shadow |
| Error rate | > 10% | Auto-rollback to Shadow |
| Avg latency | > 200ms | Auto-rollback to Shadow |
| Min samples | 10 | No evaluation below this |

## Evidence Bundle Operations

```rust
use pi::extensions::{
    build_incident_evidence_bundle, verify_incident_evidence_bundle,
    replay_runtime_risk_ledger_artifact,
    IncidentBundleFilter, IncidentBundleRedactionPolicy,
    SecurityAlertCategory, SecurityAlertSeverity,
};

// Build a bundle (scoped to an extension and time window)
let filter = IncidentBundleFilter {
    start_ms: Some(start), end_ms: Some(end),
    extension_id: Some("ext-id".into()),
    alert_categories: None,  // or Some(vec![...])
    min_severity: None,       // or Some(SecurityAlertSeverity::Warning)
};
let redaction = IncidentBundleRedactionPolicy::default(); // redact all hashes
let bundle = build_incident_evidence_bundle(
    &ledger, &alerts, &telemetry, &exec, &secret,
    &quota_breaches, &filter, &redaction, now_ms,
);

// Verify bundle integrity
let report = verify_incident_evidence_bundle(&bundle);
assert!(report.valid);

// Forensic replay
let replay = replay_runtime_risk_ledger_artifact(&ledger)?;
```

## Quota Configuration

```rust
// Per-extension quota via policy overrides
let policy = ExtensionPolicy {
    per_extension: HashMap::from([(
        "ext-id".into(),
        ExtensionOverride {
            quota: Some(ExtensionQuotaConfig {
                max_hostcalls_per_second: Some(10),
                max_hostcalls_per_minute: Some(100),
                max_hostcalls_total: Some(5000),
                max_subprocesses: Some(2),
                max_write_bytes: Some(10_000_000),
                max_http_requests: Some(50),
            }),
            ..Default::default()
        },
    )]),
    ..Default::default()
};
```

## Exec Mediation

```rust
// Configure exec mediation
let policy = ExtensionPolicy {
    exec_mediation: ExecMediationPolicy {
        enabled: true,
        deny_threshold: ExecRiskTier::High,
        deny_patterns: vec!["rm -rf /".into()],
        allow_patterns: vec!["rm -rf ./node_modules".into()],
        audit_all_classified: true,
    },
    ..Default::default()
};
```

## Secret Broker

```rust
// Configure secret broker
let policy = ExtensionPolicy {
    secret_broker: SecretBrokerPolicy {
        enabled: true,
        secret_suffixes: vec!["_KEY", "_SECRET", "_TOKEN"],
        secret_prefixes: vec!["SECRET_", "AUTH_"],
        secret_exact: vec!["ANTHROPIC_API_KEY"],
        disclosure_allowlist: vec!["HOME", "PATH"],
        redaction_placeholder: "[REDACTED]".into(),
    },
    ..Default::default()
};
```

## Alert Categories (SecurityAlertCategory)

| Enum Variant | Meaning |
|-------------|---------|
| `PolicyDenial` | Denied by static capability policy |
| `AnomalyDenial` | Denied by runtime risk scorer |
| `ExecMediation` | Shell command blocked |
| `SecretBroker` | Secret detected/redacted |
| `QuotaBreach` | Quota exceeded |
| `Quarantine` | Extension terminated |
| `ProfileTransition` | Profile transition attempt |

## Common Operations Cheatsheet

| Task | Method |
|------|--------|
| Enable risk controller | `PI_EXTENSION_RISK_ENABLED=true` |
| Start in shadow mode | `PI_EXTENSION_RISK_ENFORCE=false` |
| Check current phase | `manager.rollout_state()` |
| Advance rollout | `manager.advance_rollout()` |
| Emergency rollback | `manager.set_rollout_phase(RolloutPhase::Shadow)` |
| Kill extension | `manager.set_kill_switch(id, true, reason)` |
| Verify ledger | `verify_runtime_risk_ledger_artifact(&ledger)` |
| Build evidence bundle | `build_incident_evidence_bundle(...)` |
| Verify bundle | `verify_incident_evidence_bundle(&bundle)` |
| Replay decisions | `replay_runtime_risk_ledger_artifact(&ledger)` |
| Check FP rate | `manager.rollout_state().window_stats` |
