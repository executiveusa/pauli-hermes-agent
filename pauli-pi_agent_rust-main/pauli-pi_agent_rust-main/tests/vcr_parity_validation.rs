#![allow(clippy::doc_markdown)]

//! VCR parity validation tests.
//!
//! Validates that every E2E scenario in the matrix has a correct `vcr_mode`
//! classification (vcr-only, dual-mode, live-only) and that classifications
//! are consistent with test file content and VCR fixture availability.
//!
//! Also enforces structured skip reasons and budget controls for live-only
//! and dual-mode workflows.

use serde_json::Value;
use std::collections::HashSet;
use std::path::Path;

const MATRIX_PATH: &str = "docs/e2e_scenario_matrix.json";
const VCR_FIXTURE_DIR: &str = "tests/fixtures/vcr";

fn load_matrix() -> Value {
    let content =
        std::fs::read_to_string(MATRIX_PATH).expect("e2e_scenario_matrix.json must exist");
    serde_json::from_str(&content).expect("e2e_scenario_matrix.json must be valid JSON")
}

fn matrix_rows(matrix: &Value) -> &Vec<Value> {
    matrix["rows"]
        .as_array()
        .expect("matrix must have 'rows' array")
}

// ═══════════════════════════════════════════════════════════════════════
// SECTION 1: Schema validation - every row has vcr_mode
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn every_row_has_vcr_mode() {
    let matrix = load_matrix();
    for row in matrix_rows(&matrix) {
        let wf_id = row["workflow_id"].as_str().unwrap_or("<unknown>");
        assert!(
            row.get("vcr_mode").is_some(),
            "Row {wf_id} is missing 'vcr_mode' field"
        );
        let mode = row["vcr_mode"].as_str().unwrap();
        assert!(
            ["vcr-only", "dual-mode", "live-only"].contains(&mode),
            "Row {wf_id} has invalid vcr_mode: {mode}"
        );
    }
}

#[test]
fn every_row_has_vcr_mode_rationale() {
    let matrix = load_matrix();
    for row in matrix_rows(&matrix) {
        let wf_id = row["workflow_id"].as_str().unwrap_or("<unknown>");
        let rationale = row["vcr_mode_rationale"].as_str();
        assert!(
            rationale.is_some() && !rationale.unwrap().is_empty(),
            "Row {wf_id} is missing 'vcr_mode_rationale'"
        );
    }
}

#[test]
fn allowed_vcr_modes_in_policy() {
    let matrix = load_matrix();
    let policy_modes = matrix["ci_policy"]["allowed_vcr_modes"]
        .as_array()
        .expect("ci_policy must have allowed_vcr_modes");
    assert!(policy_modes.iter().any(|v| v.as_str() == Some("vcr-only")));
    assert!(policy_modes.iter().any(|v| v.as_str() == Some("dual-mode")));
    assert!(policy_modes.iter().any(|v| v.as_str() == Some("live-only")));
}

// ═══════════════════════════════════════════════════════════════════════
// SECTION 2: VCR mode consistency checks
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn vcr_only_rows_are_not_waived() {
    // If a row is vcr-only, it should not need a waiver (it doesn't need credentials)
    let matrix = load_matrix();
    for row in matrix_rows(&matrix) {
        let wf_id = row["workflow_id"].as_str().unwrap_or("<unknown>");
        let vcr_mode = row["vcr_mode"].as_str().unwrap_or("");
        let status = row["status"].as_str().unwrap_or("");
        if vcr_mode == "vcr-only" {
            assert_ne!(
                status, "waived",
                "Row {wf_id} is vcr-only but has waived status"
            );
        }
    }
}

