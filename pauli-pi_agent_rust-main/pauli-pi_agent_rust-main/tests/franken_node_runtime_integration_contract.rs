use serde_json::Value;
use std::collections::HashSet;
use std::path::PathBuf;

const CONTRACT_PATH: &str = "docs/franken-node-runtime-integration-contract.json";
const EXPECTED_SCHEMA: &str = "pi.frankennode.runtime_integration_contract.v1";
const REQUIRED_CRATE_DEPS: &[&str] = &[
    "franken_node_kernel",
    "franken_node_event_loop",
    "franken_node_io",
];
const REQUIRED_RETIREMENT_TARGETS: &[&str] = &[
    "inline_event_loop_polling",
    "inline_io_driver",
    "inline_timer_wheel",
    "inline_dns_resolver",
];
const REQUIRED_VALIDATION_STAGES: &[&str] = &[
    "compilation_check",
    "unit_test_pass",
    "integration_test_pass",
    "benchmark_regression_check",
    "e2e_smoke_test",
];
const REQUIRED_LOG_FIELDS: &[&str] = &[
    "run_id",
    "correlation_id",
    "integration_stage",
    "crate_name",
    "validation_result",
    "retired_path_id",
    "feature_flag_state",
    "timestamp_utc",
];
const REQUIRED_EVENT_TYPES: &[&str] = &[
    "crate_dependency_resolved",
    "validation_stage_started",
    "validation_stage_completed",
    "path_retirement_started",
    "path_retirement_completed",
    "feature_flag_toggled",
];
const REQUIRED_BLOCKED_BEADS: &[&str] = &["bd-3ar8v.7.12"];

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

fn validate_integration_boundary(contract: &Value) -> ValidationResult<()> {
    validate_required_set(
        contract,
        "/integration_boundary/required_crate_dependencies",
        REQUIRED_CRATE_DEPS,
        "crate dependency",
    )?;

    if contract["integration_boundary"]["fail_closed_on_missing_crate"].as_bool() != Some(true) {
        return Err("integration_boundary.fail_closed_on_missing_crate must be true".to_string());
    }

    let boundary_trait = contract["integration_boundary"]["runtime_boundary_trait"]
        .as_str()
        .ok_or_else(|| "runtime_boundary_trait must be a string".to_string())?;
    if boundary_trait.is_empty() {
        return Err("runtime_boundary_trait must not be empty".to_string());
    }

    let invariants = as_array(contract, "/integration_boundary/invariants");
    if invariants.is_empty() {
        return Err("integration_boundary.invariants must not be empty".to_string());
    }

    Ok(())
}

fn validate_retirement_and_validation(contract: &Value) -> ValidationResult<()> {
    validate_required_set(
        contract,
        "/duplicate_path_retirement/required_retirement_targets",
        REQUIRED_RETIREMENT_TARGETS,
        "retirement target",
    )?;
    validate_required_set(
        contract,
        "/validation_gates/required_validation_stages",
        REQUIRED_VALIDATION_STAGES,
        "validation stage",
    )?;

    if contract["duplicate_path_retirement"]["rollback_supported"].as_bool() != Some(true) {
        return Err("duplicate_path_retirement.rollback_supported must be true".to_string());
    }

    if contract["validation_gates"]["fail_closed"].as_bool() != Some(true) {
        return Err("validation_gates.fail_closed must be true".to_string());
    }

    Ok(())
}

