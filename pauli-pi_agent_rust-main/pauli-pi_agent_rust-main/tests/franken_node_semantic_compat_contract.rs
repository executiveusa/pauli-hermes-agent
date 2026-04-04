use serde_json::Value;
use std::collections::HashSet;
use std::path::PathBuf;

const CONTRACT_PATH: &str = "docs/franken-node-semantic-compatibility-matrix-contract.json";

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

fn normalize_upper_snake(raw: &str) -> String {
    let mut normalized = String::with_capacity(raw.len());
    let mut last_was_sep = false;
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() {
            normalized.push(ch.to_ascii_uppercase());
            last_was_sep = false;
        } else if !last_was_sep {
            normalized.push('_');
            last_was_sep = true;
        }
    }
    normalized.trim_matches('_').to_string()
}

#[test]
fn semantic_compat_contract_exists_and_is_valid_json() {
    let path = repo_root().join(CONTRACT_PATH);
    assert!(
        path.is_file(),
        "missing semantic compatibility contract artifact: {}",
        path.display()
    );
    let _ = load_contract();
}

#[test]
fn semantic_compat_contract_has_expected_schema_and_bead_linkage() {
    let contract = load_contract();
    assert_eq!(
        contract["schema"],
        Value::String("pi.frankennode.semantic_compatibility_matrix_contract.v1".to_string()),
        "semantic compatibility contract schema mismatch"
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
        Value::String("bd-3ar8v.7.3".to_string()),
        "bead linkage must target bd-3ar8v.7.3"
    );
    assert_eq!(
        contract["support_bead_id"],
        Value::String("bd-3ar8v.7.3.1".to_string()),
        "support bead linkage must target bd-3ar8v.7.3.1"
    );

    let support_bead_ids = contract["support_bead_ids"]
        .as_array()
        .expect("support_bead_ids must be an array");
    let support_bead_set: HashSet<&str> =
        support_bead_ids.iter().filter_map(Value::as_str).collect();
    for required in ["bd-3ar8v.7.3.1", "bd-3ar8v.7.3.7", "bd-3ar8v.7.4.5"] {
        assert!(
            support_bead_set.contains(required),
            "support_bead_ids missing required linkage: {required}"
        );
    }
}

#[test]
fn semantic_compat_contract_scenario_taxonomy_is_complete_and_unique() {
    let contract = load_contract();
    let scenarios = contract["scenario_taxonomy"]
        .as_array()
        .expect("scenario_taxonomy must be an array");
    assert!(
        scenarios.len() >= 6,
        "scenario_taxonomy must define at least six core scenarios"
    );

    let mut scenario_ids = HashSet::new();
    for scenario in scenarios {
        let id = scenario["scenario_id"]
            .as_str()
            .expect("scenario_id must be present on every taxonomy row");
        assert!(
            scenario_ids.insert(id),
            "duplicate scenario_id detected in taxonomy: {id}"
        );

        let criticality = scenario["criticality"]
            .as_str()
            .expect("criticality must be present on every taxonomy row");
        assert!(
            matches!(criticality, "high" | "medium" | "low"),
            "invalid criticality level for {id}: {criticality}"
        );

        let required_surfaces = scenario["required_surfaces"]
            .as_array()
            .unwrap_or_else(|| panic!("{id}: required_surfaces must be an array"));
        assert!(
            !required_surfaces.is_empty(),
            "{id}: required_surfaces must not be empty"
        );
    }

    for required in [
        "SCN-module-resolution-esm-cjs",
        "SCN-node-builtin-apis",
        "SCN-event-loop-io-ordering",
        "SCN-tooling-and-package-workflows",
        "SCN-package-interop-cjs-esm-npm",
        "SCN-error-and-diagnostics-parity",
    ] {
        assert!(
            scenario_ids.contains(required),
            "scenario taxonomy missing required scenario_id: {required}"
        );
    }
}

#[test]
fn semantic_compat_contract_verdict_policy_is_fail_closed() {
    let contract = load_contract();
    let allowed = contract["verdict_policy"]["allowed_row_verdicts"]
        .as_array()
        .expect("verdict_policy.allowed_row_verdicts must be an array");
    let allowed_set: HashSet<&str> = allowed.iter().filter_map(Value::as_str).collect();
    for required in [
        "EXACT_PARITY",
        "ACCEPTABLE_SUPERSET",
        "PARTIAL_PARITY",
        "UNSUPPORTED",
        "INCOMPATIBLE",
    ] {
        assert!(
            allowed_set.contains(required),
            "allowed_row_verdicts missing required verdict: {required}"
        );
    }

    let blockers = contract["verdict_policy"]["release_blockers"]
        .as_array()
        .expect("verdict_policy.release_blockers must be an array");
    assert!(
        !blockers.is_empty(),
        "verdict_policy.release_blockers must not be empty"
    );
    assert!(
        blockers
            .iter()
            .filter_map(Value::as_str)
            .any(|entry| entry.contains("package interop scenario rows")),
        "release_blockers must include package interop scenario coverage blocker"
    );

    let rules = &contract["verdict_policy"]["global_claim_rules"];
    for required_bool in [
        "forbid_full_replacement_when_any_high_row_non_exact",
        "forbid_global_claim_when_lineage_missing",
        "require_explicit_scope_for_partial_parity",
        "require_package_interop_policy_coverage",
    ] {
        assert_eq!(
            rules[required_bool].as_bool(),
            Some(true),
            "global_claim_rules.{required_bool} must be true"
        );
    }
}

