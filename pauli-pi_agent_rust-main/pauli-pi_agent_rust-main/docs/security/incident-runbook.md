# Security Incident Runbook

> SEC-7.3 (bd-2kle2) -- Step-by-step procedures for common security incident categories.

---

## Table of Contents

1. [General Incident Workflow](#general-incident-workflow)
2. [INC-1: Unauthorized Capability Use](#inc-1-unauthorized-capability-use)
3. [INC-2: Secret Exposure](#inc-2-secret-exposure)
4. [INC-3: Ledger Integrity Failure](#inc-3-ledger-integrity-failure)
5. [INC-4: Scanner Bypass](#inc-4-scanner-bypass)
6. [INC-5: Quarantine Escape](#inc-5-quarantine-escape)
7. [INC-6: Resource Quota Abuse](#inc-6-resource-quota-abuse)
8. [Evidence Collection](#evidence-collection)
9. [Rollback Procedures](#rollback-procedures)

---

## General Incident Workflow

Every incident follows this sequence regardless of classification:

```
DETECT  →  CONTAIN  →  COLLECT  →  VERIFY  →  ANALYZE  →  REMEDIATE  →  POSTMORTEM
```

| Phase | SLA | Actions |
|-------|-----|---------|
| Detect | Automatic | Alert raised by risk controller or CI gate |
| Contain | < 5 min | Kill extension, verify isolation |
| Collect | < 15 min | Export evidence bundle, snapshot config |
| Verify | < 30 min | Verify bundle hash, ledger chain, alert counts |
| Analyze | < 2 hours | Root cause analysis with forensic replay |
| Remediate | Varies | Policy update, scanner rule, bead filed |
| Postmortem | < 24 hours | Written, reviewed, artifacts archived |

---

## INC-1: Unauthorized Capability Use

**Symptoms:** Policy violation alert; a denied capability was somehow exercised.

### Investigation

1. **Identify the call.**
   - Check security alerts filtered by category `policy_violation`
   - Note the `extension_id`, `capability`, `method`, and `call_id`

2. **Check policy evaluation path.**
   ```bash
   pi --explain-extension-policy
   ```
   - Verify the capability is in `deny_caps`
   - Check if per-extension overrides exist for this extension

3. **Examine the dispatch path.**
   - If the call succeeded despite a Deny decision, this indicates a bypass in the dispatch pipeline
   - Check `dispatch_host_call_shared()` -- the capability must be derived server-side via `required_capability_for_host_call_static()`, NOT from the caller's `capability` field

4. **Verify invariants.**
   ```bash
   cargo test --test policy_profile_hardening -- --nocapture
   cargo test --test capability_denial_matrix -- --nocapture
   ```

### Expected Artifacts

- [ ] Security alert with category `policy_violation`
- [ ] Risk ledger entry showing the denied call
- [ ] Policy explanation snapshot at time of incident

### Remediation

- If policy misconfiguration: fix config, verify with `--explain-extension-policy`
- If bypass bug: file Critical bead, add regression test in `capability_denial_matrix.rs`
- Reference: Invariant INV-001 (policy precedence chain)

---

## INC-2: Secret Exposure

**Symptoms:** Secret broker alert; environment variable value exposed to an extension.

### Investigation

1. **Identify the exposure.**
   - Check alerts with category `secret_exposure`
   - Determine which env var was exposed and to which extension

2. **Check the secret broker configuration.**
   - Default blocklist: 26 exact names, 11 suffixes (`*_API_KEY`, `*_SECRET`, etc.), 2 prefixes
   - Verify the exposed variable matches a blocklist pattern
   - Check if the variable was in the `disclosure_allowlist` (intentional pass-through)

3. **Verify the broker was enabled.**
   - `SecretBrokerPolicy.enabled` must be `true`
   - Check `secret_broker_enabled` in the policy explanation

4. **Examine the exposure path.**
   - Was it via `env.read` hostcall?
   - Was it via `exec` (child process inheriting env)?
   - Was it via `http` (sent as header/body)?

### Expected Artifacts

- [ ] Secret broker ledger entry with `name_hash` of the exposed variable
- [ ] Alert with category `secret_exposure`
- [ ] Incident evidence bundle with `secret_broker` section populated

### Remediation

- **Immediate:** Rotate the exposed credential
- **Short-term:** Add the variable pattern to the blocklist if not already covered
- **Long-term:** If exposure was via `exec`, verify exec mediation blocks env passthrough
- Reference: Invariant INV-009 (secret broker coverage)

---

## INC-3: Ledger Integrity Failure

**Symptoms:** Hash chain verification fails; `verify_runtime_risk_ledger_artifact()` reports errors.

### Investigation

1. **Run ledger verification.**
   The verification function checks:
   - Schema version matches expected
   - Each entry's `ledger_hash` matches recomputed hash
   - Chain linkage: each `prev_ledger_hash` matches the prior entry's hash
   - No missing or duplicate entries

2. **Identify the break point.**
   - Check `verification_errors` in the report
   - The first broken link indicates where tampering or corruption occurred

3. **Distinguish corruption from tampering.**
   - **Corruption:** Typically a single entry with bad hash (memory issue, serialization bug)
   - **Tampering:** Modified/removed entries break the chain at the point of alteration

4. **Forensic replay.**
   - Replay the ledger up to the break point to reconstruct valid history
   - Entries after the break are untrusted and must be re-evaluated

### Expected Artifacts

- [ ] `RuntimeRiskLedgerVerificationReport` with `chain_valid: false`
- [ ] Specific `verification_errors` listing broken entries
- [ ] Evidence bundle with `risk_ledger` section

### Remediation

- If corruption: investigate OOM or serialization bugs, file bead
- If tampering: this is Critical severity -- full security audit required
- Reference: Invariant INV-010 (ledger integrity)

---

## INC-4: Scanner Bypass

**Symptoms:** Malicious or dangerous code was loaded despite compatibility scanning.

### Investigation

1. **Examine the extension source.**
   - Check for forbidden imports that the scanner should have caught:
     `process.binding`, `eval`, `Function()`, `require('child_process')`, etc.

2. **Review scan results.**
   - The `CompatibilityScanner` classifies entries as `forbidden`, `flagged`, or `safe`
   - Check if the dangerous pattern was in a form the scanner doesn't recognize

3. **Test scanner detection.**
   ```bash
   cargo test --test install_time_security_scanner -- --nocapture
   ```

4. **Check for evasion techniques.**
   - Dynamic requires: `require(variable)` vs `require('child_process')`
   - String concatenation: `eval('rm' + ' -rf')`
   - Encoded payloads: base64 or unicode escape sequences
   - Prototype pollution paths

### Expected Artifacts

- [ ] Scanner results for the extension (compat ledger)
- [ ] The extension source code
- [ ] Evidence of what the extension actually executed

### Remediation

- Add the evasion pattern to the scanner's forbidden/flagged patterns
- Add a regression test in `install_time_security_scanner.rs`
- Reference: SLO-02 (scanner detection >= 95%)

---

## INC-5: Quarantine Escape

**Symptoms:** An extension marked as quarantined resumed execution without explicit trust promotion.

### Investigation

1. **Check trust state transitions.**
   - `ExtensionTrustTracker` manages the state machine: Unknown -> Probation -> Trusted / Quarantined
   - Verify the state transition log shows Quarantined -> Active without a valid promotion

2. **Check the kill-switch state.**
   - If a kill-switch was active, verify it was not prematurely lifted
   - Kill-switch lifted events should include audit metadata

3. **Verify `is_hostcall_allowed_for_trust()` enforcement.**
   - This function gates hostcalls based on trust state
   - Quarantined extensions should be blocked from all hostcalls

### Expected Artifacts

- [ ] Trust state transition history
- [ ] Kill-switch activation/deactivation events
- [ ] Security alerts with the quarantined extension's ID

### Remediation

- If state machine bug: fix the transition logic, add regression test
- If promotion was unauthorized: investigate who/what triggered it
- Reference: Threat T4 (runtime abuse escalation)

---

## INC-6: Resource Quota Abuse

**Symptoms:** Extension exceeds resource quotas (memory, API calls, time).

### Investigation

1. **Check quota breach events.**
   - `drain_quota_breach_events()` returns all recorded breaches
   - Each breach includes: extension_id, resource type, limit, actual, timestamp

2. **Verify quota configuration.**
   - Default: `max_memory_mb: 256`
   - Per-extension overrides via `ExtensionOverride.quota`

3. **Assess impact.**
   - Did the breach cause OOM? Check system logs
   - Did it degrade other extensions? Check isolation

### Expected Artifacts

- [ ] Quota breach events in incident evidence bundle
- [ ] Memory/resource usage telemetry
- [ ] Alert with category `quota_breach`

### Remediation

- Tighten quota for the offending extension
- If legitimate high usage: increase quota with documentation
- Reference: SLO-14 (runtime overhead <= 3%)

---

## Evidence Collection

### Export an Incident Evidence Bundle

The incident evidence bundle aggregates all security artifacts into a single, hash-verified package.

**Components included:**
- Runtime risk ledger (hash-chained entries)
- Security alerts (filtered by time/extension/category/severity)
- Hostcall telemetry (call timing and context)
- Exec mediation log (command allow/deny decisions)
- Secret broker log (redaction events)
- Quota breach events
- Risk replay (reconstructed decision timeline)
- Summary statistics

**Bundle integrity:**
- `bundle_hash`: SHA-256 over all content sections
- `schema`: `pi.security.incident_evidence_bundle.v1`
- Verify with `verify_incident_evidence_bundle()`

### Filtering

Bundles support scoping via `IncidentBundleFilter`:
- `start_ms` / `end_ms`: Time range
- `extension_id`: Specific extension
- `alert_categories`: Specific alert types
- `min_severity`: Minimum severity threshold

### Redaction

Bundles support redaction via `IncidentBundleRedactionPolicy`:
- `redact_params_hash`: Redact parameter fingerprints
- `redact_context_hash`: Redact context hashes
- `redact_args_shape_hash`: Redact argument shape hashes
- `redact_command_hash`: Redact command hashes
- `redact_name_hash`: Redact name hashes
- `redact_remediation`: Redact remediation text

Use full redaction for external sharing; use no redaction for internal forensics.

### Verification Steps

After collecting evidence:

1. Verify bundle hash matches recomputed hash
2. Verify ledger chain integrity (no broken links)
3. Verify summary counts match actual sub-artifact counts
4. Verify schema version matches expected
5. If redaction was applied, verify no raw values leak

---

## Rollback Procedures

### Rolling Back a Policy Change

```json
// Before: settings.json (problematic)
{
  "extensionPolicy": {
    "profile": "permissive",
    "allowDangerous": true
  }
}

// After: rollback to safe defaults
{
  "extensionPolicy": {
    "profile": "safe",
    "allowDangerous": false
  }
}
```

Verification:
```bash
pi --explain-extension-policy  # Confirm safe profile active
cargo test --test security_conformance_benign -- --nocapture  # Confirm compatibility
```

### Rolling Back Risk Controller Settings

```json
// Revert to defaults
{
  "extensionRisk": {
    "enabled": false,
    "enforce": true,
    "alpha": 0.01,
    "windowSize": 128,
    "failClosed": true
  }
}
```

Or disable entirely: set `enabled: false`. The controller stops scoring but existing ledger entries are preserved.

### Emergency: Kill All Extensions

```bash
pi --no-extensions  # Disable all extension discovery and loading
```

This is the nuclear option. Use only when containment requires complete extension isolation.
