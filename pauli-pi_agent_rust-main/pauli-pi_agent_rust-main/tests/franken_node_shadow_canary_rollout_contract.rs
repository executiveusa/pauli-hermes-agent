use serde_json::Value;
use std::collections::HashSet;
use std::path::PathBuf;

const CONTRACT_PATH: &str = "docs/franken-node-shadow-canary-rollout-contract.json";
const EXPECTED_SCHEMA: &str = "pi.frankennode.shadow_canary_rollout_contract.v1";
const REQUIRED_STATES: &[&str] = &[
    "shadow_disabled",
    "shadow_observe",
    "canary_5",
    "canary_25",
    "canary_50",
    "canary_100",
    "rollback_engaged",
];
const REQUIRED_DIFF_DIMENSIONS: &[&str] = &[
    "semantic_output_hash",
    "latency_delta_ratio",
    "rss_delta_ratio",
    "error_rate_delta",
    "policy_violation_count",
];
const REQUIRED_AUTO_ACTIONS: &[&str] = &[
    "rollback_to_previous_stage",
    "demote_to_shadow_observe",
    "emit_replay_bundle",
];
const REQUIRED_ROLLBACK_TRIGGERS: &[&str] = &[
    "semantic_mismatch",
    "policy_violation",
    "latency_regression",
    "error_spike",
];
const REQUIRED_GUARD_ARTIFACTS: &[&str] = &[
    "docs/franken-node-security-sandbox-contract.json",
    "docs/franken-node-benchmark-resource-gate-contract.json",
    "docs/franken-node-claim-gating-contract.json",
];
const REQUIRED_LOG_FIELDS: &[&str] = &[
    "run_id",
    "correlation_id",
    "rollout_state",
    "candidate_state",
    "decision",
    "divergence_dimensions",
    "rollback_reason",
    "replay_artifact_path",
    "timestamp_utc",
];
const REQUIRED_EVENT_TYPES: &[&str] = &[
    "rollout_state_entered",
    "rollout_transition_evaluated",
    "divergence_detected",
    "rollback_triggered",
    "rollback_completed",
];
const REQUIRED_BLOCKED_BEADS: &[&str] = &["bd-3ar8v.7.11", "bd-3ar8v.7.12", "bd-3ar8v.7.13"];
const REQUIRED_SUPPORT_BEAD_IDS: &[&str] = &["bd-3ar8v.7.15.1"];

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn load_contract() -> Value {
    let path = repo_root().join(CONTRACT_PATH);
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    serde_json::from_str(&raw)
        .unwrap_or_else(|err| panic!("failed to parse {} as JSON: {err}", path.display()))
}

fn parse_semver(version: &str) -> Option<(u64, u64, u64)> {
    let mut parts = version.split('.');
    let major = parts.next()?.parse::<u64>().ok()?;
    let minor = parts.next()?.parse::<u64>().ok()?;
    let patch = parts.next()?.parse::<u64>().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some((major, minor, patch))
}

fn as_array<'a>(value: &'a Value, pointer: &str) -> &'a [Value] {
    value
        .pointer(pointer)
        .and_then(Value::as_array)
        .map_or_else(
            || panic!("expected JSON array at pointer {pointer}"),
            Vec::as_slice,
        )
}

fn non_empty_string_set(value: &Value, pointer: &str) -> HashSet<String> {
    let mut out = HashSet::new();
    for entry in as_array(value, pointer) {
        let raw = entry
            .as_str()
            .unwrap_or_else(|| panic!("expected string entry at {pointer}"));
        let normalized = raw.trim();
        assert!(
            !normalized.is_empty(),
            "entry at {pointer} must be non-empty"
        );
        out.insert(normalized.to_string());
    }
    out
}

type ValidationResult<T> = std::result::Result<T, String>;

