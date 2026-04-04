use serde_json::Value;
use std::collections::HashSet;
use std::path::PathBuf;

const CONTRACT_PATH: &str = "docs/franken-node-package-interop-contract.json";
const EXPECTED_SCHEMA: &str = "pi.frankennode.package_interop_contract.v1";
const REQUIRED_SCENARIOS: &[&str] = &[
    "SCN-package-exports-condition-priority",
    "SCN-cjs-esm-entrypoint-bridging",
    "SCN-npm-workspace-lockfile-resolution",
    "SCN-peer-dependency-constraint-evaluation",
    "SCN-conditional-main-module-field-fallback",
];
const REQUIRED_DIAG_CODES: &[&str] = &[
    "interop.missing_exports",
    "interop.unsupported_condition_order",
    "interop.peer_dependency_mismatch",
    "interop.lockfile_graph_nondeterminism",
];
const REQUIRED_SUPPORT_BEAD_IDS: &[&str] = &["bd-3ar8v.7.4.1", "bd-3ar8v.7.4.6"];
const REQUIRED_CLAIM_ARTIFACTS: &[&str] = &[
    "docs/franken-node-mission-contract.json",
    "docs/franken-node-semantic-compatibility-matrix-contract.json",
    "docs/franken-node-package-interop-contract.json",
];
const REQUIRED_CLAIM_CHECK_IDS: &[&str] = &[
    "claim_integrity.franken_node_phase6_runtime_beads_declared",
    "claim_integrity.franken_node_strict_replacement_dropin_certified",
    "claim_integrity.franken_node_strict_tier_required_evidence",
];
const REQUIRED_BLOCKED_BEADS: &[&str] = &[
    "bd-3ar8v.7.5",
    "bd-3ar8v.7.10",
    "bd-3ar8v.7.11",
    "bd-3ar8v.7.14",
];

type ValidationResult<T> = std::result::Result<T, String>;

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

fn collect_scenario_ids(contract: &Value) -> ValidationResult<HashSet<String>> {
    let mut ids = HashSet::new();
    for scenario in as_array(contract, "/scenario_taxonomy") {
        let scenario_id = scenario
            .get("scenario_id")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|entry| !entry.is_empty())
            .ok_or_else(|| "every scenario row must include non-empty scenario_id".to_string())?;
        if !ids.insert(scenario_id.to_string()) {
            return Err(format!("duplicate scenario_id detected: {scenario_id}"));
        }

        let module_system = scenario
            .get("module_system")
            .and_then(Value::as_str)
            .map(str::trim)
            .ok_or_else(|| format!("{scenario_id}: module_system must be present"))?;
        if !matches!(module_system, "cjs" | "esm" | "dual-mode" | "npm") {
            return Err(format!(
                "{scenario_id}: invalid module_system value {module_system}"
            ));
        }

        let criticality = scenario
            .get("criticality")
            .and_then(Value::as_str)
            .map(str::trim)
            .ok_or_else(|| format!("{scenario_id}: criticality must be present"))?;
        if !matches!(criticality, "high" | "medium" | "low") {
            return Err(format!(
                "{scenario_id}: invalid criticality value {criticality}"
            ));
        }

        let required_signals = scenario
            .get("required_signals")
            .and_then(Value::as_array)
            .ok_or_else(|| format!("{scenario_id}: required_signals must be an array"))?;
        if required_signals.is_empty() {
            return Err(format!("{scenario_id}: required_signals must not be empty"));
        }
    }
    Ok(ids)
}

fn validate_required_scenarios(contract: &Value) -> ValidationResult<()> {
    let scenario_ids = collect_scenario_ids(contract)?;
    for required in REQUIRED_SCENARIOS {
        if !scenario_ids.contains(*required) {
            return Err(format!("missing required scenario_id: {required}"));
        }
    }
    Ok(())
}

fn collect_diag_codes(contract: &Value) -> ValidationResult<HashSet<String>> {
    let mut codes = HashSet::new();
    for entry in as_array(
        contract,
        "/resolution_engine_policy/deterministic_fallback_diagnostics/required_diag_codes",
    ) {
        let code = entry
            .as_str()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| "required_diag_codes entries must be non-empty strings".to_string())?;
        if !codes.insert(code.to_string()) {
            return Err(format!("duplicate diagnostic code detected: {code}"));
        }
    }
    Ok(codes)
}

fn validate_required_diag_codes(contract: &Value) -> ValidationResult<()> {
    let codes = collect_diag_codes(contract)?;
    for required in REQUIRED_DIAG_CODES {
        if !codes.contains(*required) {
            return Err(format!("missing required diagnostic code: {required}"));
        }
    }
    Ok(())
}

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

fn remove_string_entry(contract: &mut Value, pointer: &str, value: &str) -> bool {
    let entries = contract
        .pointer_mut(pointer)
        .and_then(Value::as_array_mut)
        .unwrap_or_else(|| panic!("expected mutable array at pointer {pointer}"));
    let before = entries.len();
    entries.retain(|entry| entry.as_str().map(str::trim) != Some(value));
    before != entries.len()
}

