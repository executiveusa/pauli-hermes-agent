use serde_json::Value;
use std::collections::HashSet;
use std::path::PathBuf;

const CONTRACT_PATH: &str = "docs/franken-node-benchmark-resource-gate-contract.json";
const EXPECTED_SCHEMA: &str = "pi.frankennode.benchmark_resource_gate_contract.v1";
const REQUIRED_WORKLOAD_CLASSES: &[&str] = &[
    "macro_long_session_resume",
    "macro_extension_heavy_mixed",
    "macro_io_burst_pipeline",
    "macro_policy_restricted_path",
];
const REQUIRED_COMPARISON_TARGETS: &[&str] = &["franken_node", "nodejs", "bun"];
const REQUIRED_METRICS: &[&str] = &[
    "latency_p50_ms",
    "latency_p95_ms",
    "throughput_ops_per_sec",
    "rss_peak_mb",
    "cpu_time_ms",
];
const REQUIRED_RELATIVE_METRICS: &[&str] = &[
    "speedup_vs_node",
    "speedup_vs_bun",
    "rss_delta_vs_node",
    "rss_delta_vs_bun",
];
const REQUIRED_VARIANCE_FIELDS: &[&str] = &[
    "sample_count",
    "variance",
    "stddev",
    "confidence_interval",
    "confidence_level",
    "bootstrap_iterations",
];
const REQUIRED_GATE_CHECKS: &[&str] = &[
    "macro_workload_coverage_complete",
    "comparison_targets_present",
    "confidence_threshold_met",
    "microbench_only_claim_blocked",
    "resource_envelope_within_budget",
];
const REQUIRED_REJECTION_CHECKS: &[&str] = &[
    "claim_integrity.microbench_only_claim",
    "claim_integrity.global_claim_missing_partition_coverage",
];
const REQUIRED_LOG_FIELDS: &[&str] = &[
    "run_id",
    "correlation_id",
    "workload_class",
    "comparison_target",
    "stage",
    "metric",
    "value",
    "confidence_interval",
    "regression_root_cause_hint",
    "timestamp_utc",
];
const REQUIRED_EVENT_TYPES: &[&str] = &[
    "benchmark_stage_started",
    "benchmark_stage_completed",
    "metric_aggregated",
    "gate_check_evaluated",
    "gate_failed",
];
const REQUIRED_BLOCKED_BEADS: &[&str] = &["bd-3ar8v.7.15", "bd-3ar8v.7.16"];
const REQUIRED_SUPPORT_BEAD_IDS: &[&str] = &["bd-3ar8v.7.10.1"];

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

fn validate_workload_protocol(contract: &Value) -> ValidationResult<()> {
    validate_required_set(
        contract,
        "/workload_protocol/required_workload_classes",
        REQUIRED_WORKLOAD_CLASSES,
        "workload class",
    )?;
    validate_required_set(
        contract,
        "/workload_protocol/required_comparison_targets",
        REQUIRED_COMPARISON_TARGETS,
        "comparison target",
    )?;

    if contract["workload_protocol"]["require_absolute_and_relative_metrics"] != Value::Bool(true) {
        return Err(
            "workload_protocol.require_absolute_and_relative_metrics must be true".to_string(),
        );
    }
    if contract["workload_protocol"]["forbid_microbench_only_claims"] != Value::Bool(true) {
        return Err("workload_protocol.forbid_microbench_only_claims must be true".to_string());
    }

    let minimum_workloads = contract["workload_protocol"]["minimum_macro_workloads_per_run"]
        .as_u64()
        .ok_or_else(|| "minimum_macro_workloads_per_run must be positive integer".to_string())?;
    if minimum_workloads < REQUIRED_WORKLOAD_CLASSES.len() as u64 {
        return Err(
            "minimum_macro_workloads_per_run must cover all required workload classes".to_string(),
        );
    }

    Ok(())
}