fn validate_required_set(
    contract: &Value,
    pointer: &str,
    required_values: &[&str],
    label: &str,
) -> ValidationResult<()> {
    let values = non_empty_string_set(contract, pointer);
    for required in required_values {
        if !values.contains(*required) {
            return Err(format!("missing {label}: {required}"));
        }
    }
    Ok(())
}

fn validate_state_machine(contract: &Value) -> ValidationResult<()> {
    validate_required_set(
        contract,
        "/rollout_state_machine/states",
        REQUIRED_STATES,
        "rollout state",
    )?;

    if contract["rollout_state_machine"]["fail_closed_on_invalid_transition"] != Value::Bool(true) {
        return Err(
            "rollout_state_machine.fail_closed_on_invalid_transition must be true".to_string(),
        );
    }

    let states = non_empty_string_set(contract, "/rollout_state_machine/states");
    for transition in as_array(contract, "/rollout_state_machine/allowed_transitions") {
        let from = transition
            .get("from")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|entry| !entry.is_empty())
            .ok_or_else(|| "transition missing from state".to_string())?;
        let to = transition
            .get("to")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|entry| !entry.is_empty())
            .ok_or_else(|| "transition missing to state".to_string())?;
        let guard = transition
            .get("guard")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|entry| !entry.is_empty())
            .ok_or_else(|| "transition missing guard".to_string())?;

        if !states.contains(from) {
            return Err(format!("transition from unknown state: {from}"));
        }
        if !states.contains(to) {
            return Err(format!("transition to unknown state: {to}"));
        }
        if guard.is_empty() {
            return Err("transition guard must be non-empty".to_string());
        }
    }

    let stage_percentages: HashSet<u64> =
        as_array(contract, "/rollout_state_machine/stage_percentages")
            .iter()
            .map(|entry| {
                entry.as_u64().unwrap_or_else(|| {
                    panic!("stage_percentages entries must be positive integers")
                })
            })
            .collect();
    for required in [5_u64, 25, 50, 100] {
        if !stage_percentages.contains(&required) {
            return Err(format!("missing rollout stage percentage: {required}"));
        }
    }

    Ok(())
}

fn validate_diff_and_divergence_policy(contract: &Value) -> ValidationResult<()> {
    validate_required_set(
        contract,
        "/deterministic_diff_contract/required_dimensions",
        REQUIRED_DIFF_DIMENSIONS,
        "diff dimension",
    )?;
    validate_required_set(
        contract,
        "/divergence_policy/auto_actions",
        REQUIRED_AUTO_ACTIONS,
        "divergence auto action",
    )?;

    if contract["deterministic_diff_contract"]["fail_closed_on_missing_dimension"]
        != Value::Bool(true)
    {
        return Err(
            "deterministic_diff_contract.fail_closed_on_missing_dimension must be true".to_string(),
        );
    }
    if contract["divergence_policy"]["fail_closed"] != Value::Bool(true) {
        return Err("divergence_policy.fail_closed must be true".to_string());
    }

    let max_latency = contract["divergence_policy"]["max_latency_delta_ratio"]
        .as_f64()
        .ok_or_else(|| "max_latency_delta_ratio must be numeric".to_string())?;
    let max_rss = contract["divergence_policy"]["max_rss_delta_ratio"]
        .as_f64()
        .ok_or_else(|| "max_rss_delta_ratio must be numeric".to_string())?;
    let max_error = contract["divergence_policy"]["max_error_rate_delta"]
        .as_f64()
        .ok_or_else(|| "max_error_rate_delta must be numeric".to_string())?;

    if max_latency <= 0.0 || max_rss <= 0.0 || max_error < 0.0 {
        return Err("divergence thresholds must be positive and non-negative".to_string());
    }

    Ok(())
}

