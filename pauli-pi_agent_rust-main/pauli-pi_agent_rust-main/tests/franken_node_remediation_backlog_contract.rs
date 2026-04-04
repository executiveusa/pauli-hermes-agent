use serde_json::Value;
use std::collections::HashSet;
use std::path::PathBuf;

const CONTRACT_PATH: &str = "docs/franken-node-remediation-backlog-contract.json";
const EXPECTED_SCHEMA: &str = "pi.frankennode.remediation_backlog_contract.v1";
const REQUIRED_TELEMETRY_SOURCES: &[&str] = &[
    "compatibility_doctor_reports",
    "shadow_canary_divergence_events",
    "benchmark_gate_failures",
    "user_filed_issues",
];
const REQUIRED_TELEMETRY_FIELDS: &[&str] = &[
    "source_id",
    "timestamp_utc",
    "package_name",
    "failure_category",
    "severity",
    "reproduction_context",
];
const REQUIRED_RANKING_DIMENSION_IDS: &[&str] = &[
    "user_impact",
    "severity",
    "fix_complexity",
    "ecosystem_centrality",
];
const REQUIRED_BACKLOG_FIELDS: &[&str] = &[
    "backlog_item_id",
    "package_name",
    "failure_category",
    "severity",
    "priority_score",
    "affected_project_count",
    "reproduction_command",
    "suggested_fix_type",
    "tracking_bead_id",
];
const REQUIRED_BLOCKED_BEADS: &[&str] = &["bd-3ar8v.7.12", "bd-3ar8v.7.13"];

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

fn validate_telemetry_ingestion(contract: &Value) -> ValidationResult<()> {
    validate_required_set(
        contract,
        "/telemetry_ingestion/supported_sources",
        REQUIRED_TELEMETRY_SOURCES,
        "telemetry source",
    )?;
    validate_required_set(
        contract,
        "/telemetry_ingestion/required_fields_per_event",
        REQUIRED_TELEMETRY_FIELDS,
        "telemetry field",
    )?;

    let invariants = as_array(contract, "/telemetry_ingestion/invariants");
    if invariants.is_empty() {
        return Err("telemetry_ingestion.invariants must not be empty".to_string());
    }

    Ok(())
}

fn validate_prioritization_model(contract: &Value) -> ValidationResult<()> {
    let dimensions = as_array(contract, "/prioritization_model/ranking_dimensions");

    let dim_ids: HashSet<String> = dimensions
        .iter()
        .filter_map(|d| d["dimension_id"].as_str().map(String::from))
        .collect();
    for required in REQUIRED_RANKING_DIMENSION_IDS {
        if !dim_ids.contains(*required) {
            return Err(format!("missing ranking dimension: {required}"));
        }
    }

    let weight_sum: f64 = dimensions.iter().filter_map(|d| d["weight"].as_f64()).sum();
    if (weight_sum - 1.0).abs() > 0.001 {
        return Err(format!(
            "ranking dimension weights must sum to 1.0, got: {weight_sum}"
        ));
    }

    for dim in dimensions {
        let id = dim["dimension_id"].as_str().map_or("<missing>", str::trim);
        let weight = dim["weight"]
            .as_f64()
            .ok_or_else(|| format!("dimension {id} missing numeric weight"))?;
        if weight <= 0.0 {
            return Err(format!("dimension {id} weight must be > 0, got: {weight}"));
        }
    }

    let invariants = as_array(contract, "/prioritization_model/invariants");
    if invariants.is_empty() {
        return Err("prioritization_model.invariants must not be empty".to_string());
    }

    Ok(())
}

fn validate_backlog_artifact(contract: &Value) -> ValidationResult<()> {
    validate_required_set(
        contract,
        "/backlog_artifact/required_fields_per_item",
        REQUIRED_BACKLOG_FIELDS,
        "backlog field",
    )?;

    let output_path = contract["backlog_artifact"]["output_path"]
        .as_str()
        .ok_or_else(|| "backlog_artifact.output_path must be a string".to_string())?;
    if output_path.is_empty() {
        return Err("backlog_artifact.output_path must not be empty".to_string());
    }

    let invariants = as_array(contract, "/backlog_artifact/invariants");
    if invariants.is_empty() {
        return Err("backlog_artifact.invariants must not be empty".to_string());
    }

    Ok(())
}

