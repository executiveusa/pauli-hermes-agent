# Security Incident Response Runbook

This runbook provides step-by-step procedures for responding to security
incidents in the pi extension runtime. It maps directly to implemented
controls in `src/extensions.rs` and `src/config.rs`.

## Severity Classification

| Level | Label | Example | Response Time | Escalation |
|-------|-------|---------|---------------|------------|
| P0 | Critical | Policy boundary breached, data exfiltration detected | < 1 hour | Immediate kill-switch + rollback |
| P1 | High | Risk controller degraded, sustained deny escalation | < 4 hours | Investigation + potential rollback |
| P2 | Medium | SLO budget at risk, elevated FP rate | < 1 sprint | Root cause analysis + tuning |
| P3 | Low | Informational, single transient anomaly | Next sprint | Log and backlog |

## Decision Tree: Incoming Alert

```
Alert received
├── Is extension quarantined (Terminate state)?
│   └── YES → Go to: Quarantine Investigation (P0/P1)
├── Is it a policy denial (exec/env denied)?
│   ├── Single occurrence → Log, monitor (P3)
│   └── Repeated burst (>3 in 60s) → Go to: Burst Denial Triage (P1)
├── Is it an enforcement state transition?
│   ├── Escalation (Allow→Harden→Deny) → Go to: Escalation Review (P2)
│   └── De-escalation → Log, verify cooldown worked (P3)
├── Is it a rollback trigger event?
│   └── YES → Go to: Automatic Rollback Verification (P1)
└── Is it a ledger integrity failure?
    └── YES → Go to: Ledger Corruption Recovery (P0)
```

---

## Procedure 1: Quarantine Investigation (P0/P1)

**Trigger:** Extension reaches `Terminate` enforcement state (3+ consecutive
unsafe evaluations).

**Step 1 — Verify quarantine is active**

Check the extension's enforcement state via the runtime risk ledger:

```rust
// Programmatic: ExtensionManager API
let state = manager.runtime_risk_config();
assert!(state.enabled, "risk controller must be enabled");

// Check per-extension state
let ledger = manager.runtime_risk_ledger_artifact();
let ext_entries: Vec<_> = ledger.entries.iter()
    .filter(|e| e.extension_id.as_deref() == Some("suspect-ext-id"))
    .collect();
// Last entry should show action = "terminate"
```

**Expected artifact:** `RuntimeRiskLedgerEntry` with `action: "terminate"` and
consecutive unsafe count >= 3.

**Step 2 — Assess threat severity**

Review the hostcall telemetry for the quarantined extension:

```rust
let telemetry = manager.runtime_hostcall_telemetry_artifact();
let suspicious: Vec<_> = telemetry.entries.iter()
    .filter(|e| e.extension_id.as_deref() == Some("suspect-ext-id"))
    .collect();
// Look for: capability="exec", unusual param patterns, resource_target_class
```

Questions to answer:
- What capabilities was the extension requesting? (`exec`, `http`, `fs/write`)
- Were the targets unusual? (system paths, external URLs)
- Was there a burst pattern? (many calls in short window)

**Step 3 — Execute response**

For confirmed threats:
1. Activate kill-switch: `manager.set_kill_switch("suspect-ext-id", true, "incident-ref")`
2. Verify: Check that subsequent calls from the extension are blocked
3. Export evidence bundle for forensic review

For false positives:
1. Document the FP in the incident log
2. Consider tuning alpha or window_size (see Policy Tuning Guide)
3. Reset extension state if appropriate

**Step 4 — Verify and close**

- Confirm kill-switch is active in trust state
- Verify ledger hash chain integrity: `verify_runtime_risk_ledger_artifact(&ledger)`
- Create post-incident bead with linked evidence

---

## Procedure 2: Burst Denial Triage (P1)

**Trigger:** 3+ policy denials from a single extension within 60 seconds.

**Step 1 — Identify the extension and denied capabilities**

Check security alerts:
```rust
let alerts = manager.security_alerts();
let recent: Vec<_> = alerts.iter()
    .filter(|a| a.extension_id.as_deref() == Some("ext-id"))
    .filter(|a| a.ts_ms > (now_ms - 60_000))
    .collect();
```

**Step 2 — Determine if legitimate**

- Is the extension newly installed? May need capability grants
- Did policy change recently? Check policy source (cli, env, config)
- Is the extension attempting capabilities it has never used before?

**Step 3 — Respond**