fn remove_scenario(contract: &mut Value, scenario_id: &str) -> bool {
    let scenarios = contract
        .pointer_mut("/scenario_taxonomy")
        .and_then(Value::as_array_mut)
        .expect("scenario_taxonomy must be mutable array");
    let before = scenarios.len();
    scenarios.retain(|row| {
        row.get("scenario_id")
            .and_then(Value::as_str)
            .map(str::trim)
            != Some(scenario_id)
    });
    before != scenarios.len()
}

fn remove_diag_code(contract: &mut Value, diag_code: &str) -> bool {
    remove_string_entry(
        contract,
        "/resolution_engine_policy/deterministic_fallback_diagnostics/required_diag_codes",
        diag_code,
    )
}

#[test]
fn package_interop_contract_exists_and_is_valid_json() {
    let path = repo_root().join(CONTRACT_PATH);
    assert!(
        path.is_file(),
        "missing package interop contract artifact: {}",
        path.display()
    );
    let _ = load_contract();
}

#[test]
fn package_interop_contract_has_expected_schema_and_bead_linkage() {
    let contract = load_contract();
    assert_eq!(
        contract["schema"],
        Value::String(EXPECTED_SCHEMA.to_string()),
        "package interop contract schema mismatch"
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
        Value::String("bd-3ar8v.7.4".to_string()),
        "bead linkage must target bd-3ar8v.7.4"
    );
    assert_eq!(
        contract["support_bead_id"],
        Value::String("bd-3ar8v.7.4.1".to_string()),
        "support bead linkage must target bd-3ar8v.7.4.1"
    );
    validate_required_set(
        &contract,
        "/support_bead_ids",
        REQUIRED_SUPPORT_BEAD_IDS,
        "support_bead_ids entry",
    )
    .unwrap_or_else(|err| panic!("{err}"));
}

#[test]
fn package_interop_contract_scenario_taxonomy_is_complete_and_unique() {
    let contract = load_contract();
    validate_required_scenarios(&contract).unwrap_or_else(|err| panic!("{err}"));
}

#[test]
fn package_interop_contract_metadata_interpretation_is_fail_closed() {
    let contract = load_contract();
    let required_fields = non_empty_string_set(
        &contract,
        "/package_metadata_interpretation/required_package_json_fields",
    );
    for required in ["name", "version", "type", "main", "module", "exports"] {
        assert!(
            required_fields.contains(required),
            "required_package_json_fields missing {required}"
        );
    }

    let priority = non_empty_string_set(
        &contract,
        "/package_metadata_interpretation/conditional_exports_priority",
    );
    for required in ["node", "import", "require", "default"] {
        assert!(
            priority.contains(required),
            "conditional_exports_priority missing {required}"
        );
    }

    assert_eq!(
        contract["package_metadata_interpretation"]["enforce_explicit_type_for_dual_mode_packages"]
            .as_bool(),
        Some(true),
        "enforce_explicit_type_for_dual_mode_packages must be true"
    );
    assert_eq!(
        contract["package_metadata_interpretation"]["fallback_on_missing_exports"],
        Value::String("hard_fail_with_diag".to_string()),
        "fallback_on_missing_exports must be hard_fail_with_diag"
    );
    assert_eq!(
        contract["package_metadata_interpretation"]["peer_dependency_policy"]["missing_peer_strategy"],
        Value::String("report_and_fail".to_string()),
        "peer_dependency_policy.missing_peer_strategy must be report_and_fail"
    );
}

#[test]
fn package_interop_contract_fallback_diagnostics_and_claim_linkage_are_declared() {
    let contract = load_contract();
    validate_required_diag_codes(&contract).unwrap_or_else(|err| panic!("{err}"));

    assert_eq!(
        contract["resolution_engine_policy"]["deterministic_fallback_diagnostics"]["failure_policy"],
        Value::String("hard_fail".to_string()),
        "deterministic fallback failure_policy must be hard_fail"
    );
    assert_eq!(
        contract["resolution_engine_policy"]["unsupported_edge_policy"]["require_explicit_unsupported_entries"]
            .as_bool(),
        Some(true),
        "unsupported_edge_policy.require_explicit_unsupported_entries must be true"
    );
    assert_eq!(
        contract["resolution_engine_policy"]["unsupported_edge_policy"]["silent_omission_policy"],
        Value::String("hard_fail".to_string()),
        "unsupported_edge_policy.silent_omission_policy must be hard_fail"
    );

    assert_eq!(
        contract["claim_tier_linkage"]["strict_tier_id"],
        Value::String("full_runtime_replacement".to_string()),
        "claim tier linkage must target full_runtime_replacement"
    );
    validate_required_set(
        &contract,
        "/claim_tier_linkage/required_beads",
        &["bd-3ar8v.7.2", "bd-3ar8v.7.3", "bd-3ar8v.7.4"],
        "claim_tier_linkage.required_beads",
    )
    .unwrap_or_else(|err| panic!("{err}"));
    validate_required_set(
        &contract,
        "/claim_tier_linkage/required_artifacts",
        REQUIRED_CLAIM_ARTIFACTS,
        "claim_tier_linkage.required_artifacts",
    )
    .unwrap_or_else(|err| panic!("{err}"));
    validate_required_set(
        &contract,
        "/claim_tier_linkage/required_check_ids",
        REQUIRED_CLAIM_CHECK_IDS,
        "claim_tier_linkage.required_check_ids",
    )
    .unwrap_or_else(|err| panic!("{err}"));
    validate_required_set(
        &contract,
        "/downstream_dependencies/blocked_beads",
        REQUIRED_BLOCKED_BEADS,
        "downstream_dependencies.blocked_beads",
    )
    .unwrap_or_else(|err| panic!("{err}"));
}

