use serde_json::Value;
use std::collections::HashSet;
use std::path::PathBuf;

const CONTRACT_PATH: &str = "docs/franken-node-unified-test-certification-contract.json";
const EXPECTED_SCHEMA: &str = "pi.frankennode.unified_test_certification_contract.v1";
const REQUIRED_DIMENSION_IDS: &[&str] = &[
    "unit_tests",
    "integration_tests",
    "e2e_tests",
    "conformance_tests",
    "security_tests",
    "performance_tests",
];
const REQUIRED_VERDICT_VALUES: &[&str] = &["CERTIFIED", "CONDITIONAL", "FAILED"];
const REQUIRED_VERDICT_FIELDS: &[&str] = &[
    "verdict",
    "timestamp_utc",
    "frankennode_version",
    "dimension_results",
    "total_tests_run",
    "total_tests_passed",
    "total_tests_failed",
    "waivers_applied",
];
const REQUIRED_FAILURE_OUTPUTS: &[&str] = &[
    "failing_test_name",
    "failure_message",
    "stack_trace_or_assertion_context",
    "reproduction_command",
    "related_contract_reference",
];
const REQUIRED_CONDITIONAL_OUTPUTS: &[&str] = &[
    "waiver_id",
    "waiver_reason",
    "waiver_expiry_date",
    "remediation_bead_id",
];
const REQUIRED_CI_RUNS_ON: &[&str] = &["pre_merge", "nightly", "release"];
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

fn validate_certification_dimensions(contract: &Value) -> ValidationResult<()> {
    let dimensions = as_array(contract, "/certification_dimensions");
    let dim_ids: HashSet<String> = dimensions
        .iter()
        .filter_map(|d| d["dimension_id"].as_str().map(String::from))
        .collect();
    for required in REQUIRED_DIMENSION_IDS {
        if !dim_ids.contains(*required) {
            return Err(format!("missing certification dimension: {required}"));
        }
    }

    for dim in dimensions {
        let id = dim["dimension_id"].as_str().map_or("<missing>", str::trim);

        if dim["description"].as_str().is_none_or(str::is_empty) {
            return Err(format!("dimension {id} missing description"));
        }
        if dim["pass_criteria"].as_str().is_none_or(str::is_empty) {
            return Err(format!("dimension {id} missing pass_criteria"));
        }
        if dim["coverage_minimum_percent"].as_u64().is_none()
            && dim["coverage_minimum_percent"].as_f64().is_none()
        {
            return Err(format!(
                "dimension {id} missing numeric coverage_minimum_percent"
            ));
        }
        if dim["required"].as_bool().is_none() {
            return Err(format!("dimension {id} missing boolean required field"));
        }
    }

    Ok(())
}

fn validate_certification_verdict(contract: &Value) -> ValidationResult<()> {
    validate_required_set(
        contract,
        "/certification_verdict/verdict_values",
        REQUIRED_VERDICT_VALUES,
        "verdict value",
    )?;
    validate_required_set(
        contract,
        "/certification_verdict/required_fields",
        REQUIRED_VERDICT_FIELDS,
        "verdict required field",
    )?;

    for rule in &[
        "certified_requires",
        "conditional_requires",
        "failed_requires",
    ] {
        if contract["certification_verdict"][rule]
            .as_str()
            .is_none_or(str::is_empty)
        {
            return Err(format!(
                "certification_verdict.{rule} must be a non-empty string"
            ));
        }
    }

    let artifact_path = contract["certification_verdict"]["artifact_path"]
        .as_str()
        .ok_or_else(|| "certification_verdict.artifact_path must be a string".to_string())?;
    if artifact_path.is_empty() {
        return Err("certification_verdict.artifact_path must not be empty".to_string());
    }

    Ok(())
}

fn validate_diagnostic_requirements(contract: &Value) -> ValidationResult<()> {
    validate_required_set(
        contract,
        "/diagnostic_requirements/on_failure/required_outputs",
        REQUIRED_FAILURE_OUTPUTS,
        "failure diagnostic output",
    )?;
    validate_required_set(
        contract,
        "/diagnostic_requirements/on_conditional/required_outputs",
        REQUIRED_CONDITIONAL_OUTPUTS,
        "conditional diagnostic output",
    )?;

    let invariants = as_array(contract, "/diagnostic_requirements/invariants");
    if invariants.is_empty() {
        return Err("diagnostic_requirements.invariants must not be empty".to_string());
    }

    Ok(())
}

fn validate_ci_integration(contract: &Value) -> ValidationResult<()> {
    let gate_id = contract["ci_integration"]["gate_id"]
        .as_str()
        .ok_or_else(|| "ci_integration.gate_id must be a string".to_string())?;
    if gate_id.is_empty() {
        return Err("ci_integration.gate_id must not be empty".to_string());
    }

    if contract["ci_integration"]["blocking"].as_bool() != Some(true) {
        return Err("ci_integration.blocking must be true".to_string());
    }

    validate_required_set(
        contract,
        "/ci_integration/runs_on",
        REQUIRED_CI_RUNS_ON,
        "CI run trigger",
    )?;

    let timeout = contract["ci_integration"]["timeout_minutes"]
        .as_u64()
        .ok_or_else(|| "ci_integration.timeout_minutes must be a positive integer".to_string())?;
    if timeout == 0 {
        return Err("ci_integration.timeout_minutes must be > 0".to_string());
    }

    let no_retry = as_array(contract, "/ci_integration/retry_policy/no_retry_on");
    let no_retry_set: HashSet<&str> = no_retry.iter().filter_map(Value::as_str).collect();
    if !no_retry_set.contains("test_failure") {
        return Err(
            "ci_integration.retry_policy.no_retry_on must include 'test_failure'".to_string(),
        );
    }

    Ok(())
}