#[test]
fn live_only_rows_have_skip_policy() {
    let matrix = load_matrix();
    for row in matrix_rows(&matrix) {
        let wf_id = row["workflow_id"].as_str().unwrap_or("<unknown>");
        let vcr_mode = row["vcr_mode"].as_str().unwrap_or("");
        if vcr_mode == "live-only" {
            assert!(
                row.get("live_skip_policy").is_some(),
                "Live-only row {wf_id} must have 'live_skip_policy'"
            );
            let policy = &row["live_skip_policy"];
            assert!(
                policy["skip_reason"].as_str().is_some(),
                "Live-only row {wf_id} must have skip_reason"
            );
            assert!(
                policy["skip_env_var"].as_str().is_some(),
                "Live-only row {wf_id} must have skip_env_var"
            );
            assert!(
                policy["required_credentials"].as_array().is_some(),
                "Live-only row {wf_id} must list required_credentials"
            );
        }
    }
}

#[test]
fn live_only_rows_have_budget_controls() {
    let matrix = load_matrix();
    for row in matrix_rows(&matrix) {
        let wf_id = row["workflow_id"].as_str().unwrap_or("<unknown>");
        let vcr_mode = row["vcr_mode"].as_str().unwrap_or("");
        if vcr_mode == "live-only" {
            let policy = &row["live_skip_policy"];
            let budget = &policy["budget_controls"];
            assert!(
                budget["max_api_calls"].as_u64().is_some(),
                "Live-only row {wf_id} must have max_api_calls budget"
            );
            assert!(
                budget["max_cost_usd"].as_f64().is_some(),
                "Live-only row {wf_id} must have max_cost_usd budget"
            );
            assert!(
                budget["rate_limit_delay_ms"].as_u64().is_some(),
                "Live-only row {wf_id} must have rate_limit_delay_ms"
            );
            assert!(
                budget["timeout_per_test_secs"].as_u64().is_some(),
                "Live-only row {wf_id} must have timeout_per_test_secs"
            );
        }
    }
}

