use serde_json::Value;
use std::collections::HashSet;
use std::path::PathBuf;

const CONTRACT_PATH: &str = "docs/franken-node-deterministic-replay-contract.json";
const EXPECTED_SCHEMA: &str = "pi.frankennode.deterministic_replay_contract.v1";

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
fn replay_contract_exists_and_has_correct_schema() {
    let contract = load_contract();
    assert_eq!(
        contract["schema"].as_str().unwrap(),
        EXPECTED_SCHEMA,
        "schema must be {EXPECTED_SCHEMA}"
    );
    assert_eq!(
        contract["bead_id"].as_str().unwrap(),
        "bd-3ar8v.7.9",
        "bead_id must reference bd-3ar8v.7.9"
    );
    assert_eq!(
        contract["status"].as_str().unwrap(),
        "active_blocking_policy",
        "contract must be active_blocking_policy"
    );
}

#[test]
fn replay_contract_capture_pipeline_has_required_modes() {
    let contract = load_contract();
    let modes = contract["capture_pipeline"]["recording_modes"]
        .as_array()
        .expect("recording_modes must be an array");

    assert!(
        modes.len() >= 3,
        "must define at least 3 recording modes (full, lightweight, off), got {}",
        modes.len()
    );

    let mode_ids: HashSet<&str> = modes.iter().filter_map(|m| m["mode_id"].as_str()).collect();

    for required in ["full_deterministic", "lightweight_trace", "off"] {
        assert!(
            mode_ids.contains(required),
            "recording_modes must include {required}"
        );
    }

    for mode in modes {
        let mid = mode["mode_id"].as_str().unwrap_or("unknown");
        assert!(
            mode["description"].as_str().is_some_and(|d| !d.is_empty()),
            "recording mode {mid} must have non-empty description"
        );
        if mid != "off" {
            assert!(
                mode["overhead_budget"].as_str().is_some(),
                "recording mode {mid} must declare overhead_budget"
            );
        }
    }
}

#[test]
fn replay_contract_captured_sources_cover_non_deterministic_inputs() {
    let contract = load_contract();
    let sources = contract["capture_pipeline"]["captured_sources"]
        .as_array()
        .expect("captured_sources must be an array");

    let source_set: HashSet<&str> = sources.iter().filter_map(Value::as_str).collect();

    for required in [
        "timer_firings",
        "io_poll_results",
        "random_number_generation",
        "wall_clock_timestamps",
    ] {
        assert!(
            source_set.contains(required),
            "captured_sources must include {required}"
        );
    }
}

#[test]
fn replay_contract_capture_invariants_prevent_behavioral_alteration() {
    let contract = load_contract();
    let invariants = contract["capture_pipeline"]["invariants"]
        .as_array()
        .expect("capture_pipeline.invariants must be an array");

    assert!(
        invariants.len() >= 3,
        "must define at least 3 capture invariants, got {}",
        invariants.len()
    );

    let texts: Vec<&str> = invariants.iter().filter_map(Value::as_str).collect();
    assert!(
        texts
            .iter()
            .any(|t| t.contains("not alter") || t.contains("does not alter")),
        "capture invariants must guarantee recording does not alter behavior"
    );
    assert!(
        texts
            .iter()
            .any(|t| t.contains("sequence number") || t.contains("ordering")),
        "capture invariants must mention event ordering/sequence numbers"
    );
}

#[test]
fn replay_contract_replay_engine_has_strict_and_best_effort_modes() {
    let contract = load_contract();
    let modes = contract["replay_engine"]["replay_modes"]
        .as_array()
        .expect("replay_modes must be an array");

    let mode_ids: HashSet<&str> = modes.iter().filter_map(|m| m["mode_id"].as_str()).collect();

    assert!(
        mode_ids.contains("strict_deterministic"),
        "replay_modes must include strict_deterministic"
    );
    assert!(
        mode_ids.contains("best_effort"),
        "replay_modes must include best_effort"
    );

    for mode in modes {
        let mid = mode["mode_id"].as_str().unwrap_or("unknown");
        assert!(
            mode["divergence_policy"].as_str().is_some(),
            "replay mode {mid} must declare divergence_policy"
        );
    }
}

