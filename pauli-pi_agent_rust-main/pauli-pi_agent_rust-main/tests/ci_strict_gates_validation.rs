//! CI: Strict non-mock regression and logging completeness gates (bd-1f42.8.8).
//!
//! Validates that CI gate infrastructure enforces:
//! 1. Non-mock inventory drift detection (new test doubles → gate failure)
//! 2. Coverage floor regressions from non-mock-rubric.json
//! 3. E2E logging/evidence contract quality
//! 4. Waiver lifecycle compliance (expiry, scope, audit trail)
//! 5. Gate failure remediation commands
//!
//! Run:
//! ```bash
//! cargo test --test ci_strict_gates_validation
//! ```

#![allow(clippy::doc_markdown)]
#![allow(clippy::items_after_statements)]

use serde_json::Value;

// ─── Constants ──────────────────────────────────────────────────────────────

const NON_MOCK_RUBRIC_PATH: &str = "docs/non-mock-rubric.json";
const TEST_DOUBLE_INVENTORY_PATH: &str = "docs/test_double_inventory.json";
const TESTING_POLICY_PATH: &str = "docs/testing-policy.md";
const SUITE_CLASSIFICATION_PATH: &str = "tests/suite_classification.toml";
const CI_WORKFLOW_PATH: &str = ".github/workflows/ci.yml";
const CI_OPERATOR_RUNBOOK_PATH: &str = "docs/ci-operator-runbook.md";
const SCENARIO_MATRIX_PATH: &str = "docs/e2e_scenario_matrix.json";
const FULL_SUITE_GATE_PATH: &str = "tests/ci_full_suite_gate.rs";
const RELEASE_EVIDENCE_GATE_PATH: &str = "tests/release_evidence_gate.rs";
const PRACTICAL_FINISH_GATE_ID: &str = "practical_finish_checkpoint";
const EXT_REMEDIATION_GATE_ID: &str = "extension_remediation_backlog";
const PARAMETER_SWEEPS_GATE_ID: &str = "parameter_sweeps_integrity";
const CONFORMANCE_STRESS_LINEAGE_GATE_ID: &str = "conformance_stress_lineage";

fn load_json(path: &str) -> Value {
    let content = std::fs::read_to_string(path).unwrap_or_else(|_| panic!("Should read {path}"));
    serde_json::from_str(&content).unwrap_or_else(|_| panic!("Should parse {path} as JSON"))
}

fn load_text(path: &str) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|_| panic!("Should read {path}"))
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 1: Non-mock rubric exists and is well-formed
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn non_mock_rubric_exists_with_valid_schema() {
    let rubric = load_json(NON_MOCK_RUBRIC_PATH);
    assert!(
        rubric["schema"]
            .as_str()
            .is_some_and(|s| s.starts_with("pi.qa.non_mock_rubric")),
        "non-mock-rubric.json must have a schema field"
    );
}

#[test]
fn non_mock_rubric_has_module_thresholds() {
    let rubric = load_json(NON_MOCK_RUBRIC_PATH);
    // module_thresholds can be an object (keyed by module name) or an array
    let has_thresholds = rubric["module_thresholds"].is_object()
        || rubric["module_thresholds"].is_array()
        || rubric["modules"].is_object()
        || rubric["modules"].is_array();
    assert!(
        has_thresholds,
        "non-mock-rubric.json must define module-level coverage thresholds"
    );
}

#[test]
fn non_mock_rubric_covers_critical_modules() {
    let rubric = load_json(NON_MOCK_RUBRIC_PATH);
    let text = serde_json::to_string(&rubric).unwrap_or_default();

    let critical = ["agent", "tools", "provider", "session", "extension"];
    for module in &critical {
        assert!(
            text.contains(module),
            "non-mock-rubric must cover critical module: {module}"
        );
    }
}

