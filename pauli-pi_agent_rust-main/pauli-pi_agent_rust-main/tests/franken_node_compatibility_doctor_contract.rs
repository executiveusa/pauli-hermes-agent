use serde_json::Value;
use std::collections::HashSet;
use std::path::PathBuf;

const CONTRACT_PATH: &str = "docs/franken-node-compatibility-doctor-contract.json";
const EXPECTED_SCHEMA: &str = "pi.frankennode.compatibility_doctor_contract.v1";
const REQUIRED_CLASS_IDS: &[&str] = &[
    "PKG-unsupported-native-addon",
    "PKG-unresolved-peer-dependency",
    "RUNTIME-node-api-gap",
    "SEMANTIC-event-loop-ordering-drift",
    "TOOLING-lockfile-nondeterminism",
];
const REQUIRED_DIAGNOSTIC_FIELDS: &[&str] = &[
    "run_id",
    "correlation_id",
    "project_root",
    "rule_id",
    "class_id",
    "severity",
    "blocker_category",
    "readiness_score_delta",
    "suggested_fix_id",
    "suggested_fix_rationale",
    "evidence_refs",
];
const REQUIRED_ROOT_CAUSE_CLASSES: &[&str] = &["package", "runtime_api", "semantic", "tooling"];
const REQUIRED_SCORING_METRICS: &[&str] = &[
    "package_compatibility",
    "runtime_api_coverage",
    "semantic_parity",
    "tooling_determinism",
    "migration_effort",
];
const REQUIRED_SCORE_FIELDS: &[&str] = &[
    "project_score",
    "score_breakdown",
    "blocking_classes",
    "confidence",
];
const REQUIRED_REMEDIATION_FIELDS: &[&str] = &[
    "suggested_fix_id",
    "title",
    "rationale",
    "estimated_effort",
    "automation_level",
    "commands",
    "docs_links",
];
const REQUIRED_WORKFLOW_STAGES: &[&str] = &[
    "scan_project",
    "classify_incompatibilities",
    "score_readiness",
    "generate_remediation_plan",
    "export_diagnostic_bundle",
];
const REQUIRED_WORKFLOW_ARTIFACTS: &[&str] = &[
    "tests/e2e_results/<run>/franken_node_compatibility_doctor_report.json",
    "tests/e2e_results/<run>/franken_node_compatibility_diagnostics.jsonl",
    "tests/e2e_results/<run>/franken_node_migration_plan.json",
];
const REQUIRED_LOG_FIELDS: &[&str] = &[
    "run_id",
    "correlation_id",
    "event_type",
    "rule_id",
    "class_id",
    "blocker_category",
    "readiness_score_before",
    "readiness_score_after",
    "suggested_fix_id",
    "timestamp_utc",
];
const REQUIRED_EVENT_TYPES: &[&str] = &[
    "doctor.scan_started",
    "doctor.rule_hit",
    "doctor.score_updated",
    "doctor.remediation_emitted",
    "doctor.bundle_exported",
];
const REQUIRED_BLOCKED_BEADS: &[&str] = &[
    "bd-3ar8v.7.16",
    "bd-3ar8v.7.13",
    "bd-3ar8v.7.11",
    "bd-3ar8v.7.10",
];
const REQUIRED_SUPPORT_BEAD_IDS: &[&str] = &["bd-3ar8v.7.14.1"];

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

