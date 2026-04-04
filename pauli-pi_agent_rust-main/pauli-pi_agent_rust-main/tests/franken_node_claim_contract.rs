use serde_json::Value;
use std::collections::HashSet;
use std::path::PathBuf;

const CONTRACT_PATH: &str = "docs/franken-node-claim-gating-contract.json";
const EXPECTED_TIER_ORDER: [&str; 3] = [
    "TIER-1-EXTENSION-HOST-PARITY",
    "TIER-2-TARGETED-RUNTIME-PARITY",
    "TIER-3-FULL-NODE-BUN-REPLACEMENT",
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

#[test]
fn franken_node_claim_contract_exists_and_is_valid_json() {
    let path = repo_root().join(CONTRACT_PATH);
    assert!(
        path.is_file(),
        "missing franken-node claim contract artifact: {}",
        path.display()
    );
    let _ = load_contract();
}

#[test]
fn franken_node_claim_contract_has_expected_schema_and_version() {
    let contract = load_contract();
    assert_eq!(
        contract["schema"],
        Value::String("pi.frankennode.claim_gating_contract.v1".to_string()),
        "unexpected schema identifier for franken-node claim contract"
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
        Value::String("bd-3ar8v.7.1".to_string()),
        "bead linkage must point to bd-3ar8v.7.1"
    );
    assert_eq!(
        contract["support_bead_id"],
        Value::String("bd-3ar8v.7.1.1".to_string()),
        "support bead linkage must point to bd-3ar8v.7.1.1"
    );
}

#[test]
fn franken_node_claim_contract_declares_expected_tier_order() {
    let contract = load_contract();
    let tiers = contract["claim_tiers"]
        .as_array()
        .expect("claim_tiers must be an array");
    assert_eq!(
        tiers.len(),
        EXPECTED_TIER_ORDER.len(),
        "claim_tiers must include exactly {} tiers",
        EXPECTED_TIER_ORDER.len()
    );

    let observed: Vec<&str> = tiers
        .iter()
        .map(|tier| {
            tier["tier_id"]
                .as_str()
                .expect("tier_id must be present for every claim tier")
        })
        .collect();
    assert_eq!(
        observed, EXPECTED_TIER_ORDER,
        "claim tier order must remain strict to prevent over-claim drift"
    );
}

#[test]
fn franken_node_claim_contract_tiers_are_evidence_backed_and_fail_closed() {
    let contract = load_contract();
    let tiers = contract["claim_tiers"]
        .as_array()
        .expect("claim_tiers must be an array");

    for tier in tiers {
        let tier_id = tier["tier_id"]
            .as_str()
            .expect("tier_id must be present for every claim tier");

        let required_evidence = tier["required_evidence"]
            .as_array()
            .unwrap_or_else(|| panic!("{tier_id}: required_evidence must be an array"));
        assert!(
            !required_evidence.is_empty(),
            "{tier_id}: required_evidence must not be empty"
        );

        let allowed = tier["allowed_claim_language"]
            .as_array()
            .unwrap_or_else(|| panic!("{tier_id}: allowed_claim_language must be an array"));
        assert!(
            !allowed.is_empty(),
            "{tier_id}: allowed_claim_language must not be empty"
        );

        let forbidden = tier["forbidden_claim_language"]
            .as_array()
            .unwrap_or_else(|| panic!("{tier_id}: forbidden_claim_language must be an array"));
        assert!(
            !forbidden.is_empty(),
            "{tier_id}: forbidden_claim_language must not be empty"
        );
    }
}

#[test]
fn franken_node_claim_contract_strict_replacement_gate_is_hard_fail() {
    let contract = load_contract();

    let gate_mode = contract["claim_gate_policy"]["release_claim_gate_mode"]
        .as_str()
        .expect("claim_gate_policy.release_claim_gate_mode must be present");
    assert_eq!(
        gate_mode, "hard_fail_if_unmet",
        "strict claim gate mode must be fail-closed"
    );

    let strict_verdict =
        contract["claim_gate_policy"]["strict_replacement_requires"]["overall_verdict"]
            .as_str()
            .expect("strict_replacement_requires.overall_verdict must be present");
    assert_eq!(
        strict_verdict, "CERTIFIED",
        "strict replacement language must require CERTIFIED verdict"
    );

    let artifacts =
        contract["claim_gate_policy"]["strict_replacement_requires"]["required_artifacts"]
            .as_array()
            .expect("strict_replacement_requires.required_artifacts must be an array");
    let artifact_set: HashSet<&str> = artifacts.iter().filter_map(Value::as_str).collect();
    for required in [
        "tests/full_suite_gate/franken_node_claim_verdict.json",
        "tests/full_suite_gate/practical_finish_checkpoint.json",
    ] {
        assert!(
            artifact_set.contains(required),
            "strict replacement gate must require artifact: {required}"
        );
    }

    let blockers = contract["claim_gate_policy"]["overclaim_blockers"]
        .as_array()
        .expect("claim_gate_policy.overclaim_blockers must be an array");
    let blocker_set: HashSet<&str> = blockers.iter().filter_map(Value::as_str).collect();
    for required in [
        "missing_required_evidence",
        "open_critical_gap",
        "missing_or_stale_verdict_artifact",
        "forbidden_claim_phrase_detected",
    ] {
        assert!(
            blocker_set.contains(required),
            "overclaim_blockers must include: {required}"
        );
    }
}

#[test]
fn franken_node_claim_contract_declares_forbidden_language_and_reintegration_linkage() {
    let contract = load_contract();
    let forbidden = contract["forbidden_claim_patterns"]
        .as_array()
        .expect("forbidden_claim_patterns must be an array");
    assert!(
        forbidden.len() >= 2,
        "forbidden_claim_patterns must include at least two entries"
    );
    let forbidden_joined = forbidden
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase();
    assert!(
        forbidden_joined.contains("drop-in") || forbidden_joined.contains("drop in"),
        "forbidden claim patterns must block strict drop-in phrasing"
    );
    assert!(
        forbidden_joined.contains("node")
            && forbidden_joined.contains("bun")
            && forbidden_joined.contains("replacement"),
        "forbidden claim patterns must explicitly cover Node/Bun replacement over-claims"
    );

    let reintegration = &contract["future_crate_reintegration_obligations"];
    assert_eq!(
        reintegration["required_bead"],
        Value::String("bd-3ar8v.7.13".to_string()),
        "future crate reintegration obligations must be linked to bd-3ar8v.7.13"
    );
    let obligations = reintegration["obligations"]
        .as_array()
        .expect("future_crate_reintegration_obligations.obligations must be an array");
    assert!(
        !obligations.is_empty(),
        "future crate reintegration obligations must not be empty"
    );
}

#[test]
fn franken_node_claim_contract_declares_required_structured_logging_fields() {
    let contract = load_contract();
    let fields = contract["structured_logging_contract"]["required_fields"]
        .as_array()
        .expect("structured_logging_contract.required_fields must be an array");
    let field_set: HashSet<&str> = fields.iter().filter_map(Value::as_str).collect();

    for required in [
        "run_id",
        "tier_id",
        "decision",
        "blocking_reasons",
        "evidence_refs",
        "timestamp_utc",
    ] {
        assert!(
            field_set.contains(required),
            "structured logging required_fields must include: {required}"
        );
    }
}
