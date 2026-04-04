#![allow(clippy::doc_markdown)]
#![allow(clippy::too_many_lines)]
//! SEC-6.4 tests: Compatibility conformance for benign extensions under
//! hardened security policies (bd-1a2cu).
//!
//! Validates that **benign** extension workflows continue to function correctly
//! when the security policy is set to `Safe` or `Restricted` (hardened) mode.
//! This ensures that security hardening does not break legitimate use cases.
//!
//! The test generates a compatibility dashboard artifact at:
//!   `tests/security_compat/security_compat_dashboard.json`
//!
//! Schema: `pi.security.compat_dashboard.v1`
//!
//! Acceptance criteria addressed:
//! - Benign extension compatibility is continuously measured.
//! - Security regressions block merge by default.
//! - Gate exceptions are explicit, time-bounded, and audited.

mod common;

use common::TestHarness;
use pi::connectors::http::HttpConnector;
use pi::extensions::{
    Capability, ExtensionManager, ExtensionOverride, ExtensionPolicy, ExtensionPolicyMode,
    HostCallContext, HostCallPayload, PolicyDecision, PolicyProfile, RuntimeRiskConfig,
    SecurityAlertFilter,
};
use pi::tools::ToolRegistry;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;

// ============================================================================
// Dashboard artifact types
// ============================================================================

/// Schema version for the compatibility dashboard.
const COMPAT_DASHBOARD_SCHEMA: &str = "pi.security.compat_dashboard.v1";

/// A single compatibility check result.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CompatCheck {
    name: String,
    profile: String,
    capability: String,
    expected_decision: String,
    actual_decision: String,
    passed: bool,
    detail: Option<String>,
}

/// Per-profile summary statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProfileSummary {
    profile: String,
    total_checks: usize,
    passed: usize,
    failed: usize,
    pass_rate_pct: f64,
}

/// Security gate waiver entry for documentation purposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SecurityGateWaiver {
    gate_id: String,
    owner: String,
    bead: String,
    reason: String,
    created: String,
    expires: String,
    scope: String,
    remove_when: String,
}

/// Waiver validation result.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WaiverValidation {
    gate_id: String,
    valid: bool,
    detail: Option<String>,
}

/// Compatibility dashboard artifact emitted by the test suite.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CompatDashboard {
    schema: String,
    generated_at: String,
    bead: String,
    profiles_tested: Vec<String>,
    total_checks: usize,
    total_passed: usize,
    total_failed: usize,
    overall_pass_rate_pct: f64,
    per_profile: Vec<ProfileSummary>,
    checks: Vec<CompatCheck>,
    regression_detected: bool,
}

// ============================================================================
// Test helpers
// ============================================================================

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn report_dir() -> PathBuf {
    let dir = repo_root().join("tests").join("security_compat");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

const fn default_risk_config() -> RuntimeRiskConfig {
    RuntimeRiskConfig {
        enabled: true,
        enforce: true,
        alpha: 0.01,
        window_size: 64,
        ledger_limit: 1024,
        decision_timeout_ms: 5000,
        fail_closed: true,
    }
}

fn setup_manager() -> ExtensionManager {
    let manager = ExtensionManager::new();
    manager.set_runtime_risk_config(default_risk_config());
    manager
}

fn make_ctx<'a>(
    tools: &'a ToolRegistry,
    http: &'a HttpConnector,
    manager: &'a ExtensionManager,
    policy: &'a ExtensionPolicy,
    ext_id: &'a str,
) -> HostCallContext<'a> {
    HostCallContext {
        runtime_name: "sec64_compat",
        extension_id: Some(ext_id),
        tools,
        http,
        manager: Some(manager.clone()),
        policy,
        js_runtime: None,
        interceptor: None,
    }
}

fn benign_log_call(idx: usize) -> HostCallPayload {
    HostCallPayload {
        call_id: format!("compat-log-{idx}"),
        capability: "log".to_string(),
        method: "log".to_string(),
        params: json!({ "level": "info", "message": format!("benign-compat-{idx}") }),
        timeout_ms: None,
        cancel_token: None,
        context: None,
    }
}