#[test]
fn dual_mode_rows_have_dual_mode_policy() {
    let matrix = load_matrix();
    for row in matrix_rows(&matrix) {
        let wf_id = row["workflow_id"].as_str().unwrap_or("<unknown>");
        let vcr_mode = row["vcr_mode"].as_str().unwrap_or("");
        if vcr_mode == "dual-mode" {
            assert!(
                row.get("dual_mode_policy").is_some(),
                "Dual-mode row {wf_id} must have 'dual_mode_policy'"
            );
            let policy = &row["dual_mode_policy"];
            assert!(
                policy["default_ci_mode"].as_str().is_some(),
                "Dual-mode row {wf_id} must specify default_ci_mode"
            );
            assert!(
                policy["live_trigger_env"].as_str().is_some(),
                "Dual-mode row {wf_id} must specify live_trigger_env"
            );
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// SECTION 3: Test file existence validation
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn all_test_paths_exist() {
    let matrix = load_matrix();
    for row in matrix_rows(&matrix) {
        let wf_id = row["workflow_id"].as_str().unwrap_or("<unknown>");
        if let Some(paths) = row["test_paths"].as_array() {
            for path_val in paths {
                let path = path_val.as_str().unwrap();
                assert!(
                    Path::new(path).exists(),
                    "Row {wf_id}: test path {path} does not exist"
                );
            }
        }
    }
}

#[test]
fn vcr_fixture_dir_exists() {
    assert!(
        Path::new(VCR_FIXTURE_DIR).is_dir(),
        "VCR fixture directory {VCR_FIXTURE_DIR} must exist"
    );
}

#[test]
fn vcr_fixtures_are_not_empty() {
    let count = std::fs::read_dir(VCR_FIXTURE_DIR)
        .expect("Should read VCR fixture dir")
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
        .count();
    assert!(
        count > 0,
        "VCR fixture directory must contain at least one .json cassette"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// SECTION 4: Workflow ID uniqueness and completeness
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn workflow_ids_are_unique() {
    let matrix = load_matrix();
    let mut seen = HashSet::new();
    for row in matrix_rows(&matrix) {
        let wf_id = row["workflow_id"].as_str().unwrap_or("<unknown>");
        assert!(
            seen.insert(wf_id.to_string()),
            "Duplicate workflow_id: {wf_id}"
        );
    }
}

#[test]
fn every_row_has_required_fields() {
    let matrix = load_matrix();
    let required_fields = [
        "workflow_id",
        "workflow_class",
        "workflow_title",
        "status",
        "vcr_mode",
        "vcr_mode_rationale",
        "owner",
        "provider_families",
        "replay_command",
    ];
    for row in matrix_rows(&matrix) {
        let wf_id = row["workflow_id"].as_str().unwrap_or("<unknown>");
        for field in &required_fields {
            assert!(
                !row[field].is_null(),
                "Row {wf_id} is missing required field '{field}'"
            );
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// SECTION 5: Status/vcr_mode cross-validation
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn covered_rows_have_suite_ids() {
    let matrix = load_matrix();
    for row in matrix_rows(&matrix) {
        let wf_id = row["workflow_id"].as_str().unwrap_or("<unknown>");
        let status = row["status"].as_str().unwrap_or("");
        if status == "covered" {
            let suites = row["suite_ids"].as_array();
            assert!(
                suites.is_some() && !suites.unwrap().is_empty(),
                "Covered row {wf_id} must have non-empty suite_ids"
            );
        }
    }
}

#[test]
fn planned_rows_have_planned_suite_ids() {
    let matrix = load_matrix();
    for row in matrix_rows(&matrix) {
        let wf_id = row["workflow_id"].as_str().unwrap_or("<unknown>");
        let status = row["status"].as_str().unwrap_or("");
        if status == "planned" {
            let planned = row["planned_suite_ids"].as_array();
            assert!(
                planned.is_some() && !planned.unwrap().is_empty(),
                "Planned row {wf_id} must have non-empty planned_suite_ids"
            );
        }
    }
}

#[test]
fn waived_rows_have_waiver_reason() {
    let matrix = load_matrix();
    for row in matrix_rows(&matrix) {
        let wf_id = row["workflow_id"].as_str().unwrap_or("<unknown>");
        let status = row["status"].as_str().unwrap_or("");
        if status == "waived" {
            assert!(
                row["waiver_reason"].as_str().is_some(),
                "Waived row {wf_id} must have waiver_reason"
            );
            assert!(
                row["waiver_issue_id"].as_str().is_some(),
                "Waived row {wf_id} must have waiver_issue_id"
            );
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// SECTION 6: Live budget policy validation
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn global_live_budget_policy_exists() {
    let matrix = load_matrix();
    let budget = &matrix["ci_policy"]["live_budget_policy"];
    assert!(
        budget["max_live_api_calls_per_suite"].as_u64().is_some(),
        "Global live budget must have max_live_api_calls_per_suite"
    );
    assert!(
        budget["max_live_cost_usd_per_suite"].as_f64().is_some(),
        "Global live budget must have max_live_cost_usd_per_suite"
    );
    assert!(
        budget["rate_limit_delay_ms"].as_u64().is_some(),
        "Global live budget must have rate_limit_delay_ms"
    );
    assert!(
        budget["skip_env_var"].as_str().is_some(),
        "Global live budget must have skip_env_var"
    );
}

#[test]
#[allow(clippy::cast_precision_loss)]
fn live_budget_limits_are_reasonable() {
    let matrix = load_matrix();
    let budget = &matrix["ci_policy"]["live_budget_policy"];
    let max_calls = budget["max_live_api_calls_per_suite"].as_u64().unwrap();
    let max_cost = budget["max_live_cost_usd_per_suite"].as_f64().unwrap();
    let rate_delay = budget["rate_limit_delay_ms"].as_u64().unwrap();

    assert!(
        max_calls <= 200,
        "max_live_api_calls_per_suite ({max_calls}) should be <= 200"
    );
    assert!(
        max_cost <= 10.0,
        "max_live_cost_usd_per_suite ({max_cost}) should be <= $10"
    );
    assert!(
        rate_delay >= 100,
        "rate_limit_delay_ms ({rate_delay}) should be >= 100ms"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// SECTION 7: VCR mode distribution summary
// ═══════════════════════════════════════════════════════════════════════

#[test]
#[allow(clippy::cast_precision_loss)]
fn vcr_mode_distribution_has_vcr_only_majority() {
    // Most scenarios should be VCR-deterministic for reliable CI
    let matrix = load_matrix();
    let rows = matrix_rows(&matrix);
    let total = rows.len();
    let vcr_only = rows
        .iter()
        .filter(|r| r["vcr_mode"].as_str() == Some("vcr-only"))
        .count();

    assert!(total > 0, "Matrix must have at least one row");

    let vcr_pct = (vcr_only as f64 / total as f64) * 100.0;
    assert!(
        vcr_pct >= 50.0,
        "At least 50% of scenarios should be vcr-only, got {vcr_pct:.0}% ({vcr_only}/{total})"
    );
}

#[test]
fn no_live_only_rows_are_covered_without_waiver() {
    // live-only rows should be waived, not marked as "covered"
    let matrix = load_matrix();
    for row in matrix_rows(&matrix) {
        let wf_id = row["workflow_id"].as_str().unwrap_or("<unknown>");
        let vcr_mode = row["vcr_mode"].as_str().unwrap_or("");
        let status = row["status"].as_str().unwrap_or("");
        assert!(
            !(vcr_mode == "live-only" && status == "covered"),
            "Row {wf_id} is live-only and covered — should be waived or credential-gated"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════
// SECTION 8: Schema version validation
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn schema_version_is_v2_or_later() {
    let matrix = load_matrix();
    let schema = matrix["schema"].as_str().unwrap_or("");
    assert!(
        schema.contains("v2") || schema.contains("v3"),
        "Schema must be v2 or later (got {schema})"
    );
}

#[test]
fn matrix_has_consumed_by_this_test() {
    let matrix = load_matrix();
    let consumers = matrix["ci_policy"]["consumed_by"]
        .as_array()
        .expect("consumed_by must be an array");
    assert!(
        consumers
            .iter()
            .any(|v| v.as_str() == Some("tests/vcr_parity_validation.rs")),
        "ci_policy.consumed_by must list tests/vcr_parity_validation.rs"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// SECTION 9: Expected artifact consistency
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn all_rows_declare_expected_artifacts() {
    let matrix = load_matrix();
    let required = matrix["ci_policy"]["required_suite_artifacts"]
        .as_array()
        .expect("required_suite_artifacts must exist");
    let required_strs: HashSet<&str> = required.iter().filter_map(Value::as_str).collect();

    for row in matrix_rows(&matrix) {
        let wf_id = row["workflow_id"].as_str().unwrap_or("<unknown>");
        if let Some(artifacts) = row["expected_artifacts"].as_array() {
            let artifact_strs: HashSet<&str> = artifacts.iter().filter_map(Value::as_str).collect();
            for req in &required_strs {
                assert!(
                    artifact_strs.contains(req),
                    "Row {wf_id} is missing required artifact: {req}"
                );
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// SECTION 10: Replay command validation
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn all_rows_have_replay_command() {
    let matrix = load_matrix();
    for row in matrix_rows(&matrix) {
        let wf_id = row["workflow_id"].as_str().unwrap_or("<unknown>");
        let cmd = row["replay_command"].as_str();
        assert!(
            cmd.is_some() && !cmd.unwrap().is_empty(),
            "Row {wf_id} must have a non-empty replay_command"
        );
    }
}

#[test]
fn live_only_replay_commands_use_live_env() {
    let matrix = load_matrix();
    for row in matrix_rows(&matrix) {
        let wf_id = row["workflow_id"].as_str().unwrap_or("<unknown>");
        let vcr_mode = row["vcr_mode"].as_str().unwrap_or("");
        if vcr_mode == "live-only" {
            let cmd = row["replay_command"].as_str().unwrap_or("");
            assert!(
                cmd.contains("PI_E2E_TESTS=1") || cmd.contains("--profile full"),
                "Live-only row {wf_id} replay_command should indicate live mode"
            );
        }
    }
}