fn collect_taxonomy_class_ids(contract: &Value) -> ValidationResult<HashSet<String>> {
    let mut class_ids = HashSet::new();
    for row in as_array(contract, "/incompatibility_taxonomy") {
        let class_id = row
            .get("class_id")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|entry| !entry.is_empty())
            .ok_or_else(|| "incompatibility_taxonomy row missing class_id".to_string())?;
        if !class_ids.insert(class_id.to_string()) {
            return Err(format!("duplicate class_id detected: {class_id}"));
        }

        let category = row
            .get("category")
            .and_then(Value::as_str)
            .map(str::trim)
            .ok_or_else(|| format!("{class_id}: category must be present"))?;
        if !matches!(category, "package" | "runtime_api" | "semantic" | "tooling") {
            return Err(format!("{class_id}: invalid category value {category}"));
        }

        let severity = row
            .get("severity")
            .and_then(Value::as_str)
            .map(str::trim)
            .ok_or_else(|| format!("{class_id}: severity must be present"))?;
        if !matches!(severity, "high" | "medium" | "low") {
            return Err(format!("{class_id}: invalid severity value {severity}"));
        }

        let required_signals = row
            .get("required_signals")
            .and_then(Value::as_array)
            .ok_or_else(|| format!("{class_id}: required_signals must be an array"))?;
        if required_signals.is_empty() {
            return Err(format!("{class_id}: required_signals must not be empty"));
        }
    }

    Ok(class_ids)
}

fn validate_diagnostic_contract(contract: &Value) -> ValidationResult<()> {
    if contract["diagnostic_contract"]["fail_closed_on_missing_fields"] != Value::Bool(true) {
        return Err("diagnostic_contract.fail_closed_on_missing_fields must be true".to_string());
    }
    validate_required_set(
        contract,
        "/diagnostic_contract/required_fields",
        REQUIRED_DIAGNOSTIC_FIELDS,
        "diagnostic required_fields entry",
    )?;
    validate_required_set(
        contract,
        "/diagnostic_contract/required_root_cause_classes",
        REQUIRED_ROOT_CAUSE_CLASSES,
        "diagnostic root cause class",
    )?;

    if contract["diagnostic_contract"]["emit_json_schema"]
        != Value::String("pi.frankennode.compatibility_doctor_diagnostic.v1".to_string())
    {
        return Err("diagnostic_contract.emit_json_schema mismatch".to_string());
    }

    Ok(())
}

fn validate_scoring_contract(contract: &Value) -> ValidationResult<()> {
    if contract["readiness_scoring"]["fail_closed_on_weight_drift"] != Value::Bool(true) {
        return Err("readiness_scoring.fail_closed_on_weight_drift must be true".to_string());
    }

    let range = as_array(contract, "/readiness_scoring/range");
    if range.len() != 2 || range[0].as_i64() != Some(0) || range[1].as_i64() != Some(100) {
        return Err("readiness_scoring.range must be [0,100]".to_string());
    }

    let weights = as_array(contract, "/readiness_scoring/weights");
    if weights.is_empty() {
        return Err("readiness_scoring.weights must not be empty".to_string());
    }

    let mut metric_ids = HashSet::new();
    let mut weight_sum = 0.0_f64;
    for row in weights {
        let metric = row
            .get("metric")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|entry| !entry.is_empty())
            .ok_or_else(|| "readiness_scoring.weights row missing metric".to_string())?;

        if !metric_ids.insert(metric.to_string()) {
            return Err(format!("duplicate readiness metric detected: {metric}"));
        }

        let weight = row
            .get("weight")
            .and_then(Value::as_f64)
            .ok_or_else(|| format!("{metric}: weight must be numeric"))?;
        if !(0.0 < weight && weight <= 1.0) {
            return Err(format!("{metric}: weight must be in (0.0, 1.0]"));
        }
        weight_sum += weight;
    }

    if (weight_sum - 1.0).abs() > 1e-9 {
        return Err(format!(
            "readiness_scoring.weights must sum to 1.0, got {weight_sum}"
        ));
    }

    for required in REQUIRED_SCORING_METRICS {
        if !metric_ids.contains(*required) {
            return Err(format!("missing readiness metric: {required}"));
        }
    }

    validate_required_set(
        contract,
        "/readiness_scoring/score_fields",
        REQUIRED_SCORE_FIELDS,
        "readiness score field",
    )
}