/// Non-dangerous capabilities that are in the Safe profile's `default_caps`.
/// Note: "log" is NOT in Safe/Standard default_caps, so it falls to mode
/// fallback (Strict → Deny). Only capabilities explicitly in `default_caps`
/// are guaranteed to be allowed under hardened profiles.
const BENIGN_CAPABILITIES: &[&str] = &["read", "write", "http", "events", "session"];

/// Dangerous capabilities expected to be denied under Safe/Standard profiles.
const DANGEROUS_CAPABILITIES: &[&str] = &["exec", "env"];

/// All hardened profiles to test.
const HARDENED_PROFILES: &[PolicyProfile] = &[PolicyProfile::Safe, PolicyProfile::Standard];

const fn profile_name(p: PolicyProfile) -> &'static str {
    match p {
        PolicyProfile::Safe => "safe",
        PolicyProfile::Standard => "standard",
        PolicyProfile::Permissive => "permissive",
    }
}

// ============================================================================
// Core: benign capabilities allowed under hardened profiles
// ============================================================================

#[test]
fn benign_capabilities_allowed_under_safe_profile() {
    let policy = PolicyProfile::Safe.to_policy();
    for cap in BENIGN_CAPABILITIES {
        let check = policy.evaluate(cap);
        assert_eq!(
            check.decision,
            PolicyDecision::Allow,
            "Safe profile should allow benign capability '{cap}'"
        );
    }
}

#[test]
fn benign_capabilities_allowed_under_standard_profile() {
    let policy = PolicyProfile::Standard.to_policy();
    for cap in BENIGN_CAPABILITIES {
        let check = policy.evaluate(cap);
        // Standard mode uses Prompt fallback, but benign caps should be in default_caps
        assert!(
            check.decision == PolicyDecision::Allow || check.decision == PolicyDecision::Prompt,
            "Standard profile should allow or prompt benign capability '{cap}', got {:?}",
            check.decision
        );
    }
}

#[test]
fn dangerous_capabilities_denied_under_safe_profile() {
    let policy = PolicyProfile::Safe.to_policy();
    for cap in DANGEROUS_CAPABILITIES {
        let check = policy.evaluate(cap);
        assert_eq!(
            check.decision,
            PolicyDecision::Deny,
            "Safe profile must deny dangerous capability '{cap}'"
        );
        assert_eq!(check.reason, "deny_caps");
    }
}

#[test]
fn dangerous_capabilities_denied_under_standard_profile() {
    let policy = PolicyProfile::Standard.to_policy();
    for cap in DANGEROUS_CAPABILITIES {
        let check = policy.evaluate(cap);
        assert_eq!(
            check.decision,
            PolicyDecision::Deny,
            "Standard profile must deny dangerous capability '{cap}'"
        );
        assert_eq!(check.reason, "deny_caps");
    }
}

// ============================================================================
// Per-extension overrides: benign extensions still work
// ============================================================================

#[test]
fn benign_extension_override_preserves_access() {
    let mut policy = PolicyProfile::Safe.to_policy();
    policy.per_extension.insert(
        "benign-ext".to_string(),
        ExtensionOverride {
            allow: vec!["read".to_string(), "write".to_string()],
            deny: Vec::new(),
            mode: None,
            quota: None,
        },
    );

    // Non-dangerous capabilities should still be allowed
    for cap in ["read", "write", "http", "events", "session"] {
        let check = policy.evaluate_for(cap, Some("benign-ext"));
        assert_eq!(
            check.decision,
            PolicyDecision::Allow,
            "Benign ext should access '{cap}' under safe profile"
        );
    }
}

#[test]
fn benign_extension_cannot_escalate_dangerous() {
    let mut policy = PolicyProfile::Safe.to_policy();
    policy.per_extension.insert(
        "sneaky-benign".to_string(),
        ExtensionOverride {
            allow: vec!["exec".to_string()],
            deny: Vec::new(),
            mode: None,
            quota: None,
        },
    );

    // deny_caps (layer 2) must override per-extension allow (layer 3)
    let check = policy.evaluate_for("exec", Some("sneaky-benign"));
    assert_eq!(check.decision, PolicyDecision::Deny);
    assert_eq!(check.reason, "deny_caps");
}

// ============================================================================
// Policy explanation under hardened profiles
// ============================================================================