fn validate_rollback_and_policy_gates(contract: &Value) -> ValidationResult<()> {
    validate_required_set(
        contract,
        "/rollback_contract/required_triggers",
        REQUIRED_ROLLBACK_TRIGGERS,
        "rollback trigger",
    )?;
    validate_required_set(
        contract,
        "/policy_gate_contract/required_guard_artifacts",
        REQUIRED_GUARD_ARTIFACTS,
        "policy gate guard artifact",
    )?;

    if contract["rollback_contract"]["rollback_target_state"]
        != Value::String("shadow_observe".to_string())
    {
        return Err("rollback_contract.rollback_target_state must be shadow_observe".to_string());
    }
    if contract["policy_gate_contract"]["require_machine_gate_approval"] != Value::Bool(true) {
        return Err("policy_gate_contract.require_machine_gate_approval must be true".to_string());
    }
    if contract["policy_gate_contract"]["bypass_allowed"] != Value::Bool(false) {
        return Err("policy_gate_contract.bypass_allowed must be false".to_string());
    }

    Ok(())
}

fn validate_logging_and_dependencies(contract: &Value) -> ValidationResult<()> {
    validate_required_set(
        contract,
        "/structured_logging_contract/required_fields",
        REQUIRED_LOG_FIELDS,
        "structured logging required field",
    )?;
    validate_required_set(
        contract,
        "/structured_logging_contract/required_event_types",
        REQUIRED_EVENT_TYPES,
        "structured logging event type",
    )?;
    validate_required_set(
        contract,
        "/support_bead_ids",
        REQUIRED_SUPPORT_BEAD_IDS,
        "support_bead_ids entry",
    )?;
    validate_required_set(
        contract,
        "/downstream_dependencies/blocked_beads",
        REQUIRED_BLOCKED_BEADS,
        "blocked bead",
    )
}

fn remove_string_entry(contract: &mut Value, pointer: &str, value: &str) -> bool {
    let entries = contract
        .pointer_mut(pointer)
        .and_then(Value::as_array_mut)
        .unwrap_or_else(|| panic!("expected mutable array at pointer {pointer}"));
    let before = entries.len();
    entries.retain(|entry| entry.as_str().map(str::trim) != Some(value));
    before != entries.len()
}

fn set_divergence_fail_closed(contract: &mut Value, enabled: bool) {
    let field = contract
        .pointer_mut("/divergence_policy/fail_closed")
        .expect("divergence_policy.fail_closed must be mutable");
    *field = Value::Bool(enabled);
}

fn set_policy_bypass_allowed(contract: &mut Value, allowed: bool) {
    let field = contract
        .pointer_mut("/policy_gate_contract/bypass_allowed")
        .expect("policy_gate_contract.bypass_allowed must be mutable");
    *field = Value::Bool(allowed);
}

#[test]
fn shadow_canary_rollout_contract_exists_and_is_valid_json() {
    let path = repo_root().join(CONTRACT_PATH);
    assert!(
        path.is_file(),
        "missing shadow-canary rollout contract artifact: {}",
        path.display()
    );
    let _ = load_contract();
}

#[test]
fn shadow_canary_rollout_contract_has_expected_schema_version_and_linkage() {
    let contract = load_contract();
    assert_eq!(
        contract["schema"],
        Value::String(EXPECTED_SCHEMA.to_string()),
        "shadow-canary rollout contract schema mismatch"
    );

    let version = contract["contract_version"]
        .as_str()
        .expect("contract_version must be present");
    assert!(
        parse_semver(version).is_some(),
        "contract_version must be semantic version x.y.z, got: {version}"
    );

    assert_eq!(
        contract["bead_id"],
        Value::String("bd-3ar8v.7.15".to_string()),
        "bead linkage must target bd-3ar8v.7.15"
    );
    assert_eq!(
        contract["support_bead_id"],
        Value::String("bd-3ar8v.7.15.1".to_string()),
        "support bead linkage must target bd-3ar8v.7.15.1"
    );
}

#[test]
fn shadow_canary_rollout_state_machine_is_complete_and_valid() {
    let contract = load_contract();
    validate_state_machine(&contract).unwrap_or_else(|err| panic!("{err}"));
}

