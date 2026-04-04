use serde_json::Value;
use std::collections::HashSet;
use std::path::PathBuf;

const CONTRACT_PATH: &str = "docs/franken-node-migration-assistant-contract.json";
const EXPECTED_SCHEMA: &str = "pi.frankennode.migration_assistant_contract.v1";

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
fn migration_contract_exists_and_has_correct_schema() {
    let contract = load_contract();
    assert_eq!(
        contract["schema"].as_str().unwrap(),
        EXPECTED_SCHEMA,
        "schema must be {EXPECTED_SCHEMA}"
    );
    assert_eq!(
        contract["bead_id"].as_str().unwrap(),
        "bd-3ar8v.7.14",
        "bead_id must reference bd-3ar8v.7.14"
    );
    assert_eq!(
        contract["status"].as_str().unwrap(),
        "active_blocking_policy",
        "contract must be active_blocking_policy"
    );
}

#[test]
fn migration_contract_doctor_has_required_check_categories() {
    let contract = load_contract();
    let categories = contract["compatibility_doctor"]["check_categories"]
        .as_array()
        .expect("check_categories must be an array");

    assert!(
        categories.len() >= 4,
        "must define at least 4 check categories, got {}",
        categories.len()
    );

    let cat_ids: HashSet<&str> = categories
        .iter()
        .filter_map(|c| c["category_id"].as_str())
        .collect();

    for required in [
        "package_compatibility",
        "native_addon_detection",
        "api_surface_coverage",
        "module_system_analysis",
    ] {
        assert!(
            cat_ids.contains(required),
            "check_categories must include {required}"
        );
    }

    for cat in categories {
        let cid = cat["category_id"].as_str().unwrap_or("unknown");
        assert!(
            cat["description"].as_str().is_some_and(|d| !d.is_empty()),
            "check category {cid} must have non-empty description"
        );
        assert!(
            cat["output"].as_str().is_some(),
            "check category {cid} must declare output format"
        );
    }
}

#[test]
fn migration_contract_verdict_levels_include_required_set() {
    let contract = load_contract();
    let levels = contract["compatibility_doctor"]["verdict_levels"]
        .as_array()
        .expect("verdict_levels must be an array");

    let level_set: HashSet<&str> = levels.iter().filter_map(|l| l["level"].as_str()).collect();

    for required in ["pass", "warn", "action_required", "blocked"] {
        assert!(
            level_set.contains(required),
            "verdict_levels must include {required}"
        );
    }

    for level in levels {
        let lid = level["level"].as_str().unwrap_or("unknown");
        assert!(
            level["description"].as_str().is_some_and(|d| !d.is_empty()),
            "verdict level {lid} must have non-empty description"
        );
    }
}

#[test]
fn migration_contract_doctor_is_static_analysis_only() {
    let contract = load_contract();
    let invariants = contract["compatibility_doctor"]["invariants"]
        .as_array()
        .expect("doctor invariants must be an array");

    let texts: Vec<&str> = invariants.iter().filter_map(Value::as_str).collect();
    assert!(
        texts
            .iter()
            .any(|t| t.contains("without executing user code") || t.contains("static analysis")),
        "doctor invariants must guarantee static-analysis-only operation"
    );
}

#[test]
fn migration_contract_doctor_action_required_always_has_fix() {
    let contract = load_contract();
    let invariants = contract["compatibility_doctor"]["invariants"]
        .as_array()
        .expect("doctor invariants must be an array");

    let texts: Vec<&str> = invariants.iter().filter_map(Value::as_str).collect();
    assert!(
        texts
            .iter()
            .any(|t| t.contains("action_required") && t.contains("actionable fix")),
        "doctor invariants must guarantee action_required verdicts include fixes"
    );
}

#[test]
fn migration_contract_workflow_has_required_phases() {
    let contract = load_contract();
    let phases = contract["migration_workflow"]["phases"]
        .as_array()
        .expect("phases must be an array");

    assert!(
        phases.len() >= 4,
        "must define at least 4 workflow phases, got {}",
        phases.len()
    );

    let phase_ids: HashSet<&str> = phases
        .iter()
        .filter_map(|p| p["phase_id"].as_str())
        .collect();

    for required in ["assess", "plan", "adapt", "verify"] {
        assert!(
            phase_ids.contains(required),
            "migration_workflow.phases must include {required}"
        );
    }

    for phase in phases {
        let pid = phase["phase_id"].as_str().unwrap_or("unknown");
        assert!(
            phase["description"].as_str().is_some_and(|d| !d.is_empty()),
            "workflow phase {pid} must have non-empty description"
        );
        assert!(
            phase["output_artifact"].as_str().is_some(),
            "workflow phase {pid} must declare output_artifact"
        );
    }
}

