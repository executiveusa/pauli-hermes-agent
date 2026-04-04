use serde_json::Value;
use std::collections::HashSet;
use std::path::PathBuf;

const CONTRACT_PATH: &str = "docs/franken-node-security-sandbox-contract.json";
const EXPECTED_SCHEMA: &str = "pi.frankennode.security_sandbox_contract.v1";

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn load_contract() -> Value {
    let path = repo_root().join(CONTRACT_PATH);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {CONTRACT_PATH}: {e}"));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("invalid JSON in {CONTRACT_PATH}: {e}"))
}

#[test]
fn security_contract_exists_and_has_correct_schema() {
    let contract = load_contract();
    assert_eq!(
        contract["schema"].as_str().unwrap(),
        EXPECTED_SCHEMA,
        "schema must be {EXPECTED_SCHEMA}"
    );
    assert_eq!(
        contract["bead_id"].as_str().unwrap(),
        "bd-3ar8v.7.8",
        "bead_id must reference bd-3ar8v.7.8"
    );
    assert_eq!(
        contract["status"].as_str().unwrap(),
        "active_blocking_policy",
        "contract must be active_blocking_policy"
    );
}

#[test]
fn security_contract_capability_model_is_deny_by_default() {
    let contract = load_contract();
    let cap_model = &contract["capability_model"];

    assert_eq!(
        cap_model["policy_enforcement_mode"].as_str().unwrap(),
        "deny_by_default",
        "capability model must use deny_by_default enforcement"
    );

    let classes = cap_model["capability_classes"]
        .as_array()
        .expect("capability_classes must be an array");
    assert!(
        classes.len() >= 4,
        "must define at least 4 capability classes, got {}",
        classes.len()
    );

    let mut class_ids = HashSet::new();
    for class in classes {
        let cap_id = class["capability_id"]
            .as_str()
            .expect("capability_id must be a string");
        assert!(
            class_ids.insert(cap_id.to_string()),
            "duplicate capability_id: {cap_id}"
        );
        assert!(
            class["description"].as_str().is_some(),
            "capability {cap_id} missing description"
        );
        assert!(
            class["default_grant"].is_boolean(),
            "capability {cap_id} missing default_grant boolean"
        );
        assert!(
            class["escalation_risk"].as_str().is_some(),
            "capability {cap_id} missing escalation_risk"
        );
    }

    for required in ["fs_read", "fs_write", "network_outbound", "process_spawn"] {
        assert!(
            class_ids.contains(required),
            "capability_classes must include {required}"
        );
    }

    let invariants = cap_model["invariants"]
        .as_array()
        .expect("invariants must be an array");
    assert!(
        invariants.len() >= 3,
        "must define at least 3 capability invariants"
    );
}

#[test]
fn security_contract_sandbox_boundaries_are_enumerated() {
    let contract = load_contract();
    let boundaries = contract["sandbox_boundaries"]
        .as_array()
        .expect("sandbox_boundaries must be an array");

    assert!(
        boundaries.len() >= 4,
        "must define at least 4 sandbox boundaries, got {}",
        boundaries.len()
    );

    let mut boundary_ids = HashSet::new();
    for boundary in boundaries {
        let bid = boundary["boundary_id"]
            .as_str()
            .expect("boundary_id must be a string");
        assert!(
            boundary_ids.insert(bid.to_string()),
            "duplicate boundary_id: {bid}"
        );
        assert!(
            boundary["enforcement"].as_str().is_some(),
            "boundary {bid} missing enforcement mechanism"
        );
        assert!(
            boundary["violation_response"].as_str().is_some(),
            "boundary {bid} missing violation_response"
        );
    }

    for required in [
        "js_runtime_isolation",
        "filesystem_scope",
        "network_allowlist",
        "hostcall_abi_boundary",
    ] {
        assert!(
            boundary_ids.contains(required),
            "sandbox_boundaries must include {required}"
        );
    }
}

#[test]
fn security_contract_no_sandbox_violation_is_silent() {
    let contract = load_contract();
    let boundaries = contract["sandbox_boundaries"]
        .as_array()
        .expect("sandbox_boundaries must be an array");

    for boundary in boundaries {
        let bid = boundary["boundary_id"].as_str().unwrap_or("unknown");
        let response = boundary["violation_response"]
            .as_str()
            .expect("violation_response must be a string");
        assert!(
            response != "silent" && response != "ignore" && response != "log_only",
            "boundary {bid} has silent/permissive violation_response: {response}; \
             all violations must produce hard_abort, deny_with_diagnostic, or compilation_error"
        );
    }
}