If legitimate need:
1. Add per-extension capability override in policy config
2. Verify the override takes effect
3. Monitor for 24 hours

If suspicious:
1. Let denial continue (policy is working correctly)
2. Escalate to P0 if extension appears to be probing permissions
3. Consider activating kill-switch preemptively

---

## Procedure 3: Escalation Review (P2)

**Trigger:** Enforcement state escalated (e.g., Allow → Harden, or Harden → Deny).

**Step 1 — Review the transition**

```rust
// The enforcement transition is recorded in telemetry
let telemetry = manager.runtime_hostcall_telemetry_artifact();
// Look for entries where risk_action changed
```

**Step 2 — Evaluate if expected**

- Check the risk score that triggered escalation against score band thresholds
- Review the extension's recent hostcall pattern
- Verify hysteresis is working (no rapid flapping)

**Step 3 — Tune if needed**

If escalation is too aggressive:
- Increase score band thresholds (see Policy Tuning Guide)
- Increase `cooldown_calls` for slower de-escalation
- Consider switching to a more permissive policy profile

If escalation is correct:
- Log and monitor
- No action needed — the state machine is working as designed

---

## Procedure 4: Automatic Rollback Verification (P1)

**Trigger:** Rollback trigger fired, rollout phase reverted to Shadow.

**Step 1 — Verify rollback occurred**

```rust
let state = manager.rollout_state();
assert_eq!(state.phase, RolloutPhase::Shadow);
assert!(state.rolled_back_from.is_some());
// rolled_back_from tells you what phase we were in
```

**Step 2 — Identify trigger cause**

Check the window statistics:
```rust
let stats = state.window_stats;
// Check which threshold was breached:
// - stats.false_positive_count / stats.total_decisions > max_false_positive_rate?
// - stats.error_count / stats.total_decisions > max_error_rate?
// - stats.avg_latency_ms > max_latency_ms?
```

**Step 3 — Root cause analysis**

| Trigger | Likely Cause | Remediation |
|---------|-------------|-------------|
| High FP rate | Score thresholds too aggressive | Raise band thresholds or alpha |
| High error rate | Controller bug or resource exhaustion | Check logs, fix bug, increase timeout |
| High latency | Window size too large or system load | Reduce window_size or decision_timeout_ms |

**Step 4 — Recovery**

1. Fix the root cause
2. Gradually re-advance rollout: `manager.advance_rollout()`
3. Monitor window stats at each phase before advancing further
4. Document in incident bead

---

## Procedure 5: Ledger Corruption Recovery (P0)

**Trigger:** Hash chain verification fails on the runtime risk ledger.

**Step 1 — Confirm corruption**

```rust
let ledger = manager.runtime_risk_ledger_artifact();
let report = verify_runtime_risk_ledger_artifact(&ledger);
assert!(!report.valid, "corruption confirmed");
// report.first_invalid_index tells you where the chain broke
```

**Step 2 — Assess impact**

- How many entries are affected?
- When did corruption start? (check timestamps)
- Is the risk controller still making correct decisions?

**Step 3 — Immediate response**

1. Roll back enforcement to Shadow mode: `manager.set_rollout_phase(RolloutPhase::Shadow)`
2. Export the corrupted ledger as evidence
3. Restart the extension manager to get a fresh ledger

**Step 4 — Investigation**

- Check for concurrent modification bugs
- Check for memory corruption indicators
- Review recent code changes to ledger-touching paths

**Expected artifacts:**
- Corrupted ledger export (JSON)
- `RuntimeRiskLedgerVerificationReport` showing `valid: false`
- Incident evidence bundle

---

## Procedure 6: Evidence Bundle Export

**When to use:** After any P0-P2 incident, or when forensic evidence must be
preserved for audit or sharing.

**Step 1 -- Collect raw artifacts**

```rust
let ledger = manager.runtime_risk_ledger_artifact();
let alerts = manager.security_alert_artifact();
let telemetry = manager.runtime_hostcall_telemetry_artifact();
let exec = manager.exec_mediation_artifact();
let secret = manager.secret_broker_artifact();
let quota_breaches = manager.quota_breach_events();
```

**Step 2 -- Define scope with a filter**

