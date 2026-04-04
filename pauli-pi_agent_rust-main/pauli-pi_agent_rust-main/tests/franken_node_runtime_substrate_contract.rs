use serde_json::Value;
use std::collections::HashSet;
use std::path::PathBuf;

const CONTRACT_PATH: &str = "docs/franken-node-runtime-substrate-contract.json";
const EXPECTED_SCHEMA: &str = "pi.frankennode.runtime_substrate_contract.v1";
const REQUIRED_WORKLOAD_IDS: &[&str] = &[
    "WL-extension-only-burst",
    "WL-runtime-mixed-long-session",
    "WL-io-heavy-hostcall-burst",
    "WL-policy-sensitive-capability-restricted",
];
const REQUIRED_LANE_IDS: &[&str] = &[
    "lane_fast_path_extension",
    "lane_balanced_runtime",
    "lane_conservative_serial",
    "lane_safe_degraded",
];
const REQUIRED_OPERATION_FIELDS: &[&str] = &[
    "run_id",
    "correlation_id",
    "workload_id",
    "operation_id",
    "lane_id",
    "policy_snapshot_id",
    "capability_profile",
    "timestamp_utc",
];
const REQUIRED_POLICY_FIELDS: &[&str] = &[
    "policy_snapshot_id",
    "capability_profile",
    "allowed_capabilities",
    "denied_capabilities",
    "decision_hash",
    "captured_at_utc",
];
const REQUIRED_LOGGING_FIELDS: &[&str] = &[
    "run_id",
    "workload_id",
    "lane_id",
    "fallback_trigger",
    "policy_decision",
    "queue_pressure",
    "reason",
    "timestamp_utc",
];
const REQUIRED_EVENT_TYPES: &[&str] = &[
    "lane_selected",
    "lane_degraded",
    "capability_denied",
    "fallback_engaged",
];
const REQUIRED_BLOCKED_BEADS: &[&str] = &[
    "bd-3ar8v.7.6",
    "bd-3ar8v.7.7",
    "bd-3ar8v.7.8",
    "bd-3ar8v.7.9",
];
const REQUIRED_SUPPORT_BEAD_IDS: &[&str] = &["bd-3ar8v.7.5.1", "bd-3ar8v.7.5.2", "bd-3ar8v.7.5.3"];

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

fn collect_lane_ids(contract: &Value) -> ValidationResult<HashSet<String>> {
    let mut lane_ids = HashSet::new();
    let lanes = as_array(contract, "/lane_contract");
    if lanes.is_empty() {
        return Err("lane_contract must not be empty".to_string());
    }
    for lane in lanes {
        let lane_id = lane
            .get("lane_id")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|entry| !entry.is_empty())
            .ok_or_else(|| "lane_contract entry missing non-empty lane_id".to_string())?;

        if !lane_ids.insert(lane_id.to_string()) {
            return Err(format!("duplicate lane_id detected: {lane_id}"));
        }
    }

    for lane in lanes {
        let lane_id = lane
            .get("lane_id")
            .and_then(Value::as_str)
            .map_or("<missing>", str::trim);
        let degrade_to = lane
            .get("degrade_to")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|entry| !entry.is_empty())
            .ok_or_else(|| format!("lane {lane_id} missing non-empty degrade_to"))?;
        if !lane_ids.contains(degrade_to) {
            return Err(format!(
                "lane {lane_id} has unknown degrade_to target: {degrade_to}"
            ));
        }
    }

    Ok(lane_ids)
}