fn validate_automation(contract: &Value) -> ValidationResult<()> {
    let rules = as_array(contract, "/automation/auto_triage_rules");
    if rules.is_empty() {
        return Err("automation.auto_triage_rules must not be empty".to_string());
    }

    for (i, rule) in rules.iter().enumerate() {
        for field in &["rule_id", "condition", "action"] {
            if rule[field].as_str().is_none_or(str::is_empty) {
                return Err(format!(
                    "auto_triage_rules[{i}] missing or empty field: {field}"
                ));
            }
        }
    }

    let invariants = as_array(contract, "/automation/invariants");
    if invariants.is_empty() {
        return Err("automation.invariants must not be empty".to_string());
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

fn remove_ranking_dimension(contract: &mut Value, dimension_id: &str) -> bool {
    let dims = contract
        .pointer_mut("/prioritization_model/ranking_dimensions")
        .and_then(Value::as_array_mut)
        .expect("ranking_dimensions must be mutable array");
    let before = dims.len();
    dims.retain(|d| d["dimension_id"].as_str() != Some(dimension_id));
    before != dims.len()
}

// --- Tests ---

#[test]
fn remediation_backlog_contract_exists_and_is_valid_json() {
    let path = repo_root().join(CONTRACT_PATH);
    assert!(
        path.is_file(),
        "missing remediation backlog contract artifact: {}",
        path.display()
    );
    let _ = load_contract();
}

#[test]
fn remediation_backlog_contract_has_expected_schema_version_and_linkage() {
    let contract = load_contract();
    assert_eq!(
        contract["schema"],
        Value::String(EXPECTED_SCHEMA.to_string()),
        "remediation backlog contract schema mismatch"
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
        Value::String("bd-3ar8v.7.16".to_string()),
        "bead linkage must target bd-3ar8v.7.16"
    );
}

#[test]
fn remediation_backlog_telemetry_ingestion_is_complete() {
    let contract = load_contract();
    validate_telemetry_ingestion(&contract).unwrap_or_else(|err| panic!("{err}"));
}

#[test]
fn remediation_backlog_prioritization_model_is_sound() {
    let contract = load_contract();
    validate_prioritization_model(&contract).unwrap_or_else(|err| panic!("{err}"));
}

#[test]
fn remediation_backlog_artifact_requirements_are_complete() {
    let contract = load_contract();
    validate_backlog_artifact(&contract).unwrap_or_else(|err| panic!("{err}"));
}

#[test]
fn remediation_backlog_automation_rules_are_complete() {
    let contract = load_contract();
    validate_automation(&contract).unwrap_or_else(|err| panic!("{err}"));
}

#[test]
fn remediation_backlog_dependencies_are_complete() {
    let contract = load_contract();
    validate_dependencies(&contract).unwrap_or_else(|err| panic!("{err}"));
}

#[test]
fn remediation_backlog_fails_closed_when_telemetry_source_missing() {
    let mut contract = load_contract();
    assert!(
        remove_string_entry(
            &mut contract,
            "/telemetry_ingestion/supported_sources",
            "user_filed_issues",
        ),
        "mutation should remove required telemetry source"
    );

    let err = validate_telemetry_ingestion(&contract)
        .expect_err("contract should fail when required telemetry source is removed");
    assert!(
        err.contains("user_filed_issues"),
        "expected error to reference missing telemetry source, got: {err}"
    );
}

#[test]
fn remediation_backlog_fails_closed_when_ranking_dimension_missing() {
    let mut contract = load_contract();
    assert!(
        remove_ranking_dimension(&mut contract, "ecosystem_centrality"),
        "mutation should remove required ranking dimension"
    );

    let err = validate_prioritization_model(&contract)
        .expect_err("contract should fail when required ranking dimension is removed");
    assert!(
        err.contains("ecosystem_centrality"),
        "expected error to reference missing ranking dimension, got: {err}"
    );
}

#[test]
fn remediation_backlog_fails_closed_when_backlog_field_missing() {
    let mut contract = load_contract();
    assert!(
        remove_string_entry(
            &mut contract,
            "/backlog_artifact/required_fields_per_item",
            "reproduction_command",
        ),
        "mutation should remove required backlog field"
    );

    let err = validate_backlog_artifact(&contract)
        .expect_err("contract should fail when required backlog field is removed");
    assert!(
        err.contains("reproduction_command"),
        "expected error to reference missing backlog field, got: {err}"
    );
}

#[test]
fn remediation_backlog_fails_closed_when_blocked_bead_missing() {
    let mut contract = load_contract();
    assert!(
        remove_string_entry(
            &mut contract,
            "/downstream_dependencies/blocked_beads",
            "bd-3ar8v.7.13",
        ),
        "mutation should remove required blocked bead"
    );

    let err = validate_dependencies(&contract)
        .expect_err("contract should fail when blocked bead linkage is missing");
    assert!(
        err.contains("bd-3ar8v.7.13"),
        "expected error to reference missing blocked bead, got: {err}"
    );
}