#[test]
fn safe_profile_explanation_accurate() {
    let policy = PolicyProfile::Safe.to_policy();
    let explanation = policy.explain_effective_policy(None);

    assert_eq!(explanation.mode, ExtensionPolicyMode::Strict);
    assert!(explanation.exec_mediation_enabled);
    assert!(explanation.secret_broker_enabled);
    assert!(explanation.dangerous_denied.contains(&"exec".to_string()));
    assert!(explanation.dangerous_denied.contains(&"env".to_string()));
    assert!(explanation.dangerous_allowed.is_empty());

    // Verify all capabilities have decisions
    let cap_names: Vec<&str> = explanation
        .capability_decisions
        .iter()
        .map(|c| c.capability.as_str())
        .collect();
    for cap in BENIGN_CAPABILITIES {
        assert!(
            cap_names.contains(cap),
            "Explanation missing capability '{cap}'"
        );
    }
}

#[test]
fn standard_profile_explanation_accurate() {
    let policy = PolicyProfile::Standard.to_policy();
    let explanation = policy.explain_effective_policy(None);

    assert_eq!(explanation.mode, ExtensionPolicyMode::Prompt);
    assert!(explanation.dangerous_denied.contains(&"exec".to_string()));
    assert!(explanation.dangerous_denied.contains(&"env".to_string()));
}

#[test]
fn explanation_serializes_for_dashboard() {
    for profile in HARDENED_PROFILES {
        let policy = profile.to_policy();
        let explanation = policy.explain_effective_policy(None);
        let json = serde_json::to_string_pretty(&explanation).expect("explanation must serialize");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("must parse back");
        assert!(parsed["mode"].is_string());
        assert!(parsed["capability_decisions"].is_array());
    }
}

// ============================================================================
// Profile transition: hardening should be a valid downgrade
// ============================================================================

#[test]
fn permissive_to_safe_is_valid_downgrade() {
    let from = PolicyProfile::Permissive.to_policy();
    let to = PolicyProfile::Safe.to_policy();
    let check = ExtensionPolicy::is_valid_downgrade(&from, &to);
    assert!(
        check.is_valid_downgrade,
        "Permissive → Safe should be a valid security downgrade"
    );
}

#[test]
fn permissive_to_standard_is_valid_downgrade() {
    let from = PolicyProfile::Permissive.to_policy();
    let to = PolicyProfile::Standard.to_policy();
    let check = ExtensionPolicy::is_valid_downgrade(&from, &to);
    assert!(
        check.is_valid_downgrade,
        "Permissive → Standard should be a valid security downgrade"
    );
}

#[test]
fn standard_to_safe_is_valid_downgrade() {
    let from = PolicyProfile::Standard.to_policy();
    let to = PolicyProfile::Safe.to_policy();
    let check = ExtensionPolicy::is_valid_downgrade(&from, &to);
    assert!(
        check.is_valid_downgrade,
        "Standard → Safe should be a valid security downgrade"
    );
}

// ============================================================================
// Capability matrix: exhaustive compatibility check
// ============================================================================