fn validate_remediation_and_workflow_contract(contract: &Value) -> ValidationResult<()> {
    validate_required_set(
        contract,
        "/remediation_contract/required_fields",
        REQUIRED_REMEDIATION_FIELDS,
        "remediation required_fields entry",
    )?;

    if contract["remediation_contract"]["require_rule_hit_evidence"] != Value::Bool(true) {
        return Err("remediation_contract.require_rule_hit_evidence must be true".to_string());
    }
    if contract["remediation_contract"]["require_fix_rationale"] != Value::Bool(true) {
        return Err("remediation_contract.require_fix_rationale must be true".to_string());
    }
    if contract["remediation_contract"]["export_schema"]
        != Value::String("pi.frankennode.compatibility_remediation_plan.v1".to_string())
    {
        return Err("remediation_contract.export_schema mismatch".to_string());
    }

    validate_required_set(
        contract,
        "/workflow_contract/stages",
        REQUIRED_WORKFLOW_STAGES,
        "workflow stage",
    )?;
    validate_required_set(
        contract,
        "/workflow_contract/required_artifacts",
        REQUIRED_WORKFLOW_ARTIFACTS,
        "workflow required artifact",
    )?;

    if contract["workflow_contract"]["bundle_schema"]
        != Value::String("pi.frankennode.compatibility_doctor_bundle.v1".to_string())
    {
        return Err("workflow_contract.bundle_schema mismatch".to_string());
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

    if contract["structured_logging_contract"]["fail_closed_on_missing_correlation"]
        != Value::Bool(true)
    {
        return Err(
            "structured_logging_contract.fail_closed_on_missing_correlation must be true"
                .to_string(),
        );
    }

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

fn remove_taxonomy_class(contract: &mut Value, class_id: &str) -> bool {
    let rows = contract
        .pointer_mut("/incompatibility_taxonomy")
        .and_then(Value::as_array_mut)
        .expect("incompatibility_taxonomy must be mutable array");
    let before = rows.len();
    rows.retain(|row| row.get("class_id").and_then(Value::as_str).map(str::trim) != Some(class_id));
    before != rows.len()
}

fn set_scope_fail_closed(contract: &mut Value, enabled: bool) {
    let field = contract
        .pointer_mut("/doctor_scope/fail_closed_on_unknown_signals")
        .expect("doctor_scope.fail_closed_on_unknown_signals must be mutable");
    *field = Value::Bool(enabled);
}

fn set_metric_weight(contract: &mut Value, metric: &str, new_weight: f64) {
    let rows = contract
        .pointer_mut("/readiness_scoring/weights")
        .and_then(Value::as_array_mut)
        .expect("readiness_scoring.weights must be mutable array");
    let row = rows
        .iter_mut()
        .find(|row| row.get("metric").and_then(Value::as_str).map(str::trim) == Some(metric))
        .unwrap_or_else(|| panic!("missing readiness_scoring metric for mutation: {metric}"));
    row["weight"] = Value::from(new_weight);
}

#[test]
fn compatibility_doctor_contract_exists_and_is_valid_json() {
    let path = repo_root().join(CONTRACT_PATH);
    assert!(
        path.is_file(),
        "missing compatibility doctor contract artifact: {}",
        path.display()
    );
    let _ = load_contract();
}

#[test]
fn compatibility_doctor_contract_has_expected_schema_version_and_linkage() {
    let contract = load_contract();
    assert_eq!(
        contract["schema"],
        Value::String(EXPECTED_SCHEMA.to_string()),
        "compatibility doctor contract schema mismatch"
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
        Value::String("bd-3ar8v.7.14".to_string()),
        "bead linkage must target bd-3ar8v.7.14"
    );
    assert_eq!(
        contract["support_bead_id"],
        Value::String("bd-3ar8v.7.14.1".to_string()),
        "support bead linkage must target bd-3ar8v.7.14.1"
    );

    assert_eq!(
        contract["doctor_scope"]["fail_closed_on_unknown_signals"],
        Value::Bool(true),
        "doctor_scope.fail_closed_on_unknown_signals must be true"
    );
    assert_eq!(
        contract["doctor_scope"]["require_machine_readable_diagnostics"],
        Value::Bool(true),
        "doctor_scope.require_machine_readable_diagnostics must be true"
    );
}

#[test]
fn compatibility_doctor_taxonomy_is_complete_and_unique() {
    let contract = load_contract();
    let class_ids = collect_taxonomy_class_ids(&contract).unwrap_or_else(|err| panic!("{err}"));
    for required in REQUIRED_CLASS_IDS {
        assert!(
            class_ids.contains(*required),
            "missing required incompatibility class_id: {required}"
        );
    }
}

#[test]
fn compatibility_doctor_diagnostic_contract_is_fail_closed() {
    let contract = load_contract();
    validate_diagnostic_contract(&contract).unwrap_or_else(|err| panic!("{err}"));
}

#[test]
fn compatibility_doctor_scoring_contract_is_normalized() {
    let contract = load_contract();
    validate_scoring_contract(&contract).unwrap_or_else(|err| panic!("{err}"));
}

#[test]
fn compatibility_doctor_remediation_and_workflow_contracts_are_complete() {
    let contract = load_contract();
    validate_remediation_and_workflow_contract(&contract).unwrap_or_else(|err| panic!("{err}"));
}

#[test]
fn compatibility_doctor_logging_and_dependency_contracts_are_complete() {
    let contract = load_contract();
    validate_logging_and_dependencies(&contract).unwrap_or_else(|err| panic!("{err}"));
}

#[test]
fn compatibility_doctor_contract_fails_closed_when_required_class_missing() {
    let mut contract = load_contract();
    assert!(
        remove_taxonomy_class(&mut contract, "RUNTIME-node-api-gap"),
        "mutation should remove required class_id"
    );

    let err = collect_taxonomy_class_ids(&contract)
        .and_then(|class_ids| {
            for required in REQUIRED_CLASS_IDS {
                if !class_ids.contains(*required) {
                    return Err(format!(
                        "missing required incompatibility class_id: {required}"
                    ));
                }
            }
            Ok(())
        })
        .expect_err("contract should fail when a required class_id is removed");
    assert!(
        err.contains("RUNTIME-node-api-gap"),
        "expected error to reference removed class_id, got: {err}"
    );
}

#[test]
fn compatibility_doctor_contract_fails_closed_when_required_diagnostic_field_missing() {
    let mut contract = load_contract();
    assert!(
        remove_string_entry(
            &mut contract,
            "/diagnostic_contract/required_fields",
            "suggested_fix_rationale",
        ),
        "mutation should remove diagnostic required field"
    );

    let err = validate_diagnostic_contract(&contract)
        .expect_err("contract should fail when a required diagnostic field is removed");
    assert!(
        err.contains("suggested_fix_rationale"),
        "expected error to reference removed field, got: {err}"
    );
}

#[test]
fn compatibility_doctor_contract_fails_closed_when_scope_is_not_fail_closed() {
    let mut contract = load_contract();
    set_scope_fail_closed(&mut contract, false);

    let fail_closed = contract["doctor_scope"]["fail_closed_on_unknown_signals"]
        .as_bool()
        .expect("doctor_scope.fail_closed_on_unknown_signals must be boolean");
    assert!(
        !fail_closed,
        "contract should fail when fail_closed_on_unknown_signals is false"
    );
}

#[test]
fn compatibility_doctor_contract_fails_closed_when_weights_drift() {
    let mut contract = load_contract();
    set_metric_weight(&mut contract, "package_compatibility", 0.50);

    let err = validate_scoring_contract(&contract)
        .expect_err("contract should fail when readiness scoring weights drift");
    assert!(
        err.contains("must sum to 1.0"),
        "expected error to mention weight sum drift, got: {err}"
    );
}

#[test]
fn compatibility_doctor_contract_fails_closed_when_blocked_bead_linkage_missing() {
    let mut contract = load_contract();
    assert!(
        remove_string_entry(
            &mut contract,
            "/downstream_dependencies/blocked_beads",
            "bd-3ar8v.7.10",
        ),
        "mutation should remove required blocked bead"
    );

    let err = validate_logging_and_dependencies(&contract)
        .expect_err("contract should fail when blocked bead linkage is missing");
    assert!(
        err.contains("bd-3ar8v.7.10"),
        "expected error to reference missing blocked bead, got: {err}"
    );
}