fn validate_workload_lane_mappings(contract: &Value) -> ValidationResult<()> {
    let lane_ids = collect_lane_ids(contract)?;
    for required_lane in REQUIRED_LANE_IDS {
        if !lane_ids.contains(*required_lane) {
            return Err(format!("missing required lane_id: {required_lane}"));
        }
    }

    let mut workload_ids = HashSet::new();
    for workload in as_array(contract, "/workload_classes") {
        let workload_id = workload
            .get("workload_id")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|entry| !entry.is_empty())
            .ok_or_else(|| "workload_classes entry missing non-empty workload_id".to_string())?;
        if !workload_ids.insert(workload_id.to_string()) {
            return Err(format!("duplicate workload_id detected: {workload_id}"));
        }

        let allowed_lanes = workload
            .get("allowed_lanes")
            .and_then(Value::as_array)
            .ok_or_else(|| format!("{workload_id}: allowed_lanes must be an array"))?;
        if allowed_lanes.is_empty() {
            return Err(format!("{workload_id}: allowed_lanes must not be empty"));
        }

        let allowed_lane_set: HashSet<String> = allowed_lanes
            .iter()
            .map(|lane| {
                lane.as_str()
                    .map(str::trim)
                    .filter(|entry| !entry.is_empty())
                    .ok_or_else(|| {
                        format!("{workload_id}: allowed_lanes must be non-empty strings")
                    })
                    .map(ToOwned::to_owned)
            })
            .collect::<std::result::Result<HashSet<_>, _>>()?;

        for lane in &allowed_lane_set {
            if !lane_ids.contains(lane) {
                return Err(format!(
                    "{workload_id}: allowed_lanes includes unknown lane_id {lane}"
                ));
            }
        }

        let fallback_lane = workload
            .get("fallback_lane")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|entry| !entry.is_empty())
            .ok_or_else(|| format!("{workload_id}: fallback_lane must be non-empty"))?;

        if !allowed_lane_set.contains(fallback_lane) {
            return Err(format!(
                "{workload_id}: fallback_lane must be part of allowed_lanes ({fallback_lane})"
            ));
        }
        if !lane_ids.contains(fallback_lane) {
            return Err(format!(
                "{workload_id}: fallback_lane references unknown lane_id {fallback_lane}"
            ));
        }

        let required_envelope_fields = workload
            .get("required_envelope_fields")
            .and_then(Value::as_array)
            .ok_or_else(|| format!("{workload_id}: required_envelope_fields must be an array"))?;
        if required_envelope_fields.is_empty() {
            return Err(format!(
                "{workload_id}: required_envelope_fields must not be empty"
            ));
        }
    }

    for required_workload in REQUIRED_WORKLOAD_IDS {
        if !workload_ids.contains(*required_workload) {
            return Err(format!("missing required workload_id: {required_workload}"));
        }
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

fn remove_workload(contract: &mut Value, workload_id: &str) -> bool {
    let workloads = contract
        .pointer_mut("/workload_classes")
        .and_then(Value::as_array_mut)
        .expect("workload_classes must be mutable array");
    let before = workloads.len();
    workloads.retain(|row| {
        row.get("workload_id")
            .and_then(Value::as_str)
            .map(str::trim)
            != Some(workload_id)
    });
    before != workloads.len()
}

fn remove_lane(contract: &mut Value, lane_id: &str) -> bool {
    let lanes = contract
        .pointer_mut("/lane_contract")
        .and_then(Value::as_array_mut)
        .expect("lane_contract must be mutable array");
    let before = lanes.len();
    lanes.retain(|row| row.get("lane_id").and_then(Value::as_str).map(str::trim) != Some(lane_id));
    before != lanes.len()
}

fn set_workload_fallback(contract: &mut Value, workload_id: &str, fallback_lane: &str) {
    let workloads = contract
        .pointer_mut("/workload_classes")
        .and_then(Value::as_array_mut)
        .expect("workload_classes must be mutable array");
    let workload = workloads
        .iter_mut()
        .find(|row| {
            row.get("workload_id")
                .and_then(Value::as_str)
                .map(str::trim)
                == Some(workload_id)
        })
        .unwrap_or_else(|| panic!("missing workload_id for mutation: {workload_id}"));
    workload["fallback_lane"] = Value::String(fallback_lane.to_string());
}

#[test]
fn runtime_substrate_contract_exists_and_is_valid_json() {
    let path = repo_root().join(CONTRACT_PATH);
    assert!(
        path.is_file(),
        "missing runtime substrate contract artifact: {}",
        path.display()
    );
    let _ = load_contract();
}

#[test]
fn runtime_substrate_contract_has_expected_schema_version_and_linkage() {
    let contract = load_contract();

    assert_eq!(
        contract["schema"],
        Value::String(EXPECTED_SCHEMA.to_string()),
        "runtime substrate contract schema mismatch"
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
        Value::String("bd-3ar8v.7.5".to_string()),
        "bead linkage must target bd-3ar8v.7.5"
    );
    assert_eq!(
        contract["support_bead_id"],
        Value::String("bd-3ar8v.7.5.3".to_string()),
        "support bead linkage must target bd-3ar8v.7.5.3"
    );
    validate_required_set(
        &contract,
        "/support_bead_ids",
        REQUIRED_SUPPORT_BEAD_IDS,
        "support bead linkage",
    )
    .unwrap_or_else(|err| panic!("{err}"));
}

#[test]
fn runtime_substrate_contract_declares_workloads_and_lane_mappings() {
    let contract = load_contract();
    validate_workload_lane_mappings(&contract).unwrap_or_else(|err| panic!("{err}"));
}

#[test]
fn runtime_substrate_contract_declares_fail_closed_policy_and_envelope_fields() {
    let contract = load_contract();

    let routing = &contract["routing_policy"];
    assert_eq!(
        routing["fail_closed"].as_bool(),
        Some(true),
        "routing_policy.fail_closed must be true"
    );
    assert_eq!(
        routing["require_explicit_lane_for_workload"].as_bool(),
        Some(true),
        "routing_policy.require_explicit_lane_for_workload must be true"
    );
    assert_eq!(
        routing["require_fallback_lane"].as_bool(),
        Some(true),
        "routing_policy.require_fallback_lane must be true"
    );
    assert_eq!(
        routing["disallow_implicit_capability_escalation"].as_bool(),
        Some(true),
        "routing_policy.disallow_implicit_capability_escalation must be true"
    );

    validate_required_set(
        &contract,
        "/operation_envelope_contract/required_fields",
        REQUIRED_OPERATION_FIELDS,
        "operation envelope field",
    )
    .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(
        contract["operation_envelope_contract"]["failure_policy"].as_str(),
        Some("hard_fail"),
        "operation_envelope_contract.failure_policy must be hard_fail"
    );

    validate_required_set(
        &contract,
        "/policy_snapshot_contract/required_fields",
        REQUIRED_POLICY_FIELDS,
        "policy snapshot field",
    )
    .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(
        contract["policy_snapshot_contract"]["failure_policy"].as_str(),
        Some("hard_fail"),
        "policy_snapshot_contract.failure_policy must be hard_fail"
    );
}

#[test]
fn runtime_substrate_contract_declares_logging_and_downstream_blockers() {
    let contract = load_contract();

    validate_required_set(
        &contract,
        "/structured_logging_contract/required_fields",
        REQUIRED_LOGGING_FIELDS,
        "structured logging field",
    )
    .unwrap_or_else(|err| panic!("{err}"));
    validate_required_set(
        &contract,
        "/structured_logging_contract/required_event_types",
        REQUIRED_EVENT_TYPES,
        "structured logging event type",
    )
    .unwrap_or_else(|err| panic!("{err}"));

    validate_required_set(
        &contract,
        "/downstream_dependencies/blocked_beads",
        REQUIRED_BLOCKED_BEADS,
        "blocked downstream bead",
    )
    .unwrap_or_else(|err| panic!("{err}"));

    let release_blockers = as_array(&contract, "/release_blockers");
    assert!(
        release_blockers.len() >= 4,
        "release_blockers must declare multiple fail-closed blockers"
    );
}

#[test]
fn runtime_substrate_contract_missing_workload_mutation_fails_closed() {
    let mut contract = load_contract();
    assert!(
        remove_workload(&mut contract, "WL-runtime-mixed-long-session"),
        "mutation setup should remove required workload"
    );

    let err = validate_workload_lane_mappings(&contract)
        .expect_err("missing required workload mapping must fail validation");
    assert!(
        err.contains("WL-runtime-mixed-long-session"),
        "error should reference missing workload id, got: {err}"
    );
}

#[test]
fn runtime_substrate_contract_missing_lane_mutation_fails_closed() {
    let mut contract = load_contract();
    assert!(
        remove_lane(&mut contract, "lane_balanced_runtime"),
        "mutation setup should remove required lane"
    );

    let err = validate_workload_lane_mappings(&contract)
        .expect_err("missing required lane must fail validation");
    assert!(
        err.contains("lane_balanced_runtime"),
        "error should reference missing required lane id, got: {err}"
    );
}

#[test]
fn runtime_substrate_contract_fallback_lane_drift_mutation_fails_closed() {
    let mut contract = load_contract();
    set_workload_fallback(
        &mut contract,
        "WL-runtime-mixed-long-session",
        "lane_fast_path_extension",
    );

    let err = validate_workload_lane_mappings(&contract)
        .expect_err("fallback lane drift must fail validation");
    assert!(
        err.contains("fallback_lane"),
        "error should reference fallback lane mismatch, got: {err}"
    );
}

#[test]
fn runtime_substrate_contract_missing_logging_field_mutation_fails_closed() {
    let mut contract = load_contract();
    assert!(
        remove_string_entry(
            &mut contract,
            "/structured_logging_contract/required_fields",
            "queue_pressure",
        ),
        "mutation setup should remove queue_pressure logging field"
    );

    let err = validate_required_set(
        &contract,
        "/structured_logging_contract/required_fields",
        REQUIRED_LOGGING_FIELDS,
        "structured logging field",
    )
    .expect_err("missing required logging field must fail validation");
    assert!(
        err.contains("queue_pressure"),
        "error should reference removed logging field, got: {err}"
    );
}

#[test]
fn runtime_substrate_contract_missing_policy_field_mutation_fails_closed() {
    let mut contract = load_contract();
    assert!(
        remove_string_entry(
            &mut contract,
            "/policy_snapshot_contract/required_fields",
            "decision_hash",
        ),
        "mutation setup should remove decision_hash policy field"
    );

    let err = validate_required_set(
        &contract,
        "/policy_snapshot_contract/required_fields",
        REQUIRED_POLICY_FIELDS,
        "policy snapshot field",
    )
    .expect_err("missing required policy field must fail validation");
    assert!(
        err.contains("decision_hash"),
        "error should reference removed policy field, got: {err}"
    );
}