fn validate_metric_and_confidence_contract(contract: &Value) -> ValidationResult<()> {
    validate_required_set(
        contract,
        "/metric_aggregator_contract/required_metrics",
        REQUIRED_METRICS,
        "metric",
    )?;
    validate_required_set(
        contract,
        "/metric_aggregator_contract/required_relative_metrics",
        REQUIRED_RELATIVE_METRICS,
        "relative metric",
    )?;

    if contract["metric_aggregator_contract"]["fail_closed_on_missing_metric"] != Value::Bool(true)
    {
        return Err(
            "metric_aggregator_contract.fail_closed_on_missing_metric must be true".to_string(),
        );
    }

    validate_required_set(
        contract,
        "/variance_confidence_contract/required_fields",
        REQUIRED_VARIANCE_FIELDS,
        "variance/confidence field",
    )?;

    let min_sample_count = contract["variance_confidence_contract"]["minimum_sample_count"]
        .as_u64()
        .ok_or_else(|| "minimum_sample_count must be positive integer".to_string())?;
    if min_sample_count < 10 {
        return Err("minimum_sample_count must be >= 10".to_string());
    }

    let min_confidence = contract["variance_confidence_contract"]["minimum_confidence_level"]
        .as_f64()
        .ok_or_else(|| "minimum_confidence_level must be numeric".to_string())?;
    if !(0.0 < min_confidence && min_confidence <= 1.0) {
        return Err("minimum_confidence_level must be in (0.0, 1.0]".to_string());
    }

    if contract["variance_confidence_contract"]["fail_closed_on_insufficient_confidence"]
        != Value::Bool(true)
    {
        return Err(
            "variance_confidence_contract.fail_closed_on_insufficient_confidence must be true"
                .to_string(),
        );
    }

    Ok(())
}