#[test]
fn security_contract_policy_decision_logging_has_required_fields() {
    let contract = load_contract();
    let logging = &contract["policy_decision_logging"];

    let fields: HashSet<&str> = logging["required_fields"]
        .as_array()
        .expect("required_fields must be an array")
        .iter()
        .filter_map(Value::as_str)
        .collect();

    for required in [
        "run_id",
        "decision_id",
        "capability_id",
        "grant_or_deny",
        "policy_snapshot_id",
        "timestamp_utc",
    ] {
        assert!(
            fields.contains(required),
            "policy_decision_logging.required_fields must include {required}"
        );
    }

    let event_types: HashSet<&str> = logging["required_event_types"]
        .as_array()
        .expect("required_event_types must be an array")
        .iter()
        .filter_map(Value::as_str)
        .collect();

    for required in [
        "capability_granted",
        "capability_denied",
        "sandbox_violation_attempt",
        "escalation_blocked",
    ] {
        assert!(
            event_types.contains(required),
            "required_event_types must include {required}"
        );
    }
}

#[test]
fn security_contract_threat_model_surfaces_have_mitigations() {
    let contract = load_contract();
    let surfaces = contract["threat_model_surfaces"]
        .as_array()
        .expect("threat_model_surfaces must be an array");

    assert!(
        surfaces.len() >= 4,
        "must define at least 4 threat model surfaces, got {}",
        surfaces.len()
    );

    let mut surface_ids = HashSet::new();
    for surface in surfaces {
        let sid = surface["surface_id"]
            .as_str()
            .expect("surface_id must be a string");
        assert!(
            surface_ids.insert(sid.to_string()),
            "duplicate surface_id: {sid}"
        );
        assert!(
            surface["mitigation"]
                .as_str()
                .is_some_and(|m| !m.is_empty()),
            "threat surface {sid} must have non-empty mitigation"
        );
        assert!(
            surface["severity"].as_str().is_some(),
            "threat surface {sid} must have severity"
        );
    }

    for required in [
        "TM-privilege-escalation",
        "TM-sandbox-escape",
        "TM-extension-cross-contamination",
    ] {
        assert!(
            surface_ids.contains(required),
            "threat_model_surfaces must include {required}"
        );
    }
}

#[test]
fn security_contract_critical_threats_have_critical_severity() {
    let contract = load_contract();
    let surfaces = contract["threat_model_surfaces"]
        .as_array()
        .expect("threat_model_surfaces must be an array");

    for surface in surfaces {
        let sid = surface["surface_id"].as_str().unwrap_or("unknown");
        if sid == "TM-privilege-escalation" || sid == "TM-sandbox-escape" {
            assert_eq!(
                surface["severity"].as_str().unwrap(),
                "critical",
                "threat surface {sid} must have critical severity"
            );
        }
    }
}

#[test]
fn security_contract_release_blockers_are_defined() {
    let contract = load_contract();
    let blockers = contract["release_blockers"]
        .as_array()
        .expect("release_blockers must be an array");

    assert!(
        blockers.len() >= 3,
        "must define at least 3 release blockers"
    );

    let blocker_texts: Vec<&str> = blockers.iter().filter_map(Value::as_str).collect();
    assert!(
        blocker_texts.iter().any(|b| b.contains("deny-by-default")),
        "release_blockers must mention deny-by-default enforcement"
    );
    assert!(
        blocker_texts.iter().any(|b| b.contains("silent")),
        "release_blockers must prohibit silent violation responses"
    );
}

#[test]
fn security_contract_downstream_dependencies_reference_blocked_beads() {
    let contract = load_contract();
    let deps = &contract["downstream_dependencies"];

    let blocked: HashSet<&str> = deps["blocked_beads"]
        .as_array()
        .expect("blocked_beads must be an array")
        .iter()
        .filter_map(Value::as_str)
        .collect();

    for required in ["bd-3ar8v.7.10", "bd-3ar8v.7.11"] {
        assert!(
            blocked.contains(required),
            "downstream_dependencies.blocked_beads must include {required}"
        );
    }
}

#[test]
fn security_contract_process_spawn_is_not_default_granted() {
    let contract = load_contract();
    let classes = contract["capability_model"]["capability_classes"]
        .as_array()
        .expect("capability_classes must be an array");

    for class in classes {
        let cap_id = class["capability_id"].as_str().unwrap_or("unknown");
        if cap_id == "process_spawn" {
            assert_eq!(
                class["default_grant"].as_bool(),
                Some(false),
                "process_spawn must never be default-granted (critical escalation risk)"
            );
            assert_eq!(
                class["escalation_risk"].as_str(),
                Some("critical"),
                "process_spawn must be classified as critical escalation risk"
            );
        }
    }
}
