use serde_json::Value;
use std::collections::HashSet;
use std::path::PathBuf;

const CONTRACT_PATH: &str = "docs/franken-node-event-loop-io-parity-contract.json";
const EXPECTED_SCHEMA: &str = "pi.frankennode.event_loop_io_parity_contract.v1";

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
fn event_loop_contract_exists_and_has_correct_schema() {
    let contract = load_contract();
    assert_eq!(
        contract["schema"].as_str().unwrap(),
        EXPECTED_SCHEMA,
        "schema must be {EXPECTED_SCHEMA}"
    );
    assert_eq!(
        contract["bead_id"].as_str().unwrap(),
        "bd-3ar8v.7.7",
        "bead_id must reference bd-3ar8v.7.7"
    );
    assert_eq!(
        contract["status"].as_str().unwrap(),
        "active_blocking_policy",
        "contract must be active_blocking_policy"
    );
}

#[test]
fn event_loop_contract_has_required_phases() {
    let contract = load_contract();
    let phases = contract["event_loop_model"]["phases"]
        .as_array()
        .expect("phases must be an array");

    assert!(
        phases.len() >= 5,
        "must define at least 5 event-loop phases (Node.js canonical), got {}",
        phases.len()
    );

    let phase_ids: HashSet<&str> = phases
        .iter()
        .filter_map(|p| p["phase_id"].as_str())
        .collect();

    for required in [
        "timers",
        "pending_callbacks",
        "poll",
        "check",
        "close_callbacks",
    ] {
        assert!(
            phase_ids.contains(required),
            "event_loop_model.phases must include {required}"
        );
    }

    for phase in phases {
        let pid = phase["phase_id"].as_str().unwrap_or("unknown");
        assert!(
            phase["description"].as_str().is_some_and(|d| !d.is_empty()),
            "phase {pid} must have non-empty description"
        );
        assert!(
            phase["node_parity"].as_str().is_some(),
            "phase {pid} must declare node_parity level"
        );
        assert!(
            phase["deterministic_fallback"].as_str().is_some(),
            "phase {pid} must declare deterministic_fallback behavior"
        );
    }
}

#[test]
fn event_loop_contract_microtask_semantics_are_node_compatible() {
    let contract = load_contract();
    let microtasks = &contract["event_loop_model"]["microtask_semantics"];

    assert!(
        microtasks["process_nextTick"].as_str().is_some(),
        "microtask_semantics must define process_nextTick behavior"
    );
    assert!(
        microtasks["promise_then"].as_str().is_some(),
        "microtask_semantics must define promise_then behavior"
    );
    assert_eq!(
        microtasks["node_parity"].as_str().unwrap(),
        "full",
        "microtask semantics must have full Node.js parity"
    );
}

#[test]
fn event_loop_contract_has_invariants() {
    let contract = load_contract();
    let invariants = contract["event_loop_model"]["invariants"]
        .as_array()
        .expect("invariants must be an array");

    assert!(
        invariants.len() >= 4,
        "must define at least 4 event-loop invariants, got {}",
        invariants.len()
    );

    let texts: Vec<&str> = invariants.iter().filter_map(Value::as_str).collect();
    assert!(
        texts
            .iter()
            .any(|t| t.contains("microtask") || t.contains("Microtask")),
        "invariants must mention microtask draining"
    );
    assert!(
        texts
            .iter()
            .any(|t| t.contains("starve") || t.contains("starvation")),
        "invariants must address timer/IO starvation"
    );
}

#[test]
fn event_loop_contract_io_primitives_cover_required_set() {
    let contract = load_contract();
    let primitives = contract["io_subsystem"]["supported_primitives"]
        .as_array()
        .expect("supported_primitives must be an array");

    assert!(
        primitives.len() >= 4,
        "must define at least 4 IO primitives, got {}",
        primitives.len()
    );

    let primitive_ids: HashSet<&str> = primitives
        .iter()
        .filter_map(|p| p["primitive_id"].as_str())
        .collect();

    for required in ["tcp_stream", "filesystem", "dns_resolver", "child_process"] {
        assert!(
            primitive_ids.contains(required),
            "io_subsystem.supported_primitives must include {required}"
        );
    }

    for prim in primitives {
        let pid = prim["primitive_id"].as_str().unwrap_or("unknown");
        assert!(
            prim["description"].as_str().is_some_and(|d| !d.is_empty()),
            "IO primitive {pid} must have non-empty description"
        );
        assert!(
            prim["node_api_parity"]
                .as_array()
                .is_some_and(|a| !a.is_empty()),
            "IO primitive {pid} must list node_api_parity references"
        );
        assert!(
            prim["fallback_on_unsupported"].as_str().is_some(),
            "IO primitive {pid} must declare fallback_on_unsupported behavior"
        );
    }
}