fn validate_gate_and_evidence_contract(contract: &Value) -> ValidationResult<()> {
    validate_required_set(
        contract,
        "/gate_evaluation_contract/required_gate_checks",
        REQUIRED_GATE_CHECKS,
        "gate check",
    )?;
    validate_required_set(
        contract,
        "/evidence_contract/required_rejection_checks",
        REQUIRED_REJECTION_CHECKS,
        "rejection check",
    )?;

    if contract["gate_evaluation_contract"]["fail_closed"] != Value::Bool(true) {
        return Err("gate_evaluation_contract.fail_closed must be true".to_string());
    }

    for threshold in [
        "min_speedup_vs_node",
        "min_speedup_vs_bun",
        "max_rss_increase_ratio",
        "max_cpu_time_increase_ratio",
    ] {
        let value = contract["gate_evaluation_contract"]["strict_gate_thresholds"][threshold]
            .as_f64()
            .ok_or_else(|| format!("strict_gate_thresholds.{threshold} must be numeric"))?;
        if value <= 0.0 {
            return Err(format!("strict_gate_thresholds.{threshold} must be > 0"));
        }
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

fn set_microbench_guard(contract: &mut Value, enabled: bool) {
    let field = contract
        .pointer_mut("/workload_protocol/forbid_microbench_only_claims")
        .expect("workload_protocol.forbid_microbench_only_claims must be mutable");
    *field = Value::Bool(enabled);
}

#[test]
fn benchmark_resource_gate_contract_exists_and_is_valid_json() {
    let path = repo_root().join(CONTRACT_PATH);
    assert!(
        path.is_file(),
        "missing benchmark/resource gate contract artifact: {}",
        path.display()
    );
    let _ = load_contract();
}

#[test]
fn benchmark_resource_gate_contract_has_expected_schema_version_and_linkage() {
    let contract = load_contract();
    assert_eq!(
        contract["schema"],
        Value::String(EXPECTED_SCHEMA.to_string()),
        "benchmark/resource gate contract schema mismatch"
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
        Value::String("bd-3ar8v.7.10".to_string()),
        "bead linkage must target bd-3ar8v.7.10"
    );
    assert_eq!(
        contract["support_bead_id"],
        Value::String("bd-3ar8v.7.10.1".to_string()),
        "support bead linkage must target bd-3ar8v.7.10.1"
    );
}

#[test]
fn benchmark_resource_gate_workload_protocol_is_complete() {
    let contract = load_contract();
    validate_workload_protocol(&contract).unwrap_or_else(|err| panic!("{err}"));
}

#[test]
fn benchmark_resource_gate_metric_and_confidence_contract_is_complete() {
    let contract = load_contract();
    validate_metric_and_confidence_contract(&contract).unwrap_or_else(|err| panic!("{err}"));
}

#[test]
fn benchmark_resource_gate_gate_and_evidence_contract_is_fail_closed() {
    let contract = load_contract();
    validate_gate_and_evidence_contract(&contract).unwrap_or_else(|err| panic!("{err}"));
}

#[test]
fn benchmark_resource_gate_logging_and_dependencies_are_complete() {
    let contract = load_contract();
    validate_logging_and_dependencies(&contract).unwrap_or_else(|err| panic!("{err}"));
}

#[test]
fn benchmark_resource_gate_contract_fails_closed_when_workload_class_missing() {
    let mut contract = load_contract();
    assert!(
        remove_string_entry(
            &mut contract,
            "/workload_protocol/required_workload_classes",
            "macro_policy_restricted_path",
        ),
        "mutation should remove required workload class"
    );

    let err = validate_workload_protocol(&contract)
        .expect_err("contract should fail when required workload class is removed");
    assert!(
        err.contains("macro_policy_restricted_path"),
        "expected error to reference missing workload class, got: {err}"
    );
}

#[test]
fn benchmark_resource_gate_contract_fails_closed_when_comparison_target_missing() {
    let mut contract = load_contract();
    assert!(
        remove_string_entry(
            &mut contract,
            "/workload_protocol/required_comparison_targets",
            "bun",
        ),
        "mutation should remove required comparison target"
    );

    let err = validate_workload_protocol(&contract)
        .expect_err("contract should fail when required comparison target is removed");
    assert!(
        err.contains("bun"),
        "expected error to reference missing comparison target, got: {err}"
    );
}

#[test]
fn benchmark_resource_gate_contract_fails_closed_when_microbench_guard_disabled() {
    let mut contract = load_contract();
    set_microbench_guard(&mut contract, false);

    let err = validate_workload_protocol(&contract)
        .expect_err("contract should fail when microbench-only rejection guard is disabled");
    assert!(
        err.contains("forbid_microbench_only_claims"),
        "expected error to reference microbench guard, got: {err}"
    );
}

#[test]
fn benchmark_resource_gate_contract_fails_closed_when_gate_check_missing() {
    let mut contract = load_contract();
    assert!(
        remove_string_entry(
            &mut contract,
            "/gate_evaluation_contract/required_gate_checks",
            "microbench_only_claim_blocked",
        ),
        "mutation should remove required gate check"
    );

    let err = validate_gate_and_evidence_contract(&contract)
        .expect_err("contract should fail when required gate check is removed");
    assert!(
        err.contains("microbench_only_claim_blocked"),
        "expected error to reference missing gate check, got: {err}"
    );
}

#[test]
fn benchmark_resource_gate_contract_fails_closed_when_rejection_check_missing() {
    let mut contract = load_contract();
    assert!(
        remove_string_entry(
            &mut contract,
            "/evidence_contract/required_rejection_checks",
            "claim_integrity.microbench_only_claim",
        ),
        "mutation should remove required rejection check"
    );

    let err = validate_gate_and_evidence_contract(&contract)
        .expect_err("contract should fail when required rejection check is removed");
    assert!(
        err.contains("claim_integrity.microbench_only_claim"),
        "expected error to reference missing rejection check, got: {err}"
    );
}

#[test]
fn benchmark_resource_gate_contract_fails_closed_when_blocked_bead_missing() {
    let mut contract = load_contract();
    assert!(
        remove_string_entry(
            &mut contract,
            "/downstream_dependencies/blocked_beads",
            "bd-3ar8v.7.16",
        ),
        "mutation should remove required blocked bead"
    );

    let err = validate_logging_and_dependencies(&contract)
        .expect_err("contract should fail when blocked bead linkage is missing");
    assert!(
        err.contains("bd-3ar8v.7.16"),
        "expected error to reference missing blocked bead, got: {err}"
    );
}