fn validate_feature_flag(contract: &Value) -> ValidationResult<()> {
    let flag_name = contract["feature_flag_contract"]["integration_feature_flag"]
        .as_str()
        .ok_or_else(|| "integration_feature_flag must be a string".to_string())?;
    if flag_name.is_empty() {
        return Err("integration_feature_flag must not be empty".to_string());
    }

    if contract["feature_flag_contract"]["default_enabled"].as_bool() != Some(false) {
        return Err(
            "feature_flag_contract.default_enabled must be false (opt-in integration)".to_string(),
        );
    }

    let graduation = contract["feature_flag_contract"]["graduation_criteria"]
        .as_str()
        .ok_or_else(|| "graduation_criteria must be a string".to_string())?;
    if graduation.is_empty() {
        return Err("graduation_criteria must not be empty".to_string());
    }

    let invariants = as_array(contract, "/feature_flag_contract/invariants");
    if invariants.is_empty() {
        return Err("feature_flag_contract.invariants must not be empty".to_string());
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

fn set_fail_closed_flag(contract: &mut Value, pointer: &str, enabled: bool) {
    let field = contract
        .pointer_mut(pointer)
        .unwrap_or_else(|| panic!("expected mutable field at {pointer}"));
    *field = Value::Bool(enabled);
}

// --- Tests ---

#[test]
fn runtime_integration_contract_exists_and_is_valid_json() {
    let path = repo_root().join(CONTRACT_PATH);
    assert!(
        path.is_file(),
        "missing runtime integration contract artifact: {}",
        path.display()
    );
    let _ = load_contract();
}

#[test]
fn runtime_integration_contract_has_expected_schema_version_and_linkage() {
    let contract = load_contract();
    assert_eq!(
        contract["schema"],
        Value::String(EXPECTED_SCHEMA.to_string()),
        "runtime integration contract schema mismatch"
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
        Value::String("bd-3ar8v.7.13".to_string()),
        "bead linkage must target bd-3ar8v.7.13"
    );
}

#[test]
fn runtime_integration_boundary_is_fail_closed() {
    let contract = load_contract();
    validate_integration_boundary(&contract).unwrap_or_else(|err| panic!("{err}"));
}

#[test]
fn runtime_integration_retirement_and_validation_are_complete() {
    let contract = load_contract();
    validate_retirement_and_validation(&contract).unwrap_or_else(|err| panic!("{err}"));
}

#[test]
fn runtime_integration_feature_flag_defaults_to_disabled() {
    let contract = load_contract();
    validate_feature_flag(&contract).unwrap_or_else(|err| panic!("{err}"));
}

#[test]
fn runtime_integration_logging_and_dependencies_are_complete() {
    let contract = load_contract();
    validate_logging_and_dependencies(&contract).unwrap_or_else(|err| panic!("{err}"));
}

#[test]
fn runtime_integration_fails_closed_when_crate_dependency_missing() {
    let mut contract = load_contract();
    assert!(
        remove_string_entry(
            &mut contract,
            "/integration_boundary/required_crate_dependencies",
            "franken_node_event_loop",
        ),
        "mutation should remove required crate dependency"
    );

    let err = validate_integration_boundary(&contract)
        .expect_err("contract should fail when required crate dependency is removed");
    assert!(
        err.contains("franken_node_event_loop"),
        "expected error to reference missing crate dependency, got: {err}"
    );
}

#[test]
fn runtime_integration_fails_closed_when_retirement_target_missing() {
    let mut contract = load_contract();
    assert!(
        remove_string_entry(
            &mut contract,
            "/duplicate_path_retirement/required_retirement_targets",
            "inline_timer_wheel",
        ),
        "mutation should remove required retirement target"
    );

    let err = validate_retirement_and_validation(&contract)
        .expect_err("contract should fail when required retirement target is removed");
    assert!(
        err.contains("inline_timer_wheel"),
        "expected error to reference missing retirement target, got: {err}"
    );
}

#[test]
fn runtime_integration_fails_closed_when_validation_not_fail_closed() {
    let mut contract = load_contract();
    set_fail_closed_flag(&mut contract, "/validation_gates/fail_closed", false);

    let err = validate_retirement_and_validation(&contract)
        .expect_err("contract should fail when validation gates are not fail-closed");
    assert!(
        err.contains("fail_closed"),
        "expected error to reference fail_closed, got: {err}"
    );
}

#[test]
fn runtime_integration_fails_closed_when_feature_flag_default_enabled() {
    let mut contract = load_contract();
    set_fail_closed_flag(
        &mut contract,
        "/feature_flag_contract/default_enabled",
        true,
    );

    let err = validate_feature_flag(&contract)
        .expect_err("contract should fail when feature flag defaults to enabled");
    assert!(
        err.contains("default_enabled"),
        "expected error to reference default_enabled, got: {err}"
    );
}

#[test]
fn runtime_integration_fails_closed_when_blocked_bead_missing() {
    let mut contract = load_contract();
    assert!(
        remove_string_entry(
            &mut contract,
            "/downstream_dependencies/blocked_beads",
            "bd-3ar8v.7.12",
        ),
        "mutation should remove required blocked bead"
    );

    let err = validate_logging_and_dependencies(&contract)
        .expect_err("contract should fail when blocked bead linkage is missing");
    assert!(
        err.contains("bd-3ar8v.7.12"),
        "expected error to reference missing blocked bead, got: {err}"
    );
}