#[test]
fn semantic_compat_contract_declares_package_interop_policy_fail_closed() {
    let contract = load_contract();
    let policy = &contract["package_interop_policy"];

    let required_tiers = policy["required_for_claim_tiers"]
        .as_array()
        .expect("package_interop_policy.required_for_claim_tiers must be an array");
    let required_tier_set: HashSet<&str> =
        required_tiers.iter().filter_map(Value::as_str).collect();
    for required in ["ecosystem_compatibility", "full_runtime_replacement"] {
        assert!(
            required_tier_set.contains(required),
            "required_for_claim_tiers missing required value: {required}"
        );
    }

    let required_scenarios = policy["required_scenario_ids"]
        .as_array()
        .expect("package_interop_policy.required_scenario_ids must be an array");
    let required_scenario_set: HashSet<&str> = required_scenarios
        .iter()
        .filter_map(Value::as_str)
        .collect();
    for required in [
        "SCN-package-interop-cjs-esm-npm",
        "SCN-tooling-and-package-workflows",
    ] {
        assert!(
            required_scenario_set.contains(required),
            "required_scenario_ids missing required value: {required}"
        );
    }
    let taxonomy = contract["scenario_taxonomy"]
        .as_array()
        .expect("scenario_taxonomy must be an array");
    let package_interop_row = taxonomy
        .iter()
        .find(|row| {
            row.get("scenario_id").and_then(Value::as_str)
                == Some("SCN-package-interop-cjs-esm-npm")
        })
        .expect("scenario_taxonomy must include SCN-package-interop-cjs-esm-npm");
    let package_surfaces = package_interop_row["required_surfaces"]
        .as_array()
        .expect("SCN-package-interop-cjs-esm-npm.required_surfaces must be an array");
    let package_surface_set: HashSet<&str> =
        package_surfaces.iter().filter_map(Value::as_str).collect();
    for required in [
        "cjs-entrypoints",
        "esm-entrypoints",
        "package-json-main",
        "package-json-type-module",
        "conditional-exports",
        "fallback-diagnostics",
    ] {
        assert!(
            package_surface_set.contains(required),
            "SCN-package-interop-cjs-esm-npm.required_surfaces missing required value: {required}"
        );
    }

    let required_fields = policy["required_diagnostic_fields"]
        .as_array()
        .expect("package_interop_policy.required_diagnostic_fields must be an array");
    let required_field_set: HashSet<&str> =
        required_fields.iter().filter_map(Value::as_str).collect();
    for required in [
        "fallback_diagnostics_signature",
        "fallback_reason_code",
        "fallback_remediation_tag",
    ] {
        assert!(
            required_field_set.contains(required),
            "required_diagnostic_fields missing required value: {required}"
        );
    }

    assert_eq!(
        policy["require_deterministic_fallback_diagnostics"].as_bool(),
        Some(true),
        "require_deterministic_fallback_diagnostics must be true"
    );
    assert_eq!(
        policy["missing_policy_behavior"].as_str(),
        Some("hard_fail"),
        "missing_policy_behavior must be hard_fail"
    );
}

#[test]
fn semantic_compat_contract_lineage_fields_are_required_and_hard_fail() {
    let contract = load_contract();
    let fields = contract["evidence_lineage_contract"]["required_fields"]
        .as_array()
        .expect("evidence_lineage_contract.required_fields must be an array");
    let field_set: HashSet<&str> = fields.iter().filter_map(Value::as_str).collect();
    for required in [
        "run_id",
        "scenario_id",
        "fixture_id",
        "oracle_source",
        "observed_runtime",
        "comparison_result",
        "artifact_path",
        "captured_at_utc",
    ] {
        assert!(
            field_set.contains(required),
            "required lineage field missing: {required}"
        );
    }

    assert_eq!(
        contract["evidence_lineage_contract"]["lineage_failure_policy"],
        Value::String("hard_fail".to_string()),
        "lineage failure policy must be hard_fail"
    );
}