```rust
use pi::extensions::{
    IncidentBundleFilter, SecurityAlertCategory, SecurityAlertSeverity,
};

let filter = IncidentBundleFilter {
    start_ms: Some(incident_start_ms),   // Time window start
    end_ms: Some(incident_end_ms),       // Time window end
    extension_id: Some("suspect-ext".into()),  // Scope to one extension
    alert_categories: Some(vec![         // Limit to relevant categories
        SecurityAlertCategory::ExecMediation,
        SecurityAlertCategory::AnomalyDenial,
    ]),
    min_severity: Some(SecurityAlertSeverity::Warning),
};
```

**Step 3 -- Set redaction policy**

For external sharing (redact all hashes):
```rust
use pi::extensions::IncidentBundleRedactionPolicy;

let redaction = IncidentBundleRedactionPolicy {
    redact_params_hash: true,
    redact_context_hash: true,
    redact_args_shape_hash: true,
    redact_command_hash: true,
    redact_name_hash: true,
    redact_remediation: false,  // Keep human-readable remediation text
};
```

For internal investigation (no redaction):
```rust
let redaction = IncidentBundleRedactionPolicy {
    redact_params_hash: false,
    redact_context_hash: false,
    redact_args_shape_hash: false,
    redact_command_hash: false,
    redact_name_hash: false,
    redact_remediation: false,
};
```

**Step 4 -- Build the bundle**

```rust
use pi::extensions::build_incident_evidence_bundle;

let bundle = build_incident_evidence_bundle(
    &ledger, &alerts, &telemetry, &exec, &secret,
    &quota_breaches, &filter, &redaction, now_ms,
);
```

**Step 5 -- Verify integrity**

```rust
use pi::extensions::verify_incident_evidence_bundle;

let report = verify_incident_evidence_bundle(&bundle);
assert!(report.valid, "Bundle integrity check failed: {:?}", report.errors);
assert!(report.ledger_chain_intact, "Ledger hash chain broken");
assert_eq!(report.bundle_hash, report.recomputed_hash, "Hash mismatch");
```

**Step 6 -- Review bundle summary**

```rust
let summary = &bundle.summary;
println!("Ledger entries: {}", summary.ledger_entry_count);
println!("Alerts: {}", summary.alert_count);
println!("Telemetry events: {}", summary.telemetry_event_count);
println!("Exec mediation: {}", summary.exec_mediation_count);
println!("Secret broker: {}", summary.secret_broker_count);
println!("Quota breaches: {}", summary.quota_breach_count);
println!("Distinct extensions: {}", summary.distinct_extensions);
println!("Peak risk score: {:.3}", summary.peak_risk_score);
println!("Deny/Terminate count: {}", summary.deny_or_terminate_count);
println!("Ledger chain intact: {}", summary.ledger_chain_intact);
```

**Step 7 -- Forensic replay (optional)**

Reconstruct the decision sequence step-by-step:

```rust
use pi::extensions::replay_runtime_risk_ledger_artifact;

let replay = replay_runtime_risk_ledger_artifact(&ledger)?;
for step in &replay.steps {
    println!("[{}] ext={} cap={}.{} score={:.3} action={:?} state={:?}",
        step.ts_ms, step.extension_id, step.capability, step.method,
        step.risk_score, step.selected_action, step.derived_state);
}
```

**Expected artifacts:**
- `IncidentEvidenceBundle` (schema `pi.ext.incident_evidence_bundle.v1`)
- `IncidentBundleVerificationReport` confirming `valid: true`
- Optional `RuntimeRiskReplayArtifact` for decision reconstruction

---

## Evidence Collection Checklist

All incidents should produce an evidence bundle containing:

1. **Runtime risk ledger** -- hash-chained decision history
2. **Hostcall telemetry** -- per-call feature vectors and scores
3. **Security alerts** -- alert stream filtered to incident timeframe
4. **Exec mediation log** -- command classifications and decisions
5. **Secret broker log** -- redaction decisions
6. **Quota breach events** -- resource limit violations
7. **Rollout state snapshot** -- phase, enforce flag, window stats
8. **Extension trust state** -- current trust level and kill-switch audit

Bundle integrity is verified via SHA-256 hashing. The
`verify_incident_evidence_bundle()` function confirms bundle hash and ledger
chain validity.

---

## Post-Incident Checklist

- [ ] Incident severity classified (P0-P3)
- [ ] Immediate response executed per procedure above
- [ ] Evidence bundle collected and hash verified
- [ ] Root cause identified
- [ ] Remediation applied (config tuning, bug fix, policy update)
- [ ] Verification steps confirm remediation worked
- [ ] Incident bead created with linked evidence artifacts
- [ ] Rollout phase restored (if rolled back) with monitoring
