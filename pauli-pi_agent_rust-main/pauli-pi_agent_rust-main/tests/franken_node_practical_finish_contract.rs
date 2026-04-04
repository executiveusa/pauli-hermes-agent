use serde_json::Value;
use std::collections::HashSet;
use std::path::PathBuf;

const CONTRACT_PATH: &str = "docs/franken-node-practical-finish-contract.json";
const EXPECTED_SCHEMA: &str = "pi.frankennode.practical_finish_contract.v1";
const REQUIRED_GATE_CONTRACTS: &[&str] = &[
    "docs/franken-node-runtime-substrate-contract.json",
    "docs/franken-node-security-sandbox-contract.json",
    "docs/franken-node-benchmark-resource-gate-contract.json",
    "docs/franken-node-unified-test-certification-contract.json",
    "docs/franken-node-shadow-canary-rollout-contract.json",
    "docs/franken-node-runtime-integration-contract.json",
];
const REQUIRED_SIGNAL_FIELDS: &[&str] = &[
    "signal",
    "timestamp_utc",
    "frankennode_version",
    "gate_results",
    "open_beads_count",
    "waiver_count",
    "certification_verdict",
    "emitted_by",
];
const REQUIRED_SIGNAL_VALUES: &[&str] = &["PRACTICAL_FINISH", "NOT_READY"];
const REQUIRED_DEFERRED_DOCS: &[&str] = &[
    "architecture_overview_guide",
    "migration_quickstart",
    "api_reference_generation",
    "performance_tuning_guide",
    "troubleshooting_runbook",
];
const REQUIRED_HANDOFF_FIELDS: &[&str] = &[
    "doc_id",
    "priority",
    "estimated_effort",
    "owner",
    "tracking_bead_id",
];

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

fn validate_completion_criteria(contract: &Value) -> ValidationResult<()> {
    validate_required_set(
        contract,
        "/completion_criteria/required_gate_contracts",
        REQUIRED_GATE_CONTRACTS,
        "gate contract",
    )?;

    if contract["completion_criteria"]["all_gates_must_pass"].as_bool() != Some(true) {
        return Err("completion_criteria.all_gates_must_pass must be true".to_string());
    }

    let residual = contract["completion_criteria"]["residual_bead_threshold"]
        .as_u64()
        .ok_or_else(|| "residual_bead_threshold must be a non-negative integer".to_string())?;
    if residual != 0 {
        return Err(format!(
            "residual_bead_threshold must be 0, got: {residual}"
        ));
    }

    let invariants = as_array(contract, "/completion_criteria/invariants");
    if invariants.is_empty() {
        return Err("completion_criteria.invariants must not be empty".to_string());
    }

    Ok(())
}

fn validate_practical_finish_signal(contract: &Value) -> ValidationResult<()> {
    validate_required_set(
        contract,
        "/practical_finish_signal/required_fields",
        REQUIRED_SIGNAL_FIELDS,
        "signal required field",
    )?;
    validate_required_set(
        contract,
        "/practical_finish_signal/signal_values",
        REQUIRED_SIGNAL_VALUES,
        "signal value",
    )?;

    let artifact_path = contract["practical_finish_signal"]["signal_artifact_path"]
        .as_str()
        .ok_or_else(|| "signal_artifact_path must be a string".to_string())?;
    if artifact_path.is_empty() {
        return Err("signal_artifact_path must not be empty".to_string());
    }

    let invariants = as_array(contract, "/practical_finish_signal/invariants");
    if invariants.is_empty() {
        return Err("practical_finish_signal.invariants must not be empty".to_string());
    }

    Ok(())
}