#[test]
fn replay_contract_strict_mode_aborts_on_divergence() {
    let contract = load_contract();
    let modes = contract["replay_engine"]["replay_modes"]
        .as_array()
        .expect("replay_modes must be an array");

    for mode in modes {
        if mode["mode_id"].as_str() == Some("strict_deterministic") {
            let policy = mode["divergence_policy"].as_str().unwrap();
            assert!(
                policy.contains("abort"),
                "strict_deterministic mode must abort on divergence, got: {policy}"
            );
        }
    }
}

#[test]
fn replay_contract_fault_dossier_has_required_sections() {
    let contract = load_contract();
    let sections = contract["fault_dossier"]["required_sections"]
        .as_array()
        .expect("required_sections must be an array");

    assert!(
        sections.len() >= 4,
        "must define at least 4 dossier sections, got {}",
        sections.len()
    );

    let section_ids: HashSet<&str> = sections
        .iter()
        .filter_map(|s| s["section_id"].as_str())
        .collect();

    for required in [
        "execution_timeline",
        "divergence_report",
        "resource_snapshot",
        "causal_chain",
    ] {
        assert!(
            section_ids.contains(required),
            "fault_dossier.required_sections must include {required}"
        );
    }

    for section in sections {
        let sid = section["section_id"].as_str().unwrap_or("unknown");
        assert!(
            section["description"]
                .as_str()
                .is_some_and(|d| !d.is_empty()),
            "dossier section {sid} must have non-empty description"
        );
    }
}

#[test]
fn replay_contract_fault_dossier_is_self_contained() {
    let contract = load_contract();
    let invariants = contract["fault_dossier"]["invariants"]
        .as_array()
        .expect("fault_dossier.invariants must be an array");

    let texts: Vec<&str> = invariants.iter().filter_map(Value::as_str).collect();
    assert!(
        texts.iter().any(|t| t.contains("self-contained")),
        "fault dossier invariants must declare self-containedness"
    );
    assert!(
        texts
            .iter()
            .any(|t| t.contains("bounded") || t.contains("size")),
        "fault dossier invariants must declare size bounds"
    );
}

#[test]
fn replay_contract_fault_injection_only_in_non_production_modes() {
    let contract = load_contract();
    let invariants = contract["fault_injection"]["invariants"]
        .as_array()
        .expect("fault_injection.invariants must be an array");

    let texts: Vec<&str> = invariants.iter().filter_map(Value::as_str).collect();
    assert!(
        texts
            .iter()
            .any(|t| t.contains("never in production") || t.contains("off mode")),
        "fault injection invariants must prohibit production-mode injection"
    );
}

#[test]
fn replay_contract_injectable_faults_cover_required_types() {
    let contract = load_contract();
    let faults = contract["fault_injection"]["injectable_faults"]
        .as_array()
        .expect("injectable_faults must be an array");

    assert!(
        faults.len() >= 3,
        "must define at least 3 injectable fault types, got {}",
        faults.len()
    );

    let fault_ids: HashSet<&str> = faults
        .iter()
        .filter_map(|f| f["fault_id"].as_str())
        .collect();

    for required in ["io_delay", "io_error", "timer_jitter"] {
        assert!(
            fault_ids.contains(required),
            "injectable_faults must include {required}"
        );
    }
}

#[test]
fn replay_contract_release_blockers_are_defined() {
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
        blocker_texts.iter().any(|b| b.contains("behavior")),
        "release_blockers must mention behavioral alteration"
    );
    assert!(
        blocker_texts
            .iter()
            .any(|b| b.contains("divergence") || b.contains("ordering")),
        "release_blockers must mention replay divergence"
    );
}

#[test]
fn replay_contract_downstream_dependencies_reference_blocked_beads() {
    let contract = load_contract();
    let blocked: HashSet<&str> = contract["downstream_dependencies"]["blocked_beads"]
        .as_array()
        .expect("blocked_beads must be an array")
        .iter()
        .filter_map(Value::as_str)
        .collect();

    for required in ["bd-3ar8v.7.10", "bd-3ar8v.7.11", "bd-3ar8v.7.15"] {
        assert!(
            blocked.contains(required),
            "downstream_dependencies.blocked_beads must include {required}"
        );
    }
}