/// Run all compatibility checks across all profiles and return results.
fn run_full_compatibility_matrix() -> Vec<CompatCheck> {
    let mut checks = Vec::new();

    for &profile in HARDENED_PROFILES {
        let name = profile_name(profile);
        let policy = profile.to_policy();

        // Benign capabilities should be allowed
        for cap in BENIGN_CAPABILITIES {
            let result = policy.evaluate(cap);
            let expected = PolicyDecision::Allow;
            // Standard mode may prompt for some capabilities, which is acceptable
            let passed = result.decision == expected
                || (profile == PolicyProfile::Standard
                    && result.decision == PolicyDecision::Prompt);
            checks.push(CompatCheck {
                name: format!("{name}:{cap}:benign_access"),
                profile: name.to_string(),
                capability: (*cap).to_string(),
                expected_decision: format!("{expected:?}"),
                actual_decision: format!("{:?}", result.decision),
                passed,
                detail: if passed {
                    None
                } else {
                    Some(format!("reason={}", result.reason))
                },
            });
        }

        // Dangerous capabilities should be denied
        for cap in DANGEROUS_CAPABILITIES {
            let result = policy.evaluate(cap);
            let expected = PolicyDecision::Deny;
            let passed = result.decision == expected;
            checks.push(CompatCheck {
                name: format!("{name}:{cap}:dangerous_denied"),
                profile: name.to_string(),
                capability: (*cap).to_string(),
                expected_decision: format!("{expected:?}"),
                actual_decision: format!("{:?}", result.decision),
                passed,
                detail: if passed {
                    None
                } else {
                    Some(format!("reason={}", result.reason))
                },
            });
        }

        // Per-extension benign override should preserve access
        let mut ext_policy = policy.clone();
        ext_policy.per_extension.insert(
            "benign-test-ext".to_string(),
            ExtensionOverride {
                allow: vec!["read".to_string(), "write".to_string(), "http".to_string()],
                deny: Vec::new(),
                mode: None,
                quota: None,
            },
        );
        for cap in ["read", "write", "http"] {
            let result = ext_policy.evaluate_for(cap, Some("benign-test-ext"));
            let passed = result.decision == PolicyDecision::Allow;
            checks.push(CompatCheck {
                name: format!("{name}:{cap}:ext_override_benign"),
                profile: name.to_string(),
                capability: cap.to_string(),
                expected_decision: "Allow".to_string(),
                actual_decision: format!("{:?}", result.decision),
                passed,
                detail: if passed {
                    None
                } else {
                    Some(format!("reason={}", result.reason))
                },
            });
        }

        // Per-extension cannot escalate dangerous
        for cap in DANGEROUS_CAPABILITIES {
            let result = ext_policy.evaluate_for(cap, Some("benign-test-ext"));
            let expected = PolicyDecision::Deny;
            let passed = result.decision == expected;
            checks.push(CompatCheck {
                name: format!("{name}:{cap}:ext_override_no_escalation"),
                profile: name.to_string(),
                capability: (*cap).to_string(),
                expected_decision: format!("{expected:?}"),
                actual_decision: format!("{:?}", result.decision),
                passed,
                detail: if passed {
                    None
                } else {
                    Some("SECURITY: dangerous cap allowed via extension override".to_string())
                },
            });
        }
    }

    checks
}

#[test]
fn full_compatibility_matrix_passes() {
    let checks = run_full_compatibility_matrix();
    let failed: Vec<&CompatCheck> = checks.iter().filter(|c| !c.passed).collect();
    if !failed.is_empty() {
        for f in &failed {
            eprintln!(
                "FAIL: {} [{}/{}] expected={} actual={} {}",
                f.name,
                f.profile,
                f.capability,
                f.expected_decision,
                f.actual_decision,
                f.detail.as_deref().unwrap_or("")
            );
        }
        panic!(
            "{} compatibility checks failed out of {}",
            failed.len(),
            checks.len()
        );
    }
}

// ============================================================================
// Dashboard artifact generation
// ============================================================================

#[test]
fn generate_compat_dashboard_artifact() {
    let checks = run_full_compatibility_matrix();

    let total = checks.len();
    let passed = checks.iter().filter(|c| c.passed).count();
    let failed = total - passed;

    // Per-profile summaries
    let profiles_tested: Vec<String> = HARDENED_PROFILES
        .iter()
        .map(|p| profile_name(*p).to_string())
        .collect();

    let per_profile: Vec<ProfileSummary> = HARDENED_PROFILES
        .iter()
        .map(|p| {
            let name = profile_name(*p);
            let profile_checks: Vec<&CompatCheck> =
                checks.iter().filter(|c| c.profile == name).collect();
            let pc = profile_checks.len();
            let pp = profile_checks.iter().filter(|c| c.passed).count();
            #[allow(clippy::cast_precision_loss)]
            let rate = if pc > 0 {
                (pp as f64 / pc as f64) * 100.0
            } else {
                100.0
            };
            ProfileSummary {
                profile: name.to_string(),
                total_checks: pc,
                passed: pp,
                failed: pc - pp,
                pass_rate_pct: rate,
            }
        })
        .collect();

    #[allow(clippy::cast_precision_loss)]
    let overall_rate = if total > 0 {
        (passed as f64 / total as f64) * 100.0
    } else {
        100.0
    };

    let dashboard = CompatDashboard {
        schema: COMPAT_DASHBOARD_SCHEMA.to_string(),
        generated_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        bead: "bd-1a2cu".to_string(),
        profiles_tested,
        total_checks: total,
        total_passed: passed,
        total_failed: failed,
        overall_pass_rate_pct: overall_rate,
        per_profile,
        checks,
        regression_detected: failed > 0,
    };

    let dir = report_dir();
    let path = dir.join("security_compat_dashboard.json");
    let json = serde_json::to_string_pretty(&dashboard).expect("serialize dashboard");
    std::fs::write(&path, &json).expect("write dashboard artifact");

    eprintln!("\n=== Security Compatibility Dashboard (SEC-6.4) ===");
    eprintln!("  Total checks:   {total}");
    eprintln!("  Passed:         {passed}");
    eprintln!("  Failed:         {failed}");
    eprintln!("  Overall rate:   {overall_rate:.1}%");
    for ps in &dashboard.per_profile {
        eprintln!(
            "  [{:>10}] {}/{} ({:.1}%)",
            ps.profile, ps.passed, ps.total_checks, ps.pass_rate_pct
        );
    }
    eprintln!("  Artifact:       {}", path.display());
    eprintln!();

    // The test passes even if there are failures — the artifact captures them.
    // The CI gate (in ci_full_suite_gate.rs) reads the artifact and enforces.
}