#[test]
fn migration_contract_adapt_phase_requires_user_confirmation() {
    let contract = load_contract();
    let phases = contract["migration_workflow"]["phases"]
        .as_array()
        .expect("phases must be an array");

    let adapt_phase = phases
        .iter()
        .find(|p| p["phase_id"].as_str() == Some("adapt"))
        .expect("adapt phase must exist");

    assert_eq!(
        adapt_phase["requires_user_confirmation"].as_bool(),
        Some(true),
        "adapt phase must require user confirmation before modifying files"
    );
}

#[test]
fn migration_contract_workflow_invariants_prevent_unsafe_modifications() {
    let contract = load_contract();
    let invariants = contract["migration_workflow"]["invariants"]
        .as_array()
        .expect("workflow invariants must be an array");

    let texts: Vec<&str> = invariants.iter().filter_map(Value::as_str).collect();
    assert!(
        texts
            .iter()
            .any(|t| t.contains("never modifies files without") || t.contains("user confirmation")),
        "workflow invariants must prevent file modifications without user confirmation"
    );
    assert!(
        texts
            .iter()
            .any(|t| t.contains("resumed") || t.contains("checkpoint")),
        "workflow invariants must support phase-level resumption"
    );
}

#[test]
fn migration_contract_fix_suggestions_cover_required_types() {
    let contract = load_contract();
    let types = contract["fix_suggestion_engine"]["suggestion_types"]
        .as_array()
        .expect("suggestion_types must be an array");

    assert!(
        types.len() >= 3,
        "must define at least 3 suggestion types, got {}",
        types.len()
    );

    let type_ids: HashSet<&str> = types.iter().filter_map(|t| t["type_id"].as_str()).collect();

    for required in [
        "api_replacement",
        "polyfill_injection",
        "configuration_change",
    ] {
        assert!(
            type_ids.contains(required),
            "suggestion_types must include {required}"
        );
    }

    for stype in types {
        let tid = stype["type_id"].as_str().unwrap_or("unknown");
        assert!(
            stype["description"].as_str().is_some_and(|d| !d.is_empty()),
            "suggestion type {tid} must have non-empty description"
        );
        assert!(
            stype["example"].as_str().is_some_and(|e| !e.is_empty()),
            "suggestion type {tid} must include an example"
        );
    }
}

#[test]
fn migration_contract_fix_suggestions_never_introduce_vulnerabilities() {
    let contract = load_contract();
    let invariants = contract["fix_suggestion_engine"]["invariants"]
        .as_array()
        .expect("fix_suggestion_engine invariants must be an array");

    let texts: Vec<&str> = invariants.iter().filter_map(Value::as_str).collect();
    assert!(
        texts
            .iter()
            .any(|t| t.contains("security vulnerabilities") || t.contains("security contract")),
        "fix suggestion invariants must prohibit introducing security vulnerabilities"
    );
    assert!(
        texts.iter().any(|t| t.contains("idempotent")),
        "fix suggestion invariants must guarantee idempotency"
    );
}

#[test]
fn migration_contract_reporting_has_required_fields() {
    let contract = load_contract();
    let fields = contract["reporting"]["required_fields"]
        .as_array()
        .expect("required_fields must be an array");

    let field_set: HashSet<&str> = fields.iter().filter_map(Value::as_str).collect();

    for required in [
        "project_name",
        "assessment_timestamp_utc",
        "total_checks_run",
        "pass_count",
        "blocked_count",
        "overall_verdict",
    ] {
        assert!(
            field_set.contains(required),
            "reporting.required_fields must include {required}"
        );
    }
}

#[test]
fn migration_contract_release_blockers_are_defined() {
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
        blocker_texts
            .iter()
            .any(|b| b.contains("executes user code")),
        "release_blockers must prohibit executing user code during analysis"
    );
    assert!(
        blocker_texts
            .iter()
            .any(|b| b.contains("without user confirmation") || b.contains("modifies files")),
        "release_blockers must prohibit unsolicited file modifications"
    );
}

#[test]
fn migration_contract_downstream_dependencies_reference_blocked_beads() {
    let contract = load_contract();
    let blocked: HashSet<&str> = contract["downstream_dependencies"]["blocked_beads"]
        .as_array()
        .expect("blocked_beads must be an array")
        .iter()
        .filter_map(Value::as_str)
        .collect();

    for required in ["bd-3ar8v.7.10", "bd-3ar8v.7.11", "bd-3ar8v.7.13"] {
        assert!(
            blocked.contains(required),
            "downstream_dependencies.blocked_beads must include {required}"
        );
    }
}