#[test]
fn non_mock_rubric_has_exception_template() {
    let rubric = load_json(NON_MOCK_RUBRIC_PATH);
    let text = serde_json::to_string(&rubric).unwrap_or_default();
    assert!(
        text.contains("exception") || text.contains("allowlist"),
        "non-mock-rubric must define an exception/allowlist template"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 2: Test double inventory baseline
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_double_inventory_exists_with_schema() {
    let inventory = load_json(TEST_DOUBLE_INVENTORY_PATH);
    assert!(
        inventory["schema"].is_string(),
        "test_double_inventory.json must declare schema"
    );
}

#[test]
fn test_double_inventory_has_entry_count() {
    let inventory = load_json(TEST_DOUBLE_INVENTORY_PATH);
    let text = serde_json::to_string(&inventory).unwrap_or_default();
    assert!(
        text.contains("entry_count") || text.contains("entries"),
        "inventory must report entry counts"
    );
}

#[test]
fn test_double_inventory_has_risk_distribution() {
    let inventory = load_json(TEST_DOUBLE_INVENTORY_PATH);
    let text = serde_json::to_string(&inventory).unwrap_or_default();
    assert!(
        text.contains("risk") || text.contains("high") || text.contains("severity"),
        "inventory must include risk categorization"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 3: Testing policy defines enforcement rules
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn testing_policy_exists() {
    assert!(
        std::path::Path::new(TESTING_POLICY_PATH).exists(),
        "docs/testing-policy.md must exist"
    );
}

#[test]
fn testing_policy_defines_suite_categories() {
    let policy = load_text(TESTING_POLICY_PATH);
    let categories = ["Unit", "VCR", "E2E"];
    for cat in &categories {
        assert!(
            policy.contains(cat),
            "testing-policy must define suite category: {cat}"
        );
    }
}

#[test]
fn testing_policy_lists_allowlisted_exceptions() {
    let policy = load_text(TESTING_POLICY_PATH);
    assert!(
        policy.contains("MockHttpServer") || policy.contains("allowlist"),
        "testing-policy must list allowlisted test double exceptions"
    );
}

#[test]
fn testing_policy_defines_ci_enforcement() {
    let policy = load_text(TESTING_POLICY_PATH);
    assert!(
        policy.contains("CI") || policy.contains("enforcement") || policy.contains("gate"),
        "testing-policy must define CI enforcement rules"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 4: CI workflow has gate stages
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn ci_workflow_exists() {
    assert!(
        std::path::Path::new(CI_WORKFLOW_PATH).exists(),
        ".github/workflows/ci.yml must exist"
    );
}

#[test]
fn ci_workflow_has_suite_classification_guard() {
    let ci = load_text(CI_WORKFLOW_PATH);
    assert!(
        ci.contains("suite_classification") || ci.contains("suite-classification"),
        "CI must include suite classification guard"
    );
}

#[test]
fn ci_workflow_has_coverage_gate() {
    let ci = load_text(CI_WORKFLOW_PATH);
    assert!(
        ci.contains("coverage") || ci.contains("llvm-cov"),
        "CI must include coverage gate"
    );
}

#[test]
fn ci_workflow_has_clippy_fmt_gates() {
    let ci = load_text(CI_WORKFLOW_PATH);
    assert!(ci.contains("clippy"), "CI must include clippy gate");
    assert!(ci.contains("fmt"), "CI must include fmt gate");
}

#[test]
fn ci_workflow_has_conformance_gate() {
    let ci = load_text(CI_WORKFLOW_PATH);
    assert!(
        ci.contains("conformance"),
        "CI must include conformance regression gate"
    );
}

#[test]
fn ci_workflow_has_evidence_bundle_gate() {
    let ci = load_text(CI_WORKFLOW_PATH);
    assert!(
        ci.contains("evidence") || ci.contains("bundle"),
        "CI must include evidence bundle gate"
    );
}

#[test]
fn ci_operator_runbook_retains_perf3x_incident_addendum_heading() {
    let runbook = load_text(CI_OPERATOR_RUNBOOK_PATH);
    assert!(
        runbook.contains("### PERF-3X Gate Incident Addendum (bd-3ar8v.6.4)"),
        "CI operator runbook must retain PERF-3X incident addendum heading"
    );
}

#[test]
fn ci_operator_runbook_retains_perf3x_incident_artifact_checklist() {
    let runbook = load_text(CI_OPERATOR_RUNBOOK_PATH);
    for token in [
        "tests/full_suite_gate/practical_finish_checkpoint.json",
        "tests/full_suite_gate/extension_remediation_backlog.json",
        "tests/perf/reports/budget_summary.json",
        "tests/perf/reports/perf_comparison.json",
        "tests/perf/reports/stress_triage.json",
        "tests/perf/reports/budget_events.jsonl",
        "tests/perf/reports/perf_comparison_events.jsonl",
        "tests/perf/reports/stress_events.jsonl",
    ] {
        assert!(
            runbook.contains(token),
            "CI operator runbook must retain PERF-3X artifact token: {token}"
        );
    }
}

#[test]
fn ci_operator_runbook_retains_parameter_sweeps_signature_playbook_tokens() {
    let runbook = load_text(CI_OPERATOR_RUNBOOK_PATH);
    for token in [
        "### PERF-3X signature: `parameter_sweeps_integrity` gate failure",
        "full_suite_verdict.json",
        "parameter_sweeps_integrity",
        "tests/perf/reports/parameter_sweeps.json",
        "tests/perf/reports/parameter_sweeps_events.jsonl",
        "tests/perf/reports/phase1_matrix_validation.json",
        "rch exec -- cargo test --test release_evidence_gate --",
        "parameter_sweeps_contract_links_phase1_matrix_and_readiness --nocapture --exact",
        "Enforce artifact schema `pi.perf.parameter_sweeps.v1`.",
        "source_identity",
        "phase1_matrix_validation",
        "status = ready",
        "status = blocked",
        "ready_for_phase5",
        "blocking_reasons",
        "docs/qa-runbook.md",
        "PERF-3X Regression Triage (bd-3ar8v.6.4)",
    ] {
        assert!(
            runbook.contains(token),
            "CI operator runbook must retain parameter_sweeps signature token: {token}"
        );
    }
}

#[test]
fn ci_operator_runbook_retains_practical_finish_signature_playbook_tokens() {
    let runbook = load_text(CI_OPERATOR_RUNBOOK_PATH);
    for token in [
        "### PERF-3X signature: `practical_finish_checkpoint` readiness drift",
        "technical PERF-3X issue(s) still open",
        "Fail-closed practical-finish source read error",
        "tests/full_suite_gate/practical_finish_checkpoint.json",
        ".beads/issues.jsonl",
        "tests/full_suite_gate/certification_events.jsonl",
        "rch exec -- cargo test --test ci_full_suite_gate --",
        "practical_finish_report_fails_when_technical_open_issues_remain --nocapture --exact",
        "rch exec -- cargo test --test release_readiness -- practical_finish_checkpoint_ -- --nocapture",
        "pi.perf3x.practical_finish_checkpoint.v1",
        "technical_completion_reached",
        "residual_open_scope",
        "open_perf3x_count = technical_open_count + docs_or_report_open_count",
        "docs/qa-runbook.md",
        "PERF-3X Regression Triage (bd-3ar8v.6.4)",
    ] {
        assert!(
            runbook.contains(token),
            "CI operator runbook must retain practical_finish signature token: {token}"
        );
    }
}

#[test]
fn ci_operator_runbook_retains_franken_node_claim_tier_order_signature_tokens() {
    let runbook = load_text(CI_OPERATOR_RUNBOOK_PATH);
    for token in [
        "### FrankenNode claim signature: `claim_tier_order_drift`",
        "docs/franken-node-claim-gating-contract.json",
        "tests/full_suite_gate/franken_node_claim_verdict.json",
        "TIER-1-EXTENSION-HOST-PARITY",
        "TIER-2-TARGETED-RUNTIME-PARITY",
        "TIER-3-FULL-NODE-BUN-REPLACEMENT",
        "rch exec -- cargo test --test franken_node_claim_contract --",
        "franken_node_claim_contract_declares_expected_tier_order -- --nocapture",
        "rch exec -- cargo test --test release_evidence_gate --",
        "franken_node_claim_contract_is_present_and_valid --nocapture --exact",
    ] {
        assert!(
            runbook.contains(token),
            "CI operator runbook must retain franken-node claim-tier-order token: {token}"
        );
    }
}

#[test]
fn ci_operator_runbook_retains_franken_node_kernel_boundary_drift_signature_tokens() {
    let runbook = load_text(CI_OPERATOR_RUNBOOK_PATH);
    for token in [
        "### FrankenNode kernel-boundary signature: `kernel_boundary_drift`",
        "docs/franken-node-kernel-extraction-boundary-manifest.json",
        "tests/full_suite_gate/franken_node_kernel_boundary_drift_report.json",
        "kernel_boundary.all_modules_mapped_or_deferred",
        "kernel_boundary.no_duplicate_domain_ownership",
        "kernel_boundary.banned_cross_boundary_pairs_absent",
        "rch exec -- cargo test --test franken_node_kernel_extraction_boundary_manifest --",
        "kernel_boundary_manifest_ -- --nocapture",
        "rch exec -- cargo test --test qa_docs_policy_validation --",
        "franken_node_mission_contract_tier_mapping_declares_required_checks_and_phase6_beads -- --nocapture",
    ] {
        assert!(
            runbook.contains(token),
            "CI operator runbook must retain franken-node kernel-boundary drift token: {token}"
        );
    }
}

#[test]
fn ci_operator_runbook_retains_node_runtime_availability_signature_tokens() {
    let runbook = load_text(CI_OPERATOR_RUNBOOK_PATH);
    for token in [
        "### FrankenNode compat harness signature: `node_runtime_unavailable_or_shimmed`",
        "Node.js not found",
        "SKIP: generate_compatibility_matrix requires both Node.js and Bun",
        "tests/franken_node_compat_harness.rs",
        "rch exec -- cargo test --test franken_node_compat_harness --",
        "node_detection_rejects_bun_node_shim_when_present -- --nocapture",
        "generate_compatibility_matrix -- --nocapture",
        "find_node()",
        "is_real_node()",
        "/home/ubuntu/.bun/bin/node",
    ] {
        assert!(
            runbook.contains(token),
            "CI operator runbook must retain node-runtime-availability token: {token}"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 5: Full suite gate has blocking gates
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn full_suite_gate_exists() {
    assert!(
        std::path::Path::new(FULL_SUITE_GATE_PATH).exists(),
        "tests/ci_full_suite_gate.rs must exist"
    );
}

#[test]
fn full_suite_gate_has_preflight_lane() {
    let gate = load_text(FULL_SUITE_GATE_PATH);
    assert!(
        gate.contains("preflight"),
        "full suite gate must have preflight fast-fail lane"
    );
}

#[test]
fn full_suite_gate_has_full_certification_lane() {
    let gate = load_text(FULL_SUITE_GATE_PATH);
    assert!(
        gate.contains("full") && gate.contains("certification"),
        "full suite gate must have full certification lane"
    );
}

#[test]
fn full_suite_gate_has_blocking_verdicts() {
    let gate = load_text(FULL_SUITE_GATE_PATH);
    assert!(
        gate.contains("blocking"),
        "full suite gate must support blocking verdicts"
    );
}

#[test]
fn full_suite_gate_validates_waiver_lifecycle() {
    let gate = load_text(FULL_SUITE_GATE_PATH);
    assert!(
        gate.contains("waiver"),
        "full suite gate must validate waiver lifecycle"
    );
}

#[test]
fn full_suite_gate_wires_practical_finish_checkpoint_contract() {
    let gate = load_text(FULL_SUITE_GATE_PATH);
    for token in [
        PRACTICAL_FINISH_GATE_ID,
        "evaluate_practical_finish_checkpoint",
        "pi.perf3x.practical_finish_checkpoint.v1",
    ] {
        assert!(
            gate.contains(token),
            "full suite gate must retain practical_finish_checkpoint token: {token}"
        );
    }
}

#[test]
fn full_suite_gate_wires_extension_remediation_backlog_contract() {
    let gate = load_text(FULL_SUITE_GATE_PATH);
    for token in [
        EXT_REMEDIATION_GATE_ID,
        "check_extension_remediation_backlog_artifact",
        "pi.qa.extension_remediation_backlog.v1",
    ] {
        assert!(
            gate.contains(token),
            "full suite gate must retain extension_remediation_backlog token: {token}"
        );
    }
}

#[test]
fn full_suite_gate_wires_parameter_sweeps_integrity_contract() {
    let gate = load_text(FULL_SUITE_GATE_PATH);
    for token in [
        PARAMETER_SWEEPS_GATE_ID,
        "check_parameter_sweeps_artifact",
        "pi.perf.parameter_sweeps.v1",
    ] {
        assert!(
            gate.contains(token),
            "full suite gate must retain parameter_sweeps token: {token}"
        );
    }
}

#[test]
fn full_suite_gate_wires_conformance_stress_lineage_contract() {
    let gate = load_text(FULL_SUITE_GATE_PATH);
    for token in [
        CONFORMANCE_STRESS_LINEAGE_GATE_ID,
        "check_conformance_stress_lineage_coherence",
        "stress_triage.json",
        "conformance_summary.json",
    ] {
        assert!(
            gate.contains(token),
            "full suite gate must retain conformance_stress_lineage token: {token}"
        );
    }
}

#[test]
fn full_suite_gate_keeps_phase1_matrix_claim_integrity_tokens() {
    let gate = load_text(FULL_SUITE_GATE_PATH);
    for token in [
        "claim_integrity.phase1_matrix_validation_path_configured",
        "claim_integrity.phase1_matrix_validation_schema",
        "claim_integrity.phase1_matrix_cells_primary_e2e_metrics_present",
    ] {
        assert!(
            gate.contains(token),
            "full suite gate must retain phase1 matrix claim-integrity token: {token}"
        );
    }
}

#[test]
fn release_evidence_gate_retains_canonical_franken_node_tier_order_tokens() {
    let gate = load_text(RELEASE_EVIDENCE_GATE_PATH);
    for token in [
        "FRANKEN_NODE_REQUIRED_TIER_IDS",
        "\"TIER-1-EXTENSION-HOST-PARITY\"",
        "\"TIER-2-TARGETED-RUNTIME-PARITY\"",
        "\"TIER-3-FULL-NODE-BUN-REPLACEMENT\"",
        "missing required claim tier",
    ] {
        assert!(
            gate.contains(token),
            "release_evidence_gate must retain franken-node tier-order token: {token}"
        );
    }

    let tier1 = gate
        .find("\"TIER-1-EXTENSION-HOST-PARITY\"")
        .expect("tier-1 token must exist");
    let tier2 = gate
        .find("\"TIER-2-TARGETED-RUNTIME-PARITY\"")
        .expect("tier-2 token must exist");
    let tier3 = gate
        .find("\"TIER-3-FULL-NODE-BUN-REPLACEMENT\"")
        .expect("tier-3 token must exist");
    assert!(
        tier1 < tier2 && tier2 < tier3,
        "release_evidence_gate must preserve canonical tier token ordering"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 6: Suite classification has waiver infrastructure
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn suite_classification_exists() {
    assert!(
        std::path::Path::new(SUITE_CLASSIFICATION_PATH).exists(),
        "tests/suite_classification.toml must exist"
    );
}

#[test]
fn suite_classification_is_valid_toml() {
    let content = load_text(SUITE_CLASSIFICATION_PATH);
    // Use toml::Table (document) parse — toml 1.0 changed Value::from_str
    // to parse a single value expression rather than a full TOML document.
    if let Err(e) = content.parse::<toml::Table>() {
        panic!("suite_classification.toml must be valid TOML: {e}");
    }
}

#[test]
fn suite_classification_has_suite_sections() {
    let content = load_text(SUITE_CLASSIFICATION_PATH);
    assert!(
        content.contains("[suite."),
        "suite_classification must define [suite.*] sections"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 7: Remediation commands in gate outputs
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn ci_gate_failures_include_remediation_hints() {
    let gate = load_text(FULL_SUITE_GATE_PATH);
    // Gate failures should include remediation or hint text
    assert!(
        gate.contains("remediation") || gate.contains("hint") || gate.contains("fix"),
        "gate failure outputs must include remediation guidance"
    );
}

#[test]
fn scenario_matrix_consumed_by_ci_gates() {
    let matrix = load_json(SCENARIO_MATRIX_PATH);
    let consumed_by = matrix["ci_policy"]["consumed_by"]
        .as_array()
        .expect("consumed_by array");
    let consumers: Vec<&str> = consumed_by.iter().filter_map(Value::as_str).collect();
    assert!(
        consumers.iter().any(|c| c.contains("ci_full_suite_gate")),
        "scenario matrix must be consumed by ci_full_suite_gate"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 8: Gate promotion infrastructure
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn ci_gate_promotion_mode_supported() {
    let ci = load_text(CI_WORKFLOW_PATH);
    assert!(
        ci.contains("PROMOTION") || ci.contains("promotion") || ci.contains("strict"),
        "CI must support gate promotion mode (strict/rollback)"
    );
}

#[test]
fn ci_gate_pass_rate_threshold_defined() {
    let ci = load_text(CI_WORKFLOW_PATH);
    assert!(
        ci.contains("PASS_RATE") || ci.contains("pass_rate") || ci.contains("threshold"),
        "CI must define pass rate threshold for gates"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 9: Evidence artifacts exist
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn full_suite_verdict_artifact_exists() {
    let path = "tests/full_suite_gate/full_suite_verdict.json";
    assert!(
        std::path::Path::new(path).exists(),
        "full_suite_verdict.json must exist for CI verification"
    );
}

#[test]
fn full_suite_verdict_has_gates() {
    let verdict = load_json("tests/full_suite_gate/full_suite_verdict.json");
    assert!(
        verdict["gates"].is_array() || verdict["sub_gates"].is_array(),
        "full_suite_verdict must contain gates array"
    );
}

#[test]
fn full_suite_report_artifact_exists() {
    let path = "tests/full_suite_gate/full_suite_report.md";
    assert!(
        std::path::Path::new(path).exists(),
        "full_suite_report.md must exist"
    );
}

#[test]
fn full_suite_events_artifact_exists() {
    let path = "tests/full_suite_gate/full_suite_events.jsonl";
    assert!(
        std::path::Path::new(path).exists(),
        "full_suite_events.jsonl must exist"
    );
}