#[test]
fn event_loop_contract_io_primitives_never_silently_fail() {
    let contract = load_contract();
    let primitives = contract["io_subsystem"]["supported_primitives"]
        .as_array()
        .expect("supported_primitives must be an array");

    for prim in primitives {
        let pid = prim["primitive_id"].as_str().unwrap_or("unknown");
        let fallback = prim["fallback_on_unsupported"]
            .as_str()
            .expect("fallback_on_unsupported must be a string");
        assert!(
            fallback != "silent" && fallback != "ignore" && fallback != "swallow",
            "IO primitive {pid} has silent/permissive fallback: {fallback}; \
             all primitives must produce diagnostics or explicit errors on unsupported operations"
        );
    }
}

#[test]
fn event_loop_contract_compatibility_toggles_have_required_fields() {
    let contract = load_contract();
    let toggles = contract["compatibility_toggles"]
        .as_array()
        .expect("compatibility_toggles must be an array");

    assert!(
        toggles.len() >= 2,
        "must define at least 2 compatibility toggles, got {}",
        toggles.len()
    );

    let mut toggle_ids = HashSet::new();
    for toggle in toggles {
        let tid = toggle["toggle_id"]
            .as_str()
            .expect("toggle_id must be a string");
        assert!(
            toggle_ids.insert(tid.to_string()),
            "duplicate toggle_id: {tid}"
        );
        assert!(
            toggle["description"]
                .as_str()
                .is_some_and(|d| !d.is_empty()),
            "toggle {tid} must have non-empty description"
        );
        assert!(
            toggle["default"].is_boolean(),
            "toggle {tid} must have boolean default"
        );
        assert!(
            toggle["affects"].as_array().is_some_and(|a| !a.is_empty()),
            "toggle {tid} must list affected behaviors"
        );
    }
}

#[test]
fn event_loop_contract_parity_matrix_covers_core_apis() {
    let contract = load_contract();
    let covered = contract["parity_matrix"]["covered_apis"]
        .as_array()
        .expect("covered_apis must be an array");

    let api_set: HashSet<&str> = covered.iter().filter_map(Value::as_str).collect();

    for required in [
        "setTimeout",
        "setInterval",
        "setImmediate",
        "process.nextTick",
        "queueMicrotask",
        "EventEmitter",
    ] {
        assert!(
            api_set.contains(required),
            "parity_matrix.covered_apis must include {required}"
        );
    }
}

#[test]
fn event_loop_contract_known_divergences_have_severity() {
    let contract = load_contract();
    let divergences = contract["parity_matrix"]["known_divergences"]
        .as_array()
        .expect("known_divergences must be an array");

    for div in divergences {
        let api = div["api"].as_str().unwrap_or("unknown");
        assert!(
            div["divergence"].as_str().is_some_and(|d| !d.is_empty()),
            "known divergence for {api} must have non-empty divergence description"
        );
        assert!(
            div["reason"].as_str().is_some_and(|r| !r.is_empty()),
            "known divergence for {api} must have non-empty reason"
        );
        assert!(
            div["severity"].as_str().is_some(),
            "known divergence for {api} must have severity"
        );
    }
}

#[test]
fn event_loop_contract_release_blockers_are_defined() {
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
        blocker_texts.iter().any(|b| b.contains("phase")),
        "release_blockers must mention event-loop phase ordering"
    );
    assert!(
        blocker_texts.iter().any(|b| b.contains("microtask")),
        "release_blockers must mention microtask draining"
    );
}