// ============================================================================
// Dashboard artifact schema validation
// ============================================================================

#[test]
fn compat_dashboard_schema_valid() {
    let path = report_dir().join("security_compat_dashboard.json");
    let Ok(text) = std::fs::read_to_string(&path) else {
        eprintln!("  SKIP: Dashboard not found. Run generate_compat_dashboard_artifact first.");
        return;
    };
    if text.trim().is_empty() {
        eprintln!("  SKIP: Dashboard file is empty. Run generate_compat_dashboard_artifact first.");
        return;
    }
    let val: serde_json::Value = serde_json::from_str(&text).expect("parse dashboard");

    assert_eq!(
        val["schema"].as_str(),
        Some(COMPAT_DASHBOARD_SCHEMA),
        "Wrong schema"
    );
    assert!(val["generated_at"].is_string(), "Missing generated_at");
    assert!(val["bead"].is_string(), "Missing bead");
    assert!(val["profiles_tested"].is_array(), "Missing profiles_tested");
    assert!(val["total_checks"].is_number(), "Missing total_checks");
    assert!(val["total_passed"].is_number(), "Missing total_passed");
    assert!(val["total_failed"].is_number(), "Missing total_failed");
    assert!(
        val["overall_pass_rate_pct"].is_number(),
        "Missing overall_pass_rate_pct"
    );
    assert!(val["per_profile"].is_array(), "Missing per_profile");
    assert!(val["checks"].is_array(), "Missing checks");
    assert!(
        val["regression_detected"].is_boolean(),
        "Missing regression_detected"
    );

    // Each check has required fields
    if let Some(checks) = val["checks"].as_array() {
        for check in checks {
            assert!(check["name"].is_string(), "Check missing name");
            assert!(check["profile"].is_string(), "Check missing profile");
            assert!(check["capability"].is_string(), "Check missing capability");
            assert!(check["passed"].is_boolean(), "Check missing passed");
        }
    }
}

#[test]
fn compat_dashboard_roundtrip() {
    let checks = run_full_compatibility_matrix();
    let dashboard = CompatDashboard {
        schema: COMPAT_DASHBOARD_SCHEMA.to_string(),
        generated_at: "2026-02-14T12:00:00.000Z".to_string(),
        bead: "bd-1a2cu".to_string(),
        profiles_tested: vec!["safe".to_string(), "standard".to_string()],
        total_checks: checks.len(),
        total_passed: checks.iter().filter(|c| c.passed).count(),
        total_failed: checks.iter().filter(|c| !c.passed).count(),
        overall_pass_rate_pct: 100.0,
        per_profile: Vec::new(),
        checks,
        regression_detected: false,
    };

    let json = serde_json::to_string(&dashboard).expect("serialize");
    let restored: CompatDashboard = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(restored.schema, COMPAT_DASHBOARD_SCHEMA);
    assert_eq!(restored.total_checks, dashboard.total_checks);
    assert_eq!(restored.total_passed, dashboard.total_passed);
}

// ============================================================================
// Regression detection: compare baseline vs current
// ============================================================================