fn validate_dependencies(contract: &Value) -> ValidationResult<()> {
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

fn remove_certification_dimension(contract: &mut Value, dimension_id: &str) -> bool {
    let dims = contract
        .pointer_mut("/certification_dimensions")
        .and_then(Value::as_array_mut)
        .expect("certification_dimensions must be mutable array");
    let before = dims.len();
    dims.retain(|d| d["dimension_id"].as_str() != Some(dimension_id));
    before != dims.len()
}

fn set_blocking_flag(contract: &mut Value, enabled: bool) {
    let field = contract
        .pointer_mut("/ci_integration/blocking")
        .expect("ci_integration.blocking must be mutable");
    *field = Value::Bool(enabled);
}

// --- Tests ---

#[test]
fn unified_test_certification_contract_exists_and_is_valid_json() {
    let path = repo_root().join(CONTRACT_PATH);
    assert!(
        path.is_file(),
        "missing unified test certification contract artifact: {}",
        path.display()
    );
    let _ = load_contract();
}

#[test]
fn unified_test_certification_contract_has_expected_schema_version_and_linkage() {
    let contract = load_contract();
    assert_eq!(
        contract["schema"],
        Value::String(EXPECTED_SCHEMA.to_string()),
        "unified test certification contract schema mismatch"
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
        Value::String("bd-3ar8v.7.11".to_string()),
        "bead linkage must target bd-3ar8v.7.11"
    );
}

#[test]
fn unified_test_certification_dimensions_are_complete() {
    let contract = load_contract();
    validate_certification_dimensions(&contract).unwrap_or_else(|err| panic!("{err}"));
}

#[test]
fn unified_test_certification_verdict_is_complete() {
    let contract = load_contract();
    validate_certification_verdict(&contract).unwrap_or_else(|err| panic!("{err}"));
}

#[test]
fn unified_test_certification_diagnostics_are_complete() {
    let contract = load_contract();
    validate_diagnostic_requirements(&contract).unwrap_or_else(|err| panic!("{err}"));
}

#[test]
fn unified_test_certification_ci_integration_is_blocking() {
    let contract = load_contract();
    validate_ci_integration(&contract).unwrap_or_else(|err| panic!("{err}"));
}

#[test]
fn unified_test_certification_dependencies_are_complete() {
    let contract = load_contract();
    validate_dependencies(&contract).unwrap_or_else(|err| panic!("{err}"));
}

#[test]
fn unified_test_certification_fails_closed_when_dimension_missing() {
    let mut contract = load_contract();
    assert!(
        remove_certification_dimension(&mut contract, "security_tests"),
        "mutation should remove required certification dimension"
    );

    let err = validate_certification_dimensions(&contract)
        .expect_err("contract should fail when required dimension is removed");
    assert!(
        err.contains("security_tests"),
        "expected error to reference missing dimension, got: {err}"
    );
}

#[test]
fn unified_test_certification_fails_closed_when_verdict_field_missing() {
    let mut contract = load_contract();
    assert!(
        remove_string_entry(
            &mut contract,
            "/certification_verdict/required_fields",
            "waivers_applied",
        ),
        "mutation should remove required verdict field"
    );

    let err = validate_certification_verdict(&contract)
        .expect_err("contract should fail when required verdict field is removed");
    assert!(
        err.contains("waivers_applied"),
        "expected error to reference missing verdict field, got: {err}"
    );
}

#[test]
fn unified_test_certification_fails_closed_when_failure_output_missing() {
    let mut contract = load_contract();
    assert!(
        remove_string_entry(
            &mut contract,
            "/diagnostic_requirements/on_failure/required_outputs",
            "reproduction_command",
        ),
        "mutation should remove required failure output"
    );

    let err = validate_diagnostic_requirements(&contract)
        .expect_err("contract should fail when required failure output is removed");
    assert!(
        err.contains("reproduction_command"),
        "expected error to reference missing failure output, got: {err}"
    );
}

#[test]
fn unified_test_certification_fails_closed_when_ci_not_blocking() {
    let mut contract = load_contract();
    set_blocking_flag(&mut contract, false);

    let err = validate_ci_integration(&contract)
        .expect_err("contract should fail when CI integration is not blocking");
    assert!(
        err.contains("blocking"),
        "expected error to reference blocking flag, got: {err}"
    );
}

#[test]
fn unified_test_certification_fails_closed_when_blocked_bead_missing() {
    let mut contract = load_contract();
    assert!(
        remove_string_entry(
            &mut contract,
            "/downstream_dependencies/blocked_beads",
            "bd-3ar8v.7.12",
        ),
        "mutation should remove required blocked bead"
    );

    let err = validate_dependencies(&contract)
        .expect_err("contract should fail when blocked bead linkage is missing");
    assert!(
        err.contains("bd-3ar8v.7.12"),
        "expected error to reference missing blocked bead, got: {err}"
    );
}