fn validate_docs_last_handoff(contract: &Value) -> ValidationResult<()> {
    validate_required_set(
        contract,
        "/docs_last_handoff/deferred_documentation",
        REQUIRED_DEFERRED_DOCS,
        "deferred documentation item",
    )?;
    validate_required_set(
        contract,
        "/docs_last_handoff/handoff_required_fields",
        REQUIRED_HANDOFF_FIELDS,
        "handoff required field",
    )?;

    let invariants = as_array(contract, "/docs_last_handoff/invariants");
    if invariants.is_empty() {
        return Err("docs_last_handoff.invariants must not be empty".to_string());
    }

    Ok(())
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

fn set_all_gates_must_pass(contract: &mut Value, enabled: bool) {
    let field = contract
        .pointer_mut("/completion_criteria/all_gates_must_pass")
        .expect("completion_criteria.all_gates_must_pass must be mutable");
    *field = Value::Bool(enabled);
}

// --- Tests ---

#[test]
fn practical_finish_contract_exists_and_is_valid_json() {
    let path = repo_root().join(CONTRACT_PATH);
    assert!(
        path.is_file(),
        "missing practical-finish contract artifact: {}",
        path.display()
    );
    let _ = load_contract();
}

#[test]
fn practical_finish_contract_has_expected_schema_version_and_linkage() {
    let contract = load_contract();
    assert_eq!(
        contract["schema"],
        Value::String(EXPECTED_SCHEMA.to_string()),
        "practical-finish contract schema mismatch"
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
        Value::String("bd-3ar8v.7.12".to_string()),
        "bead linkage must target bd-3ar8v.7.12"
    );
}

#[test]
fn practical_finish_completion_criteria_require_all_gates() {
    let contract = load_contract();
    validate_completion_criteria(&contract).unwrap_or_else(|err| panic!("{err}"));
}

#[test]
fn practical_finish_signal_is_complete() {
    let contract = load_contract();
    validate_practical_finish_signal(&contract).unwrap_or_else(|err| panic!("{err}"));
}

#[test]
fn practical_finish_docs_last_handoff_is_complete() {
    let contract = load_contract();
    validate_docs_last_handoff(&contract).unwrap_or_else(|err| panic!("{err}"));
}

#[test]
fn practical_finish_fails_closed_when_gate_contract_missing() {
    let mut contract = load_contract();
    assert!(
        remove_string_entry(
            &mut contract,
            "/completion_criteria/required_gate_contracts",
            "docs/franken-node-security-sandbox-contract.json",
        ),
        "mutation should remove required gate contract"
    );

    let err = validate_completion_criteria(&contract)
        .expect_err("contract should fail when required gate contract is removed");
    assert!(
        err.contains("franken-node-security-sandbox-contract"),
        "expected error to reference missing gate contract, got: {err}"
    );
}

#[test]
fn practical_finish_fails_closed_when_all_gates_not_required() {
    let mut contract = load_contract();
    set_all_gates_must_pass(&mut contract, false);

    let err = validate_completion_criteria(&contract)
        .expect_err("contract should fail when all_gates_must_pass is false");
    assert!(
        err.contains("all_gates_must_pass"),
        "expected error to reference all_gates_must_pass, got: {err}"
    );
}

#[test]
fn practical_finish_fails_closed_when_signal_field_missing() {
    let mut contract = load_contract();
    assert!(
        remove_string_entry(
            &mut contract,
            "/practical_finish_signal/required_fields",
            "certification_verdict",
        ),
        "mutation should remove required signal field"
    );

    let err = validate_practical_finish_signal(&contract)
        .expect_err("contract should fail when required signal field is removed");
    assert!(
        err.contains("certification_verdict"),
        "expected error to reference missing signal field, got: {err}"
    );
}

#[test]
fn practical_finish_fails_closed_when_deferred_doc_missing() {
    let mut contract = load_contract();
    assert!(
        remove_string_entry(
            &mut contract,
            "/docs_last_handoff/deferred_documentation",
            "troubleshooting_runbook",
        ),
        "mutation should remove required deferred documentation item"
    );

    let err = validate_docs_last_handoff(&contract)
        .expect_err("contract should fail when required deferred doc is removed");
    assert!(
        err.contains("troubleshooting_runbook"),
        "expected error to reference missing deferred doc, got: {err}"
    );
}

#[test]
fn practical_finish_fails_closed_when_handoff_field_missing() {
    let mut contract = load_contract();
    assert!(
        remove_string_entry(
            &mut contract,
            "/docs_last_handoff/handoff_required_fields",
            "tracking_bead_id",
        ),
        "mutation should remove required handoff field"
    );

    let err = validate_docs_last_handoff(&contract)
        .expect_err("contract should fail when required handoff field is removed");
    assert!(
        err.contains("tracking_bead_id"),
        "expected error to reference missing handoff field, got: {err}"
    );
}