#[test]
fn semantic_compat_contract_declares_incremental_tiering_and_explicit_unsupported_policy() {
    let contract = load_contract();
    let policy = &contract["incremental_tiering_policy"];

    assert_eq!(
        policy["enabled"].as_bool(),
        Some(true),
        "incremental_tiering_policy.enabled must be true"
    );
    assert_eq!(
        policy["active_tier"].as_str(),
        Some("tier0-core"),
        "incremental_tiering_policy.active_tier must be tier0-core"
    );

    let tiers = policy["tier_definitions"]
        .as_array()
        .expect("incremental_tiering_policy.tier_definitions must be an array");
    assert!(
        tiers.len() >= 3,
        "tier_definitions must include at least three tiers"
    );
    let mut tier_ids = HashSet::new();
    let mut tier_supported_scenarios = std::collections::HashMap::new();
    for tier in tiers {
        let tier_id = tier["tier_id"]
            .as_str()
            .expect("tier_definitions entries must include tier_id");
        assert!(tier_ids.insert(tier_id), "duplicate tier_id: {tier_id}");
        let supported = tier["supported_scenario_ids"]
            .as_array()
            .expect("tier_definitions entries must include supported_scenario_ids array");
        assert!(
            !supported.is_empty(),
            "{tier_id}: supported_scenario_ids must not be empty"
        );
        tier_supported_scenarios.insert(
            tier_id.to_string(),
            supported
                .iter()
                .filter_map(Value::as_str)
                .map(ToString::to_string)
                .collect::<HashSet<_>>(),
        );
    }
    for required in ["tier0-core", "tier1-expanded", "tier2-longtail"] {
        assert!(tier_ids.contains(required), "missing tier_id {required}");
    }
    let tier1_expanded_scenarios = tier_supported_scenarios
        .get("tier1-expanded")
        .expect("tier1-expanded must be present in tier_definitions");
    for required in [
        "SCN-tooling-and-package-workflows",
        "SCN-package-interop-cjs-esm-npm",
    ] {
        assert!(
            tier1_expanded_scenarios.contains(required),
            "tier1-expanded supported_scenario_ids missing required scenario: {required}"
        );
    }

    let unsupported = &policy["unsupported_class_policy"];
    assert_eq!(
        unsupported["explicit_unsupported_required_for_out_of_tier"].as_bool(),
        Some(true),
        "explicit_unsupported_required_for_out_of_tier must be true"
    );
    assert_eq!(
        unsupported["unsupported_verdict"].as_str(),
        Some("UNSUPPORTED"),
        "unsupported_verdict must be UNSUPPORTED"
    );
    assert_eq!(
        unsupported["silent_omission_policy"].as_str(),
        Some("hard_fail"),
        "silent_omission_policy must be hard_fail"
    );

    let metadata_fields = unsupported["required_metadata_fields"]
        .as_array()
        .expect("required_metadata_fields must be an array");
    let metadata_set: HashSet<&str> = metadata_fields.iter().filter_map(Value::as_str).collect();
    for required in ["unsupported_reason_code", "remediation_tag"] {
        assert!(
            metadata_set.contains(required),
            "required_metadata_fields missing required value: {required}"
        );
    }
}

#[test]
fn semantic_compat_contract_declares_executable_row_schema_and_adjudication_policy() {
    let contract = load_contract();
    let row_schema = &contract["executable_row_schema"];
    let required_fields = row_schema["required_fields"]
        .as_array()
        .expect("executable_row_schema.required_fields must be an array");
    let required_field_set: HashSet<&str> =
        required_fields.iter().filter_map(Value::as_str).collect();

    for required in [
        "scenario_id",
        "expected_baseline",
        "observed_runtime",
        "comparison_result",
        "verdict",
        "lineage",
    ] {
        assert!(
            required_field_set.contains(required),
            "required executable row field missing: {required}"
        );
    }

    assert_eq!(
        row_schema["scenario_id_normalization"]["trim_whitespace"].as_bool(),
        Some(true),
        "scenario_id_normalization.trim_whitespace must be true"
    );
    assert_eq!(
        row_schema["scenario_id_normalization"]["require_exact_taxonomy_match"].as_bool(),
        Some(true),
        "scenario_id_normalization.require_exact_taxonomy_match must be true"
    );
    assert_eq!(
        row_schema["verdict_normalization"]["trim_whitespace"].as_bool(),
        Some(true),
        "verdict_normalization.trim_whitespace must be true"
    );
    assert_eq!(
        row_schema["verdict_normalization"]["case"],
        Value::String("upper_snake".to_string()),
        "verdict_normalization.case must be upper_snake"
    );
    assert_eq!(
        row_schema["adjudication_policy"]["rule"],
        Value::String("comparison_result_must_match_verdict_after_normalization".to_string()),
        "adjudication_policy.rule mismatch"
    );
    assert_eq!(
        row_schema["adjudication_policy"]["failure_policy"],
        Value::String("hard_fail".to_string()),
        "adjudication_policy.failure_policy must be hard_fail"
    );
}

#[test]
fn semantic_compat_contract_declares_downstream_blocking_and_integration_links() {
    let contract = load_contract();
    let blocked = contract["downstream_dependencies"]["blocked_beads"]
        .as_array()
        .expect("downstream_dependencies.blocked_beads must be an array");
    let blocked_set: HashSet<&str> = blocked.iter().filter_map(Value::as_str).collect();
    for required in [
        "bd-3ar8v.7.4",
        "bd-3ar8v.7.5",
        "bd-3ar8v.7.8",
        "bd-3ar8v.7.11",
        "bd-3ar8v.7.14",
    ] {
        assert!(
            blocked_set.contains(required),
            "blocked_beads must include downstream dependency: {required}"
        );
    }

    let integration_contracts = contract["downstream_dependencies"]["integration_contracts"]
        .as_array()
        .expect("downstream_dependencies.integration_contracts must be an array");
    let integration_set: HashSet<&str> = integration_contracts
        .iter()
        .filter_map(Value::as_str)
        .collect();
    assert!(
        integration_set.contains("docs/franken-node-claim-gating-contract.json"),
        "integration contract linkage must include docs/franken-node-claim-gating-contract.json"
    );
    assert!(
        integration_set.contains("docs/franken-node-package-interop-contract.json"),
        "integration contract linkage must include docs/franken-node-package-interop-contract.json"
    );
}

fn sample_lineage(scenario_id: &str) -> Value {
    serde_json::json!({
        "run_id": "run-franken-semantic-001",
        "scenario_id": scenario_id,
        "fixture_id": format!("fixture-{scenario_id}"),
        "oracle_source": "node-baseline-fixture",
        "observed_runtime": "frankennode",
        "comparison_result": "EXACT_PARITY",
        "artifact_path": format!("tests/e2e_results/{scenario_id}/semantic_row.json"),
        "captured_at_utc": "2026-02-17T00:00:00Z"
    })
}