#[test]
fn package_interop_contract_duplicate_scenario_mutation_fails_closed() {
    let mut contract = load_contract();
    let scenarios = contract
        .pointer_mut("/scenario_taxonomy")
        .and_then(Value::as_array_mut)
        .expect("scenario_taxonomy must be mutable array");
    let duplicate = scenarios
        .iter()
        .find(|scenario| {
            scenario["scenario_id"].as_str() == Some("SCN-cjs-esm-entrypoint-bridging")
        })
        .cloned()
        .expect("mutation setup requires SCN-cjs-esm-entrypoint-bridging scenario");
    scenarios.push(duplicate);

    let err = collect_scenario_ids(&contract)
        .expect_err("duplicate scenario taxonomy entry must fail validation");
    assert!(
        err.contains("SCN-cjs-esm-entrypoint-bridging"),
        "error should reference duplicate scenario_id, got: {err}"
    );
}

#[test]
fn package_interop_contract_missing_required_scenario_mutation_fails_closed() {
    let mut contract = load_contract();
    assert!(
        remove_scenario(&mut contract, "SCN-npm-workspace-lockfile-resolution"),
        "mutation setup should remove SCN-npm-workspace-lockfile-resolution"
    );

    let err = validate_required_scenarios(&contract)
        .expect_err("missing required scenario must fail validation");
    assert!(
        err.contains("SCN-npm-workspace-lockfile-resolution"),
        "error should reference missing required scenario, got: {err}"
    );
}

#[test]
fn package_interop_contract_missing_diag_code_mutation_fails_closed() {
    let mut contract = load_contract();
    assert!(
        remove_diag_code(&mut contract, "interop.peer_dependency_mismatch"),
        "mutation setup should remove interop.peer_dependency_mismatch"
    );

    let err = validate_required_diag_codes(&contract)
        .expect_err("missing required diag code must fail validation");
    assert!(
        err.contains("interop.peer_dependency_mismatch"),
        "error should reference missing required diagnostic code, got: {err}"
    );
}

#[test]
fn package_interop_contract_missing_required_artifact_mutation_fails_closed() {
    let mut contract = load_contract();
    assert!(
        remove_string_entry(
            &mut contract,
            "/claim_tier_linkage/required_artifacts",
            "docs/franken-node-semantic-compatibility-matrix-contract.json"
        ),
        "mutation setup should remove docs/franken-node-semantic-compatibility-matrix-contract.json"
    );

    let err = validate_required_set(
        &contract,
        "/claim_tier_linkage/required_artifacts",
        REQUIRED_CLAIM_ARTIFACTS,
        "claim_tier_linkage.required_artifacts",
    )
    .expect_err("missing required artifact must fail validation");
    assert!(
        err.contains("docs/franken-node-semantic-compatibility-matrix-contract.json"),
        "error should reference missing required artifact, got: {err}"
    );
}

#[test]
fn package_interop_contract_missing_required_check_id_mutation_fails_closed() {
    let mut contract = load_contract();
    assert!(
        remove_string_entry(
            &mut contract,
            "/claim_tier_linkage/required_check_ids",
            "claim_integrity.franken_node_strict_tier_required_evidence"
        ),
        "mutation setup should remove claim_integrity.franken_node_strict_tier_required_evidence"
    );

    let err = validate_required_set(
        &contract,
        "/claim_tier_linkage/required_check_ids",
        REQUIRED_CLAIM_CHECK_IDS,
        "claim_tier_linkage.required_check_ids",
    )
    .expect_err("missing required check id must fail validation");
    assert!(
        err.contains("claim_integrity.franken_node_strict_tier_required_evidence"),
        "error should reference missing required check id, got: {err}"
    );
}

#[test]
fn package_interop_contract_missing_blocked_bead_mutation_fails_closed() {
    let mut contract = load_contract();
    assert!(
        remove_string_entry(
            &mut contract,
            "/downstream_dependencies/blocked_beads",
            "bd-3ar8v.7.11"
        ),
        "mutation setup should remove bd-3ar8v.7.11"
    );

    let err = validate_required_set(
        &contract,
        "/downstream_dependencies/blocked_beads",
        REQUIRED_BLOCKED_BEADS,
        "downstream_dependencies.blocked_beads",
    )
    .expect_err("missing blocked bead linkage must fail validation");
    assert!(
        err.contains("bd-3ar8v.7.11"),
        "error should reference missing blocked bead, got: {err}"
    );
}
