use serde_json::Value;
use std::collections::HashSet;
use std::path::PathBuf;

const CONTRACT_PATH: &str = "docs/franken-node-executable-conformance-harness-contract.json";

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

#[test]
fn conformance_harness_contract_exists_and_is_valid_json() {
    let path = repo_root().join(CONTRACT_PATH);
    assert!(
        path.is_file(),
        "missing executable conformance-harness contract artifact: {}",
        path.display()
    );
    let _ = load_contract();
}

#[test]
fn conformance_harness_contract_has_expected_schema_version_and_bead_linkage() {
    let contract = load_contract();

    assert_eq!(
        contract["schema"],
        Value::String("pi.frankennode.executable_conformance_harness_contract.v1".to_string()),
        "schema mismatch for executable conformance-harness contract"
    );

    let version = contract["contract_version"]
        .as_str()
        .expect("contract_version must be a string");
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
        Value::String("bd-1ec8w".to_string()),
        "support bead linkage must target bd-1ec8w"
    );
}

#[test]
fn conformance_harness_execution_policy_is_fail_closed_and_complete() {
    let contract = load_contract();
    let policy = &contract["harness_execution_policy"];

    assert_eq!(
        policy["execution_mode"].as_str(),
        Some("fail_closed"),
        "execution_mode must be fail_closed"
    );

    let runtime_targets = policy["required_runtime_targets"]
        .as_array()
        .expect("required_runtime_targets must be an array");
    let runtime_set: HashSet<&str> = runtime_targets.iter().filter_map(Value::as_str).collect();
    for required in ["franken-node", "node", "bun"] {
        assert!(
            runtime_set.contains(required),
            "required_runtime_targets missing: {required}"
        );
    }

    let pairings = policy["required_oracle_pairings"]
        .as_array()
        .expect("required_oracle_pairings must be an array");
    let pairing_set: HashSet<&str> = pairings.iter().filter_map(Value::as_str).collect();
    for required in ["franken-node-vs-node", "franken-node-vs-bun"] {
        assert!(
            pairing_set.contains(required),
            "required_oracle_pairings missing: {required}"
        );
    }

    let required_fields = policy["required_row_fields"]
        .as_array()
        .expect("required_row_fields must be an array");
    let required_field_set: HashSet<&str> =
        required_fields.iter().filter_map(Value::as_str).collect();
    for required in [
        "run_id",
        "correlation_id",
        "scenario_id",
        "fixture_id",
        "runtime_target",
        "oracle_target",
        "verdict",
        "artifact_path",
        "captured_at_utc",
    ] {
        assert!(
            required_field_set.contains(required),
            "required_row_fields missing: {required}"
        );
    }

    let allowed_verdicts = policy["allowed_row_verdicts"]
        .as_array()
        .expect("allowed_row_verdicts must be an array");
    let verdict_set: HashSet<&str> = allowed_verdicts.iter().filter_map(Value::as_str).collect();
    for required in [
        "EXACT_PARITY",
        "ACCEPTABLE_SUPERSET",
        "PARTIAL_PARITY",
        "INCOMPATIBLE",
        "ERROR",
    ] {
        assert!(
            verdict_set.contains(required),
            "allowed_row_verdicts missing required value: {required}"
        );
    }
}

#[test]
fn conformance_harness_verdict_derivation_policy_blocks_release_on_bad_evidence() {
    let contract = load_contract();
    let derivation = &contract["verdict_derivation_policy"];

    let summary_fields = derivation["required_summary_fields"]
        .as_array()
        .expect("required_summary_fields must be an array");
    let summary_field_set: HashSet<&str> =
        summary_fields.iter().filter_map(Value::as_str).collect();
    for required in [
        "run_id",
        "correlation_id",
        "summary_verdict",
        "high_criticality_incompatible_count",
        "lineage_missing_count",
        "error_row_count",
        "generated_at_utc",
    ] {
        assert!(
            summary_field_set.contains(required),
            "required_summary_fields missing: {required}"
        );
    }

    let rules = &derivation["release_block_rules"];
    for required_rule in [
        "high_criticality_incompatible_blocks_release",
        "lineage_missing_blocks_release",
        "error_rows_block_release",
        "missing_or_stale_summary_blocks_release",
    ] {
        assert_eq!(
            rules[required_rule].as_bool(),
            Some(true),
            "release_block_rules.{required_rule} must be true"
        );
    }
}

#[test]
fn conformance_harness_contract_declares_release_blockers_and_dependency_links() {
    let contract = load_contract();

    let blockers = contract["release_blockers"]
        .as_array()
        .expect("release_blockers must be an array");
    assert!(
        blockers.len() >= 4,
        "release_blockers must include multiple fail-closed conditions"
    );

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
            "blocked_beads missing dependency: {required}"
        );
    }

    let integration_contracts = contract["downstream_dependencies"]["integration_contracts"]
        .as_array()
        .expect("downstream_dependencies.integration_contracts must be an array");
    let integration_set: HashSet<&str> = integration_contracts
        .iter()
        .filter_map(Value::as_str)
        .collect();
    for required in [
        "docs/franken-node-semantic-compatibility-matrix-contract.json",
        "docs/franken-node-claim-gating-contract.json",
    ] {
        assert!(
            integration_set.contains(required),
            "integration_contracts missing linkage: {required}"
        );
    }
}