fn sample_row(scenario_id: &str, verdict: &str) -> Value {
    serde_json::json!({
        "scenario_id": scenario_id,
        "expected_baseline": "Node.js",
        "observed_runtime": "frankennode",
        "comparison_result": verdict,
        "verdict": verdict,
        "lineage": sample_lineage(scenario_id),
    })
}

fn sample_unsupported_row(scenario_id: &str) -> Value {
    serde_json::json!({
        "scenario_id": scenario_id,
        "expected_baseline": "Node.js",
        "observed_runtime": "frankennode",
        "comparison_result": "UNSUPPORTED",
        "verdict": "UNSUPPORTED",
        "unsupported_reason_code": "NOT_YET_IMPLEMENTED",
        "remediation_tag": "tier-upgrade-required",
        "fallback_diagnostics_signature": format!("diag-{scenario_id}"),
        "fallback_reason_code": "NOT_YET_IMPLEMENTED",
        "fallback_remediation_tag": "interop-coverage-gap",
        "lineage": sample_lineage(scenario_id),
    })
}

fn baseline_rows_with_explicit_unsupported() -> Vec<Value> {
    vec![
        sample_row("SCN-module-resolution-esm-cjs", "EXACT_PARITY"),
        sample_row("SCN-node-builtin-apis", "EXACT_PARITY"),
        sample_row("SCN-event-loop-io-ordering", "EXACT_PARITY"),
        sample_unsupported_row("SCN-package-interop-cjs-esm-npm"),
        sample_unsupported_row("SCN-tooling-and-package-workflows"),
        sample_unsupported_row("SCN-error-and-diagnostics-parity"),
    ]
}