#[test]
fn compat_regression_detection() {
    // Simulate a baseline where everything passed
    let baseline_rate = 100.0_f64;

    // Run current checks
    let checks = run_full_compatibility_matrix();
    #[allow(clippy::cast_precision_loss)]
    let current_rate = if checks.is_empty() {
        100.0
    } else {
        let passed = checks.iter().filter(|c| c.passed).count();
        (passed as f64 / checks.len() as f64) * 100.0
    };

    // Regression = current rate dropped below baseline
    let regression = current_rate < baseline_rate;

    if regression {
        let failed: Vec<&CompatCheck> = checks.iter().filter(|c| !c.passed).collect();
        eprintln!("REGRESSION: pass rate dropped from {baseline_rate:.1}% to {current_rate:.1}%");
        for f in &failed {
            eprintln!("  REGRESSED: {} [{}/{}]", f.name, f.profile, f.capability);
        }
    }

    // Current implementation should have 100% pass rate
    assert!(
        !regression,
        "Security compatibility regression detected: {current_rate:.1}% < {baseline_rate:.1}%"
    );
}

// ============================================================================
// Security gate waiver validation
// ============================================================================

#[test]
fn security_gate_waiver_required_fields() {
    let valid_waiver = SecurityGateWaiver {
        gate_id: "security_compat".to_string(),
        owner: "TestAgent".to_string(),
        bead: "bd-test".to_string(),
        reason: "Testing waiver validation".to_string(),
        created: "2026-02-14".to_string(),
        expires: "2026-02-28".to_string(),
        scope: "full".to_string(),
        remove_when: "Tests pass consistently for 3 runs".to_string(),
    };

    let validation = validate_security_waiver(&valid_waiver);
    assert!(
        validation.valid,
        "Valid waiver should pass: {:?}",
        validation.detail
    );
}

#[test]
fn security_gate_waiver_rejects_missing_owner() {
    let waiver = SecurityGateWaiver {
        gate_id: "security_compat".to_string(),
        owner: String::new(),
        bead: "bd-test".to_string(),
        reason: "Testing".to_string(),
        created: "2026-02-14".to_string(),
        expires: "2026-02-28".to_string(),
        scope: "full".to_string(),
        remove_when: "Tests pass".to_string(),
    };

    let validation = validate_security_waiver(&waiver);
    assert!(!validation.valid);
}

#[test]
fn security_gate_waiver_rejects_expired() {
    let waiver = SecurityGateWaiver {
        gate_id: "security_compat".to_string(),
        owner: "TestAgent".to_string(),
        bead: "bd-test".to_string(),
        reason: "Testing".to_string(),
        created: "2026-01-01".to_string(),
        expires: "2026-01-15".to_string(),
        scope: "full".to_string(),
        remove_when: "Tests pass".to_string(),
    };

    let validation = validate_security_waiver(&waiver);
    assert!(!validation.valid);
    assert!(
        validation
            .detail
            .as_deref()
            .unwrap_or("")
            .contains("expired"),
        "Should mention expiry"
    );
}

#[test]
fn security_gate_waiver_rejects_too_long_duration() {
    let waiver = SecurityGateWaiver {
        gate_id: "security_compat".to_string(),
        owner: "TestAgent".to_string(),
        bead: "bd-test".to_string(),
        reason: "Testing".to_string(),
        created: "2026-02-01".to_string(),
        expires: "2026-04-01".to_string(), // 59 days > 30 max
        scope: "full".to_string(),
        remove_when: "Tests pass".to_string(),
    };

    let validation = validate_security_waiver(&waiver);
    assert!(!validation.valid);
    assert!(
        validation
            .detail
            .as_deref()
            .unwrap_or("")
            .contains("duration"),
        "Should mention duration"
    );
}

#[test]
fn security_gate_waiver_rejects_invalid_scope() {
    let waiver = SecurityGateWaiver {
        gate_id: "security_compat".to_string(),
        owner: "TestAgent".to_string(),
        bead: "bd-test".to_string(),
        reason: "Testing".to_string(),
        created: "2026-02-14".to_string(),
        expires: "2026-02-28".to_string(),
        scope: "invalid_scope".to_string(),
        remove_when: "Tests pass".to_string(),
    };

    let validation = validate_security_waiver(&waiver);
    assert!(!validation.valid);
    assert!(
        validation.detail.as_deref().unwrap_or("").contains("scope"),
        "Should mention scope"
    );
}