#[test]
fn shadow_canary_rollout_diff_and_divergence_policy_is_fail_closed() {
    let contract = load_contract();
    validate_diff_and_divergence_policy(&contract).unwrap_or_else(|err| panic!("{err}"));
}

#[test]
fn shadow_canary_rollout_rollback_and_policy_gate_contract_is_fail_closed() {
    let contract = load_contract();
    validate_rollback_and_policy_gates(&contract).unwrap_or_else(|err| panic!("{err}"));
}

#[test]
fn shadow_canary_rollout_logging_and_dependencies_are_complete() {
    let contract = load_contract();
    validate_logging_and_dependencies(&contract).unwrap_or_else(|err| panic!("{err}"));
}

#[test]
fn shadow_canary_rollout_contract_fails_closed_when_state_missing() {
    let mut contract = load_contract();
    assert!(
        remove_string_entry(&mut contract, "/rollout_state_machine/states", "canary_50",),
        "mutation should remove required rollout state"
    );

    let err = validate_state_machine(&contract)
        .expect_err("contract should fail when required rollout state is removed");
    assert!(
        err.contains("canary_50"),
        "expected error to reference missing state, got: {err}"
    );
}

#[test]
fn shadow_canary_rollout_contract_fails_closed_when_diff_dimension_missing() {
    let mut contract = load_contract();
    assert!(
        remove_string_entry(
            &mut contract,
            "/deterministic_diff_contract/required_dimensions",
            "semantic_output_hash",
        ),
        "mutation should remove required diff dimension"
    );

    let err = validate_diff_and_divergence_policy(&contract)
        .expect_err("contract should fail when required diff dimension is removed");
    assert!(
        err.contains("semantic_output_hash"),
        "expected error to reference missing diff dimension, got: {err}"
    );
}

#[test]
fn shadow_canary_rollout_contract_fails_closed_when_divergence_fail_closed_disabled() {
    let mut contract = load_contract();
    set_divergence_fail_closed(&mut contract, false);

    let err = validate_diff_and_divergence_policy(&contract)
        .expect_err("contract should fail when divergence fail_closed is disabled");
    assert!(
        err.contains("divergence_policy.fail_closed"),
        "expected error to reference divergence fail_closed, got: {err}"
    );
}

#[test]
fn shadow_canary_rollout_contract_fails_closed_when_rollback_trigger_missing() {
    let mut contract = load_contract();
    assert!(
        remove_string_entry(
            &mut contract,
            "/rollback_contract/required_triggers",
            "policy_violation",
        ),
        "mutation should remove required rollback trigger"
    );

    let err = validate_rollback_and_policy_gates(&contract)
        .expect_err("contract should fail when required rollback trigger is removed");
    assert!(
        err.contains("policy_violation"),
        "expected error to reference missing rollback trigger, got: {err}"
    );
}

#[test]
fn shadow_canary_rollout_contract_fails_closed_when_policy_bypass_allowed() {
    let mut contract = load_contract();
    set_policy_bypass_allowed(&mut contract, true);

    let err = validate_rollback_and_policy_gates(&contract)
        .expect_err("contract should fail when policy bypass is allowed");
    assert!(
        err.contains("bypass_allowed"),
        "expected error to reference policy bypass, got: {err}"
    );
}

#[test]
fn shadow_canary_rollout_contract_fails_closed_when_blocked_bead_missing() {
    let mut contract = load_contract();
    assert!(
        remove_string_entry(
            &mut contract,
            "/downstream_dependencies/blocked_beads",
            "bd-3ar8v.7.13",
        ),
        "mutation should remove required blocked bead"
    );

    let err = validate_logging_and_dependencies(&contract)
        .expect_err("contract should fail when blocked bead linkage is missing");
    assert!(
        err.contains("bd-3ar8v.7.13"),
        "expected error to reference missing blocked bead, got: {err}"
    );
}