#[allow(clippy::too_many_lines)]
fn evaluate_executable_semantic_matrix(contract: &Value, rows: &[Value]) -> Value {
    let taxonomy = contract["scenario_taxonomy"]
        .as_array()
        .expect("scenario_taxonomy must be an array");
    let taxonomy_ids = taxonomy
        .iter()
        .filter_map(|row| row.get("scenario_id").and_then(Value::as_str))
        .map(ToString::to_string)
        .collect::<HashSet<_>>();
    let taxonomy_map = taxonomy
        .iter()
        .filter_map(|row| {
            let id = row.get("scenario_id").and_then(Value::as_str)?;
            let criticality = row.get("criticality").and_then(Value::as_str)?;
            Some((id.to_string(), criticality.to_string()))
        })
        .collect::<std::collections::HashMap<_, _>>();
    let incremental_tiering = &contract["incremental_tiering_policy"];
    let tiering_enabled = incremental_tiering["enabled"].as_bool().unwrap_or(false);
    let active_tier = incremental_tiering["active_tier"].as_str().unwrap_or("");
    let mut active_tier_supported_scenarios = HashSet::new();
    let mut active_tier_found = false;
    if tiering_enabled {
        let tier_definitions = incremental_tiering["tier_definitions"]
            .as_array()
            .expect("incremental_tiering_policy.tier_definitions must be an array");
        for tier in tier_definitions {
            let tier_id = tier
                .get("tier_id")
                .and_then(Value::as_str)
                .unwrap_or_default();
            for scenario_id in tier["supported_scenario_ids"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
            {
                active_tier_supported_scenarios.insert(scenario_id.to_string());
            }
            if tier_id == active_tier {
                active_tier_found = true;
                break;
            }
        }
    }
    let unsupported_policy = &incremental_tiering["unsupported_class_policy"];
    let explicit_unsupported_required = tiering_enabled
        && unsupported_policy["explicit_unsupported_required_for_out_of_tier"]
            .as_bool()
            .unwrap_or(false);
    let unsupported_verdict = unsupported_policy["unsupported_verdict"]
        .as_str()
        .unwrap_or("UNSUPPORTED");
    let required_unsupported_metadata_fields = unsupported_policy["required_metadata_fields"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    let package_interop_policy = &contract["package_interop_policy"];
    let package_interop_policy_hard_fail = package_interop_policy["missing_policy_behavior"]
        .as_str()
        .is_some_and(|policy| policy == "hard_fail");
    let required_package_interop_scenarios = package_interop_policy["required_scenario_ids"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(ToString::to_string)
        .collect::<HashSet<_>>();
    let require_deterministic_fallback_diagnostics =
        package_interop_policy["require_deterministic_fallback_diagnostics"]
            .as_bool()
            .unwrap_or(false);
    let required_package_interop_diagnostic_fields =
        package_interop_policy["required_diagnostic_fields"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(ToString::to_string)
            .collect::<Vec<_>>();

    let required_lineage_fields = contract["evidence_lineage_contract"]["required_fields"]
        .as_array()
        .expect("required_fields must be an array")
        .iter()
        .filter_map(Value::as_str)
        .map(ToString::to_string)
        .collect::<Vec<_>>();

    let allowed_verdicts = contract["verdict_policy"]["allowed_row_verdicts"]
        .as_array()
        .expect("allowed_row_verdicts must be an array")
        .iter()
        .filter_map(Value::as_str)
        .collect::<HashSet<_>>();
    let executable_row_schema = &contract["executable_row_schema"];
    let required_row_fields = executable_row_schema["required_fields"]
        .as_array()
        .expect("executable_row_schema.required_fields must be an array")
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();
    let trim_scenario_id = executable_row_schema["scenario_id_normalization"]["trim_whitespace"]
        .as_bool()
        .unwrap_or(false);
    let trim_verdict = executable_row_schema["verdict_normalization"]["trim_whitespace"]
        .as_bool()
        .unwrap_or(false);
    let verdict_case = executable_row_schema["verdict_normalization"]["case"]
        .as_str()
        .unwrap_or("identity");
    let adjudication_rule = executable_row_schema["adjudication_policy"]["rule"]
        .as_str()
        .unwrap_or("");

    let mut covered_scenario_ids = HashSet::new();
    let mut incompatible_high = Vec::new();
    let mut missing_lineage = Vec::new();
    let mut missing_required_row_fields = Vec::new();
    let mut adjudication_mismatches = Vec::new();
    let mut unknown_scenarios = Vec::new();
    let mut invalid_verdict_rows = Vec::new();
    let mut explicit_unsupported_scenarios = HashSet::new();
    let mut unsupported_verdict_mismatches = Vec::new();
    let mut unsupported_metadata_missing = Vec::new();
    let mut package_interop_missing_diagnostics = Vec::new();
    let mut evaluated_rows = Vec::new();

    for row in rows {
        let raw_scenario_id = row
            .get("scenario_id")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let scenario_id = if trim_scenario_id {
            raw_scenario_id.trim().to_string()
        } else {
            raw_scenario_id
        };

        let missing_fields = required_row_fields
            .iter()
            .filter(|field| match **field {
                "lineage" => row.get("lineage").and_then(Value::as_object).is_none(),
                key => row
                    .get(key)
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .is_none_or(str::is_empty),
            })
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        if !missing_fields.is_empty() {
            missing_required_row_fields.push(serde_json::json!({
                "scenario_id": scenario_id,
                "missing_fields": missing_fields,
            }));
        }

        let raw_verdict = row
            .get("verdict")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let normalized_verdict_input = if trim_verdict {
            raw_verdict.trim().to_string()
        } else {
            raw_verdict
        };
        let verdict = if verdict_case == "upper_snake" {
            normalize_upper_snake(&normalized_verdict_input)
        } else {
            normalized_verdict_input
        };

        let raw_comparison_result = row
            .get("comparison_result")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let normalized_comparison_input = if trim_verdict {
            raw_comparison_result.trim().to_string()
        } else {
            raw_comparison_result
        };
        let comparison_result = if verdict_case == "upper_snake" {
            normalize_upper_snake(&normalized_comparison_input)
        } else {
            normalized_comparison_input
        };
        let criticality = taxonomy_map
            .get(&scenario_id)
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());
        if criticality == "unknown" {
            unknown_scenarios.push(scenario_id.clone());
        } else if !scenario_id.is_empty() {
            covered_scenario_ids.insert(scenario_id.clone());
        }
        if !allowed_verdicts.contains(verdict.as_str()) {
            invalid_verdict_rows.push(scenario_id.clone());
        }

        let is_out_of_tier_scenario = explicit_unsupported_required
            && taxonomy_ids.contains(&scenario_id)
            && !active_tier_supported_scenarios.contains(&scenario_id);
        if is_out_of_tier_scenario {
            explicit_unsupported_scenarios.insert(scenario_id.clone());
            if verdict != unsupported_verdict {
                unsupported_verdict_mismatches.push(format!(
                    "{scenario_id}:expected_verdict={unsupported_verdict}:actual_verdict={verdict}"
                ));
            }
            let missing_unsupported_fields = required_unsupported_metadata_fields
                .iter()
                .filter(|field| {
                    row.get(field.as_str())
                        .and_then(Value::as_str)
                        .map(str::trim)
                        .is_none_or(str::is_empty)
                })
                .cloned()
                .collect::<Vec<_>>();
            if !missing_unsupported_fields.is_empty() {
                unsupported_metadata_missing.push(serde_json::json!({
                    "scenario_id": scenario_id,
                    "missing_fields": missing_unsupported_fields,
                }));
            }
        }
        if required_package_interop_scenarios.contains(&scenario_id)
            && require_deterministic_fallback_diagnostics
        {
            let missing_diagnostic_fields = required_package_interop_diagnostic_fields
                .iter()
                .filter(|field| {
                    row.get(field.as_str())
                        .and_then(Value::as_str)
                        .map(str::trim)
                        .is_none_or(str::is_empty)
                })
                .cloned()
                .collect::<Vec<_>>();
            if !missing_diagnostic_fields.is_empty() {
                package_interop_missing_diagnostics.push(serde_json::json!({
                    "scenario_id": scenario_id,
                    "missing_fields": missing_diagnostic_fields,
                }));
            }
        }

        let lineage = row.get("lineage").and_then(Value::as_object);
        let missing_lineage_fields = required_lineage_fields
            .iter()
            .filter(|field| {
                lineage
                    .and_then(|lineage| lineage.get(field.as_str()))
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .is_none_or(str::is_empty)
            })
            .cloned()
            .collect::<Vec<_>>();
        if !missing_lineage_fields.is_empty() {
            missing_lineage.push(serde_json::json!({
                "scenario_id": scenario_id,
                "missing_fields": missing_lineage_fields,
            }));
        }

        if adjudication_rule == "comparison_result_must_match_verdict_after_normalization"
            && !verdict.is_empty()
            && !comparison_result.is_empty()
            && verdict != comparison_result
        {
            adjudication_mismatches.push(format!(
                "{scenario_id}:comparison_result={comparison_result}:verdict={verdict}"
            ));
        }

        if criticality == "high" && verdict == "INCOMPATIBLE" {
            incompatible_high.push(scenario_id.clone());
        }

        evaluated_rows.push(serde_json::json!({
            "scenario_id": scenario_id,
            "criticality": criticality,
            "verdict": verdict,
            "comparison_result": comparison_result,
            "lineage_missing_fields": missing_lineage_fields,
        }));
    }

    let missing_high_scenarios = taxonomy
        .iter()
        .filter_map(|row| {
            let scenario_id = row.get("scenario_id").and_then(Value::as_str)?;
            let criticality = row.get("criticality").and_then(Value::as_str)?;
            (criticality == "high" && !covered_scenario_ids.contains(scenario_id))
                .then(|| scenario_id.to_string())
        })
        .collect::<Vec<_>>();
    let mut implicit_unsupported_scenarios = if explicit_unsupported_required {
        taxonomy_ids
            .iter()
            .filter(|scenario_id| {
                !active_tier_supported_scenarios.contains(*scenario_id)
                    && !explicit_unsupported_scenarios.contains(*scenario_id)
            })
            .cloned()
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    implicit_unsupported_scenarios.sort();
    let mut missing_required_package_interop_scenarios = required_package_interop_scenarios
        .iter()
        .filter(|scenario_id| !covered_scenario_ids.contains(scenario_id.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    missing_required_package_interop_scenarios.sort();

    let mut blocking_reasons = Vec::new();
    if tiering_enabled && !active_tier_found {
        blocking_reasons.push(format!("active_tier_not_found={active_tier}"));
    }
    if !incompatible_high.is_empty() {
        blocking_reasons.push(format!(
            "incompatible_high_critical_scenarios={}",
            incompatible_high.join(",")
        ));
    }
    if !missing_lineage.is_empty() {
        blocking_reasons.push(format!("missing_lineage_rows={}", missing_lineage.len()));
    }
    if !missing_required_row_fields.is_empty() {
        blocking_reasons.push(format!(
            "missing_required_row_fields={}",
            missing_required_row_fields.len()
        ));
    }
    if !adjudication_mismatches.is_empty() {
        blocking_reasons.push(format!(
            "adjudication_mismatches={}",
            adjudication_mismatches.len()
        ));
    }
    if !missing_high_scenarios.is_empty() {
        blocking_reasons.push(format!(
            "missing_high_critical_scenarios={}",
            missing_high_scenarios.join(",")
        ));
    }
    if !unknown_scenarios.is_empty() {
        blocking_reasons.push(format!("unknown_scenarios={}", unknown_scenarios.join(",")));
    }
    if !invalid_verdict_rows.is_empty() {
        blocking_reasons.push(format!(
            "invalid_verdict_rows={}",
            invalid_verdict_rows.join(",")
        ));
    }
    if !implicit_unsupported_scenarios.is_empty() {
        blocking_reasons.push(format!(
            "implicit_unsupported_scenarios={}",
            implicit_unsupported_scenarios.join(",")
        ));
    }
    if !unsupported_verdict_mismatches.is_empty() {
        blocking_reasons.push(format!(
            "unsupported_verdict_mismatches={}",
            unsupported_verdict_mismatches.len()
        ));
    }
    if !unsupported_metadata_missing.is_empty() {
        blocking_reasons.push(format!(
            "unsupported_metadata_missing={}",
            unsupported_metadata_missing.len()
        ));
    }
    if package_interop_policy_hard_fail && !missing_required_package_interop_scenarios.is_empty() {
        blocking_reasons.push(format!(
            "missing_required_package_interop_scenarios={}",
            missing_required_package_interop_scenarios.join(",")
        ));
    }
    if package_interop_policy_hard_fail && !package_interop_missing_diagnostics.is_empty() {
        blocking_reasons.push(format!(
            "package_interop_missing_diagnostics={}",
            package_interop_missing_diagnostics.len()
        ));
    }

    serde_json::json!({
        "schema": "pi.frankennode.semantic_compatibility_matrix_report.v1",
        "summary": {
            "release_gate_status": if blocking_reasons.is_empty() { "ready" } else { "blocked" },
            "total_rows": rows.len(),
            "incompatible_high_critical_count": incompatible_high.len(),
            "missing_lineage_count": missing_lineage.len(),
            "missing_required_row_fields_count": missing_required_row_fields.len(),
            "adjudication_mismatch_count": adjudication_mismatches.len(),
            "implicit_unsupported_scenario_count": implicit_unsupported_scenarios.len(),
            "unsupported_verdict_mismatch_count": unsupported_verdict_mismatches.len(),
            "unsupported_metadata_missing_count": unsupported_metadata_missing.len(),
            "missing_required_package_interop_scenario_count": missing_required_package_interop_scenarios.len(),
            "missing_required_package_interop_scenarios": missing_required_package_interop_scenarios,
            "package_interop_missing_diagnostics_count": package_interop_missing_diagnostics.len(),
            "missing_high_critical_scenarios": missing_high_scenarios,
            "implicit_unsupported_scenarios": implicit_unsupported_scenarios,
            "blocking_reasons": blocking_reasons,
        },
        "rows": evaluated_rows,
    })
}

#[test]
fn semantic_compat_executable_harness_fails_closed_on_high_critical_incompatibility() {
    let contract = load_contract();
    let rows = vec![
        sample_row("SCN-module-resolution-esm-cjs", "INCOMPATIBLE"),
        sample_row("SCN-node-builtin-apis", "EXACT_PARITY"),
        sample_row("SCN-event-loop-io-ordering", "EXACT_PARITY"),
    ];

    let report = evaluate_executable_semantic_matrix(&contract, &rows);
    assert_eq!(
        report["summary"]["release_gate_status"],
        Value::String("blocked".to_string()),
        "high-critical INCOMPATIBLE row must block release gate"
    );
    assert!(
        report["summary"]["blocking_reasons"]
            .as_array()
            .is_some_and(|reasons| reasons
                .iter()
                .filter_map(Value::as_str)
                .any(|reason| reason.contains("incompatible_high_critical_scenarios"))),
        "blocking reasons must include incompatible_high_critical_scenarios"
    );
}

#[test]
fn semantic_compat_executable_harness_fails_closed_on_missing_lineage_fields() {
    let contract = load_contract();
    let mut incomplete = sample_row("SCN-module-resolution-esm-cjs", "EXACT_PARITY");
    let incomplete_lineage = incomplete
        .get_mut("lineage")
        .and_then(Value::as_object_mut)
        .expect("lineage must be an object");
    incomplete_lineage["run_id"] = Value::String(String::new());

    let rows = vec![
        incomplete,
        sample_row("SCN-node-builtin-apis", "EXACT_PARITY"),
        sample_row("SCN-event-loop-io-ordering", "EXACT_PARITY"),
    ];

    let report = evaluate_executable_semantic_matrix(&contract, &rows);
    assert_eq!(
        report["summary"]["release_gate_status"],
        Value::String("blocked".to_string()),
        "missing lineage fields must fail closed"
    );
    assert_eq!(
        report["summary"]["missing_lineage_count"]
            .as_u64()
            .unwrap_or_default(),
        1,
        "missing lineage rows count should surface the incomplete row"
    );
}

#[test]
fn semantic_compat_executable_harness_reports_ready_when_high_rows_exact_with_lineage() {
    let contract = load_contract();
    let rows = baseline_rows_with_explicit_unsupported();

    let report = evaluate_executable_semantic_matrix(&contract, &rows);
    assert_eq!(
        report["schema"],
        Value::String("pi.frankennode.semantic_compatibility_matrix_report.v1".to_string()),
        "executable harness must emit expected report schema"
    );
    assert_eq!(
        report["summary"]["release_gate_status"],
        Value::String("ready".to_string()),
        "all covered high-critical rows exact with full lineage should be ready"
    );
    assert_eq!(
        report["summary"]["total_rows"].as_u64().unwrap_or_default(),
        6,
        "summary total_rows should match evaluated rows"
    );
    assert_eq!(
        report["summary"]["implicit_unsupported_scenario_count"],
        Value::Number(0_u64.into()),
        "explicit unsupported rows should eliminate implicit unsupported omissions"
    );
}

#[test]
fn semantic_compat_executable_harness_normalizes_verdict_and_comparison_case() {
    let contract = load_contract();
    let rows = vec![
        sample_row(" SCN-module-resolution-esm-cjs ", " exact parity "),
        sample_row("SCN-node-builtin-apis", "ACCEPTABLE-SUPERSET"),
        sample_row("SCN-event-loop-io-ordering", "exact_parity"),
        sample_unsupported_row("SCN-package-interop-cjs-esm-npm"),
        serde_json::json!({
            "scenario_id": "SCN-tooling-and-package-workflows",
            "expected_baseline": "Node.js",
            "observed_runtime": "frankennode",
            "comparison_result": " unsupported ",
            "verdict": "unsupported",
            "unsupported_reason_code": "NOT_YET_IMPLEMENTED",
            "remediation_tag": "tier-upgrade-required",
            "fallback_diagnostics_signature": "diag-SCN-tooling-and-package-workflows",
            "fallback_reason_code": "NOT_YET_IMPLEMENTED",
            "fallback_remediation_tag": "interop-coverage-gap",
            "lineage": sample_lineage("SCN-tooling-and-package-workflows"),
        }),
        sample_unsupported_row("SCN-error-and-diagnostics-parity"),
    ];

    let report = evaluate_executable_semantic_matrix(&contract, &rows);
    assert_eq!(
        report["summary"]["release_gate_status"],
        Value::String("ready".to_string()),
        "trim+case normalization should preserve valid verdict rows"
    );
    assert_eq!(
        report["summary"]["adjudication_mismatch_count"]
            .as_u64()
            .unwrap_or_default(),
        0,
        "normalized comparison_result/verdict values should agree"
    );
    assert_eq!(
        report["summary"]["unsupported_verdict_mismatch_count"]
            .as_u64()
            .unwrap_or_default(),
        0,
        "normalized unsupported verdict rows should not register mismatches"
    );
}

#[test]
fn semantic_compat_executable_harness_fails_closed_on_implicit_unsupported_omissions() {
    let contract = load_contract();
    let rows = vec![
        sample_row("SCN-module-resolution-esm-cjs", "EXACT_PARITY"),
        sample_row("SCN-node-builtin-apis", "EXACT_PARITY"),
        sample_row("SCN-event-loop-io-ordering", "EXACT_PARITY"),
    ];

    let report = evaluate_executable_semantic_matrix(&contract, &rows);
    assert_eq!(
        report["summary"]["release_gate_status"],
        Value::String("blocked".to_string()),
        "missing explicit unsupported rows must fail closed"
    );
    assert_eq!(
        report["summary"]["implicit_unsupported_scenario_count"]
            .as_u64()
            .unwrap_or_default(),
        3,
        "three out-of-tier scenarios should be reported as implicit unsupported omissions"
    );
    assert!(
        report["summary"]["blocking_reasons"]
            .as_array()
            .is_some_and(|reasons| reasons
                .iter()
                .filter_map(Value::as_str)
                .any(|reason| reason.contains("implicit_unsupported_scenarios="))),
        "blocking reasons must include implicit_unsupported_scenarios"
    );
}

#[test]
fn semantic_compat_executable_harness_fails_closed_on_missing_required_package_interop_scenario() {
    let contract = load_contract();
    let rows = baseline_rows_with_explicit_unsupported()
        .into_iter()
        .filter(|row| {
            row.get("scenario_id").and_then(Value::as_str)
                != Some("SCN-package-interop-cjs-esm-npm")
        })
        .collect::<Vec<_>>();

    let report = evaluate_executable_semantic_matrix(&contract, &rows);
    assert_eq!(
        report["summary"]["release_gate_status"],
        Value::String("blocked".to_string()),
        "missing required package interop scenarios must fail closed"
    );
    assert_eq!(
        report["summary"]["missing_required_package_interop_scenario_count"]
            .as_u64()
            .unwrap_or_default(),
        1,
        "required package interop scenario omissions must be counted"
    );
    assert!(
        report["summary"]["blocking_reasons"]
            .as_array()
            .is_some_and(|reasons| reasons
                .iter()
                .filter_map(Value::as_str)
                .any(|reason| reason.contains("missing_required_package_interop_scenarios="))),
        "blocking reasons must include missing_required_package_interop_scenarios"
    );
}

#[test]
fn semantic_compat_executable_harness_fails_closed_on_missing_package_interop_diagnostics() {
    let contract = load_contract();
    let mut rows = baseline_rows_with_explicit_unsupported();
    let package_interop = rows
        .iter_mut()
        .find(|row| {
            row.get("scenario_id").and_then(Value::as_str)
                == Some("SCN-package-interop-cjs-esm-npm")
        })
        .expect("package interop row must exist");
    package_interop["fallback_diagnostics_signature"] = Value::String(String::new());

    let report = evaluate_executable_semantic_matrix(&contract, &rows);
    assert_eq!(
        report["summary"]["release_gate_status"],
        Value::String("blocked".to_string()),
        "missing package interop diagnostics must fail closed"
    );
    assert_eq!(
        report["summary"]["package_interop_missing_diagnostics_count"]
            .as_u64()
            .unwrap_or_default(),
        1,
        "missing package interop diagnostics count should surface incomplete rows"
    );
    assert!(
        report["summary"]["blocking_reasons"]
            .as_array()
            .is_some_and(|reasons| reasons
                .iter()
                .filter_map(Value::as_str)
                .any(|reason| reason.contains("package_interop_missing_diagnostics="))),
        "blocking reasons must include package_interop_missing_diagnostics"
    );
}

#[test]
fn semantic_compat_executable_harness_fails_closed_on_unsupported_rows_missing_metadata() {
    let contract = load_contract();
    let mut rows = baseline_rows_with_explicit_unsupported();
    let broken = rows
        .iter_mut()
        .find(|row| {
            row.get("scenario_id").and_then(Value::as_str)
                == Some("SCN-tooling-and-package-workflows")
        })
        .expect("tooling scenario row must exist");
    broken["remediation_tag"] = Value::String(String::new());

    let report = evaluate_executable_semantic_matrix(&contract, &rows);
    assert_eq!(
        report["summary"]["release_gate_status"],
        Value::String("blocked".to_string()),
        "unsupported rows missing required metadata must fail closed"
    );
    assert_eq!(
        report["summary"]["unsupported_metadata_missing_count"]
            .as_u64()
            .unwrap_or_default(),
        1,
        "unsupported metadata missing count should capture incomplete unsupported rows"
    );
}

#[test]
fn semantic_compat_executable_harness_fails_closed_on_missing_required_row_fields() {
    let contract = load_contract();
    let mut missing_expected = sample_row("SCN-module-resolution-esm-cjs", "EXACT_PARITY");
    missing_expected["expected_baseline"] = Value::String(String::new());
    let rows = vec![
        missing_expected,
        sample_row("SCN-node-builtin-apis", "EXACT_PARITY"),
        sample_row("SCN-event-loop-io-ordering", "EXACT_PARITY"),
    ];

    let report = evaluate_executable_semantic_matrix(&contract, &rows);
    assert_eq!(
        report["summary"]["release_gate_status"],
        Value::String("blocked".to_string()),
        "rows missing required executable schema fields must fail closed"
    );
    assert_eq!(
        report["summary"]["missing_required_row_fields_count"]
            .as_u64()
            .unwrap_or_default(),
        1,
        "missing required row fields count should surface schema violations"
    );
}

#[test]
fn semantic_compat_executable_harness_fails_closed_on_adjudication_mismatch() {
    let contract = load_contract();
    let mut mismatch = sample_row("SCN-module-resolution-esm-cjs", "EXACT_PARITY");
    mismatch["comparison_result"] = Value::String("INCOMPATIBLE".to_string());
    let rows = vec![
        mismatch,
        sample_row("SCN-node-builtin-apis", "EXACT_PARITY"),
        sample_row("SCN-event-loop-io-ordering", "EXACT_PARITY"),
    ];

    let report = evaluate_executable_semantic_matrix(&contract, &rows);
    assert_eq!(
        report["summary"]["release_gate_status"],
        Value::String("blocked".to_string()),
        "comparison_result/verdict mismatches must block release claims"
    );
    assert_eq!(
        report["summary"]["adjudication_mismatch_count"]
            .as_u64()
            .unwrap_or_default(),
        1,
        "adjudication mismatch count should capture divergent verdict rows"
    );
}