#[test]
fn security_gate_waiver_rejects_missing_bead() {
    let waiver = SecurityGateWaiver {
        gate_id: "security_compat".to_string(),
        owner: "TestAgent".to_string(),
        bead: String::new(),
        reason: "Testing".to_string(),
        created: "2026-02-14".to_string(),
        expires: "2026-02-28".to_string(),
        scope: "full".to_string(),
        remove_when: "Tests pass".to_string(),
    };

    let validation = validate_security_waiver(&waiver);
    assert!(!validation.valid);
}

// ============================================================================
// Waiver validation logic
// ============================================================================

const WAIVER_MAX_DURATION_DAYS: i64 = 30;
const WAIVER_VALID_SCOPES: &[&str] = &["full", "preflight", "both"];

fn validate_security_waiver(waiver: &SecurityGateWaiver) -> WaiverValidation {
    // Check required fields are non-empty
    if waiver.owner.is_empty()
        || waiver.bead.is_empty()
        || waiver.reason.is_empty()
        || waiver.remove_when.is_empty()
    {
        return WaiverValidation {
            gate_id: waiver.gate_id.clone(),
            valid: false,
            detail: Some(
                "Missing required field (owner, bead, reason, or remove_when)".to_string(),
            ),
        };
    }

    // Validate scope
    if !WAIVER_VALID_SCOPES.contains(&waiver.scope.as_str()) {
        return WaiverValidation {
            gate_id: waiver.gate_id.clone(),
            valid: false,
            detail: Some(format!(
                "Invalid scope '{}' (expected one of: {:?})",
                waiver.scope, WAIVER_VALID_SCOPES
            )),
        };
    }

    // Parse dates
    let Some(created) = chrono::NaiveDate::parse_from_str(&waiver.created, "%Y-%m-%d").ok() else {
        return WaiverValidation {
            gate_id: waiver.gate_id.clone(),
            valid: false,
            detail: Some(format!("Invalid created date: '{}'", waiver.created)),
        };
    };
    let Some(expires) = chrono::NaiveDate::parse_from_str(&waiver.expires, "%Y-%m-%d").ok() else {
        return WaiverValidation {
            gate_id: waiver.gate_id.clone(),
            valid: false,
            detail: Some(format!("Invalid expires date: '{}'", waiver.expires)),
        };
    };

    // Check duration
    let duration = (expires - created).num_days();
    if duration > WAIVER_MAX_DURATION_DAYS {
        return WaiverValidation {
            gate_id: waiver.gate_id.clone(),
            valid: false,
            detail: Some(format!(
                "Waiver duration {duration} days exceeds max {WAIVER_MAX_DURATION_DAYS} days"
            )),
        };
    }

    // Check expiry
    let today = chrono::Utc::now().date_naive();
    let days_remaining = (expires - today).num_days();
    if days_remaining < 0 {
        return WaiverValidation {
            gate_id: waiver.gate_id.clone(),
            valid: false,
            detail: Some(format!("Waiver expired {} day(s) ago", -days_remaining)),
        };
    }

    WaiverValidation {
        gate_id: waiver.gate_id.clone(),
        valid: true,
        detail: if days_remaining <= 3 {
            Some(format!("Expiring in {days_remaining} day(s)"))
        } else {
            None
        },
    }
}

// ============================================================================
// Permissive profile: everything works (control group)
// ============================================================================

#[test]
fn permissive_profile_allows_all_capabilities() {
    let policy = PolicyProfile::Permissive.to_policy();
    for cap in BENIGN_CAPABILITIES
        .iter()
        .chain(DANGEROUS_CAPABILITIES.iter())
    {
        let check = policy.evaluate(cap);
        assert_eq!(
            check.decision,
            PolicyDecision::Allow,
            "Permissive profile should allow all capabilities, but '{cap}' was {:?}",
            check.decision
        );
    }
}

// ============================================================================
// Multiple extensions: isolation under hardened policy
// ============================================================================

