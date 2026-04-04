# Security Policy Tuning Guide

Operational guide for tuning runtime risk controller parameters,
policy profiles, and enforcement thresholds in production.

## RuntimeRiskConfig Parameters

### Configuration Sources (Precedence: env > config > default)

| Parameter | Config Key | Env Var | Default | Range |
|-----------|-----------|---------|---------|-------|
| `enabled` | `extension_risk.enabled` | `PI_EXTENSION_RISK_ENABLED` | `false` | bool |
| `enforce` | `extension_risk.enforce` | `PI_EXTENSION_RISK_ENFORCE` | `true` | bool |
| `alpha` | `extension_risk.alpha` | `PI_EXTENSION_RISK_ALPHA` | `0.01` | 1e-6..0.5 |
| `window_size` | `extension_risk.windowSize` | `PI_EXTENSION_RISK_WINDOW` | `128` | 8..4096 |
| `ledger_limit` | `extension_risk.ledgerLimit` | `PI_EXTENSION_RISK_LEDGER_LIMIT` | `2048` | 32..20000 |
| `decision_timeout_ms` | `extension_risk.decisionTimeoutMs` | `PI_EXTENSION_RISK_DECISION_TIMEOUT_MS` | `50` | 1..2000 |
| `fail_closed` | `extension_risk.failClosed` | `PI_EXTENSION_RISK_FAIL_CLOSED` | `true` | bool |

### Parameter Tuning Recipes

#### Too many false positives (benign extensions blocked)

**Symptoms:** Extensions that previously worked are now denied. FP rate > 5%.

**Levers to adjust (in order):**

1. **Increase `alpha`** (Type-I error budget). Higher alpha = more tolerant.
   - Default `0.01` is conservative. Try `0.05` for initial rollout.
   - Maximum `0.5` (very permissive, use only during evaluation).

2. **Increase score band thresholds.** The score bands for each policy profile
   determine when enforcement escalates:
   - Safe: harden=0.30, deny=0.65 (most aggressive)
   - Balanced: harden=0.40, deny=0.75 (default)
   - Permissive: harden=0.55, deny=0.85 (most tolerant)
   - Switch to a more permissive profile if FP rate is high.

3. **Increase `window_size`**. Larger window smooths out transient spikes.
   - Default `128`. Try `256` or `512` for high-traffic deployments.
   - Trade-off: slower to detect real threats.

#### Too many false negatives (threats not detected)

**Symptoms:** Known-malicious patterns pass through undetected.

**Levers to adjust:**

1. **Decrease `alpha`** to `0.001` or lower for stricter detection.
2. **Switch to `safe` policy profile** with lower score band thresholds.
3. **Decrease `window_size`** to `32` or `64` for faster reaction.
4. **Ensure `fail_closed: true`** so controller errors default to deny.

#### Decision latency too high

**Symptoms:** `avg_latency_ms` in rollout stats exceeds SLO (target: 5ms p50).

**Levers to adjust:**

1. **Decrease `window_size`**. Smaller window = less computation per decision.
2. **Decrease `decision_timeout_ms`** to enforce a hard budget.
   - Default `50ms`. For latency-sensitive: `10ms`.
   - Combined with `fail_closed: true`, timed-out decisions deny.
3. **Reduce `ledger_limit`** to keep less history in memory.

---

## Policy Profiles

### Profile Selection Guide

| Profile | Use Case | Capabilities | Unknown Cap Handling |
|---------|----------|-------------|---------------------|
| **Safe** | High-security, production | read/write/http/events/session allowed; exec/env denied | Deny |
| **Standard** (default) | General use | Same defaults | Prompt user |
| **Permissive** | Development, internal testing | All allowed | Allow |

### Changing Profiles

Profiles are set via CLI flag, environment variable, or config file:

```bash
# CLI
pi --extension-policy safe

# Environment
export PI_EXTENSION_POLICY=safe

# Config (~/.config/pi/config.toml)
[extension_policy]
profile = "safe"
```