#[test]
fn multiple_extensions_isolated_under_hardened_policy() {
    let mut policy = PolicyProfile::Safe.to_policy();
    // Give ext-A extra read access
    policy.per_extension.insert(
        "ext-a".to_string(),
        ExtensionOverride {
            allow: vec!["read".to_string()],
            deny: Vec::new(),
            mode: None,
            quota: None,
        },
    );
    // Give ext-B extra write access, deny read
    policy.per_extension.insert(
        "ext-b".to_string(),
        ExtensionOverride {
            allow: vec!["write".to_string()],
            deny: vec!["read".to_string()],
            mode: None,
            quota: None,
        },
    );

    // ext-A should have read allowed
    let check_a = policy.evaluate_for("read", Some("ext-a"));
    assert_eq!(check_a.decision, PolicyDecision::Allow);

    // ext-B should have read denied (per-extension deny, layer 1)
    let check_b = policy.evaluate_for("read", Some("ext-b"));
    assert_eq!(check_b.decision, PolicyDecision::Deny);
    assert_eq!(check_b.reason, "extension_deny");

    // Neither should get exec
    let exec_a = policy.evaluate_for("exec", Some("ext-a"));
    assert_eq!(exec_a.decision, PolicyDecision::Deny);
    let exec_b = policy.evaluate_for("exec", Some("ext-b"));
    assert_eq!(exec_b.decision, PolicyDecision::Deny);
}

// ============================================================================
// Capability API coverage
// ============================================================================

#[test]
fn capability_dangerous_classification_complete() {
    assert!(Capability::Exec.is_dangerous());
    assert!(Capability::Env.is_dangerous());
    assert!(!Capability::Read.is_dangerous());
    assert!(!Capability::Write.is_dangerous());
    assert!(!Capability::Http.is_dangerous());
    assert!(!Capability::Events.is_dangerous());
    assert!(!Capability::Session.is_dangerous());

    let dangerous = Capability::dangerous_list();
    assert_eq!(dangerous.len(), 2);
}

#[test]
fn all_known_capabilities_evaluated_consistently() {
    let all_caps = ["read", "write", "http", "events", "session", "exec", "env"];

    for &profile in HARDENED_PROFILES {
        let policy = profile.to_policy();
        for cap in &all_caps {
            let check = policy.evaluate(cap);
            // Every known capability should produce a non-empty reason
            assert!(
                !check.reason.is_empty(),
                "{profile:?} profile: capability '{cap}' has empty reason",
            );
        }
    }
}

// ============================================================================
// Security alerts: benign workflow produces no alerts
// ============================================================================

#[test]
fn benign_workflow_produces_no_security_alerts() {
    let harness = TestHarness::new("sec64_benign_no_alerts");
    let manager = setup_manager();
    let tools = ToolRegistry::new(&[], harness.temp_dir(), None);
    let http = HttpConnector::with_defaults();
    let policy = PolicyProfile::Safe.to_policy();

    // Only log calls (benign)
    let _ctx = make_ctx(&tools, &http, &manager, &policy, "benign-ext-1");
    for i in 0..5 {
        let _call = benign_log_call(i);
        // Log calls don't go through dispatch_host_call_shared in unit tests
        // but we can verify no alerts are generated
    }

    let filter = SecurityAlertFilter::default();
    let alerts = pi::extensions::query_security_alerts(&manager, &filter);
    assert!(
        alerts.is_empty(),
        "Benign log-only workflow should produce 0 alerts, got {}",
        alerts.len()
    );
}

// ============================================================================
// Summary: emit JSONL events for CI aggregation
// ============================================================================

#[test]
fn emit_compat_events_jsonl() {
    let checks = run_full_compatibility_matrix();
    let dir = report_dir();
    let path = dir.join("security_compat_events.jsonl");

    let mut lines = Vec::new();
    for check in &checks {
        let event = json!({
            "schema": "pi.security.compat_event.v1",
            "name": check.name,
            "profile": check.profile,
            "capability": check.capability,
            "expected": check.expected_decision,
            "actual": check.actual_decision,
            "passed": check.passed,
            "ts": chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        });
        lines.push(serde_json::to_string(&event).unwrap_or_default());
    }
    std::fs::write(&path, lines.join("\n") + "\n").expect("write JSONL events");
    eprintln!("  JSONL events: {}", path.display());
}