**Before changing profile in production:**
1. Review current FP/FN rates in rollout state
2. Switch to Shadow mode first: `PI_EXTENSION_RISK_ENFORCE=false`
3. Monitor for 24-48 hours under new profile
4. If metrics acceptable, re-enable enforcement

### Per-Extension Capability Overrides

Override default capabilities for specific extensions:

```toml
[extension_policy]
profile = "standard"

[extension_policy.per_extension."my-trusted-ext"]
allow = ["exec", "env"]

[extension_policy.per_extension."suspicious-ext"]
deny = ["http", "exec"]
```

Overrides are audited in the policy prompt cache and persisted to the
permission store.

---

## Enforcement Score Bands

Score bands map risk scores (0.0-1.0) to enforcement states:

| State | Safe | Balanced | Permissive |
|-------|------|----------|------------|
| Allow | < 0.30 | < 0.40 | < 0.55 |
| Harden | 0.30-0.49 | 0.40-0.59 | 0.55-0.69 |
| Prompt | 0.50-0.64 | 0.60-0.74 | 0.70-0.84 |
| Deny | 0.65-0.79 | 0.75-0.89 | 0.85-0.94 |
| Terminate | >= 0.80 | >= 0.90 | >= 0.95 |

### Hysteresis (Anti-Flapping)

De-escalation requires:
1. Score drops `de_escalation_margin` (default 0.10) below the entry threshold
2. At least `cooldown_calls` (default 3) consecutive evaluations in the lower band

This prevents rapid oscillation on borderline scores.

---

## Graduated Rollout Phases

### Phase Definitions

| Phase | Enforcement | Description |
|-------|------------|-------------|
| `shadow` | No | Risk scoring runs, telemetry recorded, no blocking |
| `log_only` | No | Decisions logged with would-be actions, calls proceed |
| `enforce_new` | Yes (new only) | Enforcement for extensions loaded after transition |
| `enforce_all` | Yes (all) | Full enforcement for all extensions |

### Phase Progression

```
shadow → log_only → enforce_new → enforce_all
```

Advance one phase at a time. Monitor at each phase before advancing:

- **Shadow → LogOnly:** Requires clean telemetry for >= 1 week
- **LogOnly → EnforceNew:** Requires FP rate < 5% for >= 2 weeks
- **EnforceNew → EnforceAll:** Requires no rollback triggers for >= 1 week

### Rollback Trigger Thresholds

| Trigger | Default | Description |
|---------|---------|-------------|
| `max_false_positive_rate` | 0.05 (5%) | FP rate over evaluation window |
| `max_error_rate` | 0.10 (10%) | Controller error rate over window |
| `max_latency_ms` | 200 | Average decision latency in window |
| `window_size` | 100 | Number of recent decisions to evaluate |

Rollback triggers only fire in enforcing phases (`enforce_new`, `enforce_all`).
When triggered, the system automatically reverts to `shadow` phase.

---

## Troubleshooting

### "Extension suddenly blocked after update"

1. Check if policy profile changed
2. Check if risk controller was enabled (`PI_EXTENSION_RISK_ENABLED`)
3. Review enforcement state for the extension in ledger
4. If FP: add per-extension override or tune alpha

### "Rollback keeps triggering"

1. Check `window_stats` to identify which threshold is breached
2. If FP rate: tune score bands or alpha
3. If error rate: investigate controller bugs, increase timeout
4. If latency: reduce window_size, check system load
5. Consider increasing `window_size` in rollback trigger (require more data)

### "Risk controller enabled but nothing happening"

1. Verify `enabled: true` AND extensions are loaded
2. Check `enforce` flag (may be in shadow mode)
3. Check rollout phase (may be in `shadow` or `log_only`)
4. Verify extensions are making hostcalls that go through risk path

### "Enforcement too slow to react"

1. Decrease `window_size` for faster detection (min: 8)
2. Decrease `cooldown_calls` in hysteresis (min: 1)
3. Decrease `de_escalation_margin` for faster recovery
4. Trade-off: more sensitive = more false positives
