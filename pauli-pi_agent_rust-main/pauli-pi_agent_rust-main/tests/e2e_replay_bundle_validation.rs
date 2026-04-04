//! E2E: One-command replay bundle validation (bd-1f42.8.7).
//!
//! Tests proving replay manifests remain valid and that failed suites can
//! be reproduced from emitted artifacts with deterministic commands:
//!
//! 1. Summary.json schema: rerun-essential fields present and typed
//! 2. Failure digest replay commands: well-formed, point to real test targets
//! 3. Scenario matrix replay commands: all parseable and internally consistent
//! 4. Rerun-from pipeline: summary with failures extracts correct suite names
//! 5. Replay command templates: env/profile/shard context restoration
//! 6. Evidence contract: required artifacts enumerated per failure class
//! 7. Correlation ID propagation: summary → digest → timeline linkage
//!
//! Run:
//! ```bash
//! cargo test --test e2e_replay_bundle_validation
//! ```

#![allow(clippy::too_many_lines)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::items_after_statements)]

use serde_json::Value;

// ─── Constants ──────────────────────────────────────────────────────────────

const SCENARIO_MATRIX_PATH: &str = "docs/e2e_scenario_matrix.json";
const RUN_ALL_SCRIPT: &str = "scripts/e2e/run_all.sh";

fn load_matrix() -> Value {
    let content =
        std::fs::read_to_string(SCENARIO_MATRIX_PATH).expect("Should read scenario matrix");
    serde_json::from_str(&content).expect("Should parse scenario matrix JSON")
}

fn load_run_all_script() -> String {
    std::fs::read_to_string(RUN_ALL_SCRIPT).expect("Should read run_all.sh")
}

fn matrix_rows(matrix: &Value) -> Vec<&Value> {
    matrix["rows"]
        .as_array()
        .expect("rows array")
        .iter()
        .collect()
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 1: Summary schema validation
// ═══════════════════════════════════════════════════════════════════════════

/// Verify that run_all.sh emits summary.json with rerun-essential fields.
#[test]
fn summary_schema_has_rerun_essential_fields() {
    let script = load_run_all_script();

    // The summary.json template must include these fields for replay to work
    let required_fields = [
        "\"schema\":",
        "\"failed_names\":",
        "\"failed_suites\":",
        "\"profile\":",
        "\"correlation_id\":",
        "\"artifact_dir\":",
        "\"rerun_from\":",
        "\"shard\":",
    ];

    for field in &required_fields {
        assert!(
            script.contains(field),
            "summary.json template missing rerun-essential field: {field}"
        );
    }
}

/// Verify that the summary schema version is declared.
#[test]
fn summary_schema_version_declared() {
    let script = load_run_all_script();
    assert!(
        script.contains("pi.e2e.summary.v1"),
        "summary.json must declare schema version pi.e2e.summary.v1"
    );
}

/// Verify that --rerun-from flag is supported in run_all.sh.
#[test]
fn run_all_supports_rerun_from_flag() {
    let script = load_run_all_script();
    assert!(
        script.contains("--rerun-from"),
        "run_all.sh must support --rerun-from flag"
    );
}

/// Verify that --diff-from flag is supported in run_all.sh.
#[test]
fn run_all_supports_diff_from_flag() {
    let script = load_run_all_script();
    assert!(
        script.contains("--diff-from"),
        "run_all.sh must support --diff-from flag"
    );
}

/// Verify that differential runner output includes semantic and mirrored
/// scenario evidence sections suitable for CI gating.
#[test]
fn run_all_emits_semantic_mirrored_diff_sections() {
    let script = load_run_all_script();
    let required_markers = [
        "semantic_diffs",
        "mirrored_scenarios",
        "semantic_focus_commands",
        "pi.e2e.semantic_diff.v1",
        "pi.e2e.mirrored_scenarios.v1",
    ];
    for marker in &required_markers {
        assert!(
            script.contains(marker),
            "run_all.sh differential output missing marker: {marker}"
        );
    }
}

/// Verify that triage diff generation enforces deterministic ordering for
/// machine-readable output consumed by CI gates.
#[test]
fn run_all_enforces_deterministic_triage_diff_ordering() {
    let script = load_run_all_script();

    assert!(
        script.contains("all_names = sorted(set(baseline_index) | set(current_index))"),
        "triage diff should sort merged section keys for deterministic ordering"
    );
    assert!(
        script
            .contains("workflows.sort(key=lambda entry: str(entry.get(\"workflow_id\") or \"\"))"),
        "mirrored scenario workflows should be sorted by workflow_id"
    );
    assert!(
        script.contains("diagnostics.sort("),
        "ranked diagnostics should be sorted deterministically"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 2: Failure diagnostics infrastructure
// ═══════════════════════════════════════════════════════════════════════════

/// Verify failure digest generation function exists in run_all.sh.
#[test]
fn failure_digest_generation_exists() {
    let script = load_run_all_script();
    assert!(
        script.contains("generate_failure_diagnostics"),
        "run_all.sh must contain generate_failure_diagnostics function"
    );
}

/// Verify that failure_digest.json includes replay_command field.
#[test]
fn failure_digest_includes_replay_command() {
    let script = load_run_all_script();
    assert!(
        script.contains("replay_command"),
        "failure_digest should include replay_command in remediation_pointer"
    );
}

/// Verify that failure_digest.json includes three levels of replay:
/// suite_replay, targeted_test_replay, and full replay.
#[test]
fn failure_digest_has_three_replay_levels() {
    let script = load_run_all_script();
    assert!(
        script.contains("suite_replay_command"),
        "failure_digest must include suite_replay_command"
    );
    assert!(
        script.contains("targeted_test_replay_command"),
        "failure_digest must include targeted_test_replay_command"
    );
    assert!(
        script.contains("replay_command"),
        "failure_digest must include replay_command"
    );
}

/// Verify root cause classification covers expected failure classes.
#[test]
fn root_cause_classifier_covers_all_classes() {
    let script = load_run_all_script();
    let expected_classes = [
        "timeout",
        "assertion_failure",
        "permission_denied",
        "network_io",
        "missing_file",
        "panic",
        "unknown",
    ];
    for class in &expected_classes {
        assert!(
            script.contains(&format!("\"{class}\"")),
            "root cause classifier missing class: {class}"
        );
    }
}

/// Verify failure timeline JSONL schema is declared.
#[test]
fn failure_timeline_schema_declared() {
    let script = load_run_all_script();
    assert!(
        script.contains("failure_timeline"),
        "run_all.sh must generate failure_timeline artifacts"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 3: Scenario matrix replay commands
// ═══════════════════════════════════════════════════════════════════════════

/// Every matrix row must have a non-empty replay_command.
#[test]
fn all_matrix_rows_have_replay_command() {
    let matrix = load_matrix();
    for row in matrix_rows(&matrix) {
        let wf_id = row["workflow_id"].as_str().unwrap_or("unknown");
        let cmd = row["replay_command"].as_str();
        assert!(
            cmd.is_some_and(|c| !c.is_empty()),
            "workflow {wf_id}: replay_command must be non-empty"
        );
    }
}

/// All replay commands must reference the run_all.sh script.
#[test]
fn replay_commands_reference_run_all_script() {
    let matrix = load_matrix();
    for row in matrix_rows(&matrix) {
        let wf_id = row["workflow_id"].as_str().unwrap_or("unknown");
        let cmd = row["replay_command"].as_str().unwrap_or("");
        assert!(
            cmd.contains("run_all.sh"),
            "workflow {wf_id}: replay_command must reference run_all.sh, got: {cmd}"
        );
    }
}

/// Replay commands for covered rows must include --suite flags matching suite_ids.
#[test]
fn covered_replay_commands_include_suite_flags() {
    let matrix = load_matrix();
    for row in matrix_rows(&matrix) {
        let wf_id = row["workflow_id"].as_str().unwrap_or("unknown");
        let status = row["status"].as_str().unwrap_or("");
        if status != "covered" {
            continue;
        }

        let cmd = row["replay_command"].as_str().unwrap_or("");
        let suite_ids = row["suite_ids"]
            .as_array()
            .map(|a| a.iter().filter_map(Value::as_str).collect::<Vec<_>>())
            .unwrap_or_default();

        for suite in &suite_ids {
            assert!(
                cmd.contains(&format!("--suite {suite}")),
                "workflow {wf_id}: replay_command missing --suite {suite}"
            );
        }
    }
}

/// Planned rows should also have a replay command (for future use).
#[test]
fn planned_rows_have_replay_command() {
    let matrix = load_matrix();
    for row in matrix_rows(&matrix) {
        let wf_id = row["workflow_id"].as_str().unwrap_or("unknown");
        let status = row["status"].as_str().unwrap_or("");
        if status != "planned" {
            continue;
        }
        let cmd = row["replay_command"].as_str();
        assert!(
            cmd.is_some_and(|c| !c.is_empty()),
            "workflow {wf_id}: even planned rows must have replay_command"
        );
    }
}

/// Replay commands for live-only rows must include live env var prefix.
#[test]
fn live_only_replay_commands_use_live_env_prefix() {
    let matrix = load_matrix();
    for row in matrix_rows(&matrix) {
        let wf_id = row["workflow_id"].as_str().unwrap_or("unknown");
        let vcr_mode = row["vcr_mode"].as_str().unwrap_or("");
        if vcr_mode != "live-only" {
            continue;
        }
        let cmd = row["replay_command"].as_str().unwrap_or("");
        assert!(
            cmd.contains("PI_E2E_"),
            "workflow {wf_id}: live-only replay must set PI_E2E_ env vars, got: {cmd}"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 4: Rerun-from pipeline validation
// ═══════════════════════════════════════════════════════════════════════════

/// The run_all.sh script must parse failed_names from summary.json for rerun.
#[test]
fn rerun_from_parses_failed_names() {
    let script = load_run_all_script();
    assert!(
        script.contains("failed_names"),
        "run_all.sh rerun logic must read failed_names from summary"
    );
}

/// The rerun-from logic should set SELECTED_SUITES to failed names.
#[test]
fn rerun_from_sets_selected_suites() {
    let script = load_run_all_script();
    // The rerun logic should update suite selection
    assert!(
        script.contains("SELECTED_SUITES"),
        "run_all.sh rerun must populate SELECTED_SUITES"
    );
}

/// Simulate summary.json with failures and verify structure is parseable.
#[test]
fn synthetic_summary_with_failures_is_valid() {
    let summary = serde_json::json!({
        "schema": "pi.e2e.summary.v1",
        "timestamp": "2026-02-13T00:00:00Z",
        "profile": "ci",
        "rerun_from": null,
        "diff_from": null,
        "artifact_dir": "/tmp/test-artifacts",
        "correlation_id": "test-replay-001",
        "shard": {
            "kind": "none",
            "name": "default",
            "index": null,
            "total": null
        },
        "total_suites": 3,
        "passed_suites": 1,
        "failed_suites": 2,
        "failed_names": ["e2e_provider_scenarios", "e2e_extension_registration"],
        "total_units": 0,
        "passed_units": 0,
        "failed_units": 0,
        "failed_unit_names": [],
        "suites": [],
        "unit_targets": []
    });

    // Verify failed_names extraction
    let failed = summary["failed_names"]
        .as_array()
        .expect("failed_names should be array");
    assert_eq!(failed.len(), 2);
    assert_eq!(failed[0].as_str().unwrap(), "e2e_provider_scenarios");
    assert_eq!(failed[1].as_str().unwrap(), "e2e_extension_registration");

    // Verify rerun command can be constructed
    let base_cmd = "./scripts/e2e/run_all.sh --profile ci";
    let mut rerun_cmd = base_cmd.to_string();
    for suite in failed.iter().filter_map(Value::as_str) {
        rerun_cmd.push_str(" --suite ");
        rerun_cmd.push_str(suite);
    }
    assert!(rerun_cmd.contains("--suite e2e_provider_scenarios"));
    assert!(rerun_cmd.contains("--suite e2e_extension_registration"));
}

/// Verify that rerun_from field in summary points back to source.
#[test]
fn summary_rerun_from_field_supports_chaining() {
    let summary_with_rerun = serde_json::json!({
        "schema": "pi.e2e.summary.v1",
        "rerun_from": "/tmp/prior-run/summary.json",
        "failed_names": ["e2e_tools"],
    });

    // When rerunning, the summary should record what it was rerun from
    assert!(
        summary_with_rerun["rerun_from"].is_string(),
        "rerun_from should record the source summary path"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 5: Replay command templates
// ═══════════════════════════════════════════════════════════════════════════

/// Verify all three replay levels produce valid command templates.
#[test]
fn replay_command_templates_are_well_formed() {
    let suites = ["e2e_agent_loop", "e2e_provider_scenarios", "e2e_tui"];

    for suite in &suites {
        // Level 1: Full runner replay
        let cmd1 =
            format!("./scripts/e2e/run_all.sh --profile focused --skip-lint --suite {suite}");
        assert!(cmd1.contains("--profile"), "missing --profile in level 1");
        assert!(cmd1.contains("--suite"), "missing --suite in level 1");

        // Level 2: Cargo test replay
        let cmd2 = format!("cargo test --test {suite} -- --nocapture");
        assert!(cmd2.contains("--test"), "missing --test in level 2");
        assert!(
            cmd2.contains("--nocapture"),
            "missing --nocapture in level 2"
        );

        // Level 3: Targeted test replay
        let test_name = "simple_conversation";
        let cmd3 = format!("cargo test --test {suite} {test_name} -- --nocapture");
        assert!(cmd3.contains(test_name), "missing test name in level 3");
    }
}

/// Verify replay commands include profile context.
#[test]
fn replay_commands_preserve_profile_context() {
    let matrix = load_matrix();
    for row in matrix_rows(&matrix) {
        let cmd = row["replay_command"].as_str().unwrap_or("");
        assert!(
            cmd.contains("--profile") || cmd.contains("--suite"),
            "replay command should specify execution context: {cmd}"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 6: Evidence contract per failure class
// ═══════════════════════════════════════════════════════════════════════════

/// Verify the failure diagnostics index artifact is generated.
#[test]
fn failure_diagnostics_index_generation() {
    let script = load_run_all_script();
    assert!(
        script.contains("failure_diagnostics_index.json"),
        "run_all.sh must generate failure_diagnostics_index.json"
    );
}

/// Verify that per-suite failure_digest.json includes required artifact paths.
#[test]
fn failure_digest_includes_artifact_paths() {
    let script = load_run_all_script();
    // The failure digest should reference key artifacts for replay
    let expected_artifact_refs = ["result.json", "output.log"];
    for artifact in &expected_artifact_refs {
        assert!(
            script.contains(artifact),
            "failure_digest should reference {artifact}"
        );
    }
}

/// Verify remediation summaries exist for all root cause classes.
#[test]
fn remediation_summaries_cover_all_root_causes() {
    let script = load_run_all_script();
    assert!(
        script.contains("remediation_summary"),
        "run_all.sh must contain remediation_summary function"
    );

    // Each root cause class should have a remediation message
    let classes = [
        "timeout",
        "assertion_failure",
        "permission_denied",
        "network_io",
        "missing_file",
        "panic",
    ];
    for class in &classes {
        assert!(
            script.contains(class),
            "remediation_summary must handle class: {class}"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 7: Correlation ID propagation
// ═══════════════════════════════════════════════════════════════════════════

/// Verify correlation_id is generated in run_all.sh.
#[test]
fn correlation_id_is_generated() {
    let script = load_run_all_script();
    assert!(
        script.contains("CORRELATION_ID"),
        "run_all.sh must generate a CORRELATION_ID"
    );
}

/// Verify correlation_id appears in summary.json template.
#[test]
fn correlation_id_in_summary_template() {
    let script = load_run_all_script();
    assert!(
        script.contains("\"correlation_id\""),
        "summary.json must include correlation_id field"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 8: Comprehensive replay bundle structure test
// ═══════════════════════════════════════════════════════════════════════════

/// End-to-end: build a synthetic failure digest and validate its structure.
#[test]
fn synthetic_failure_digest_structure_valid() {
    let digest = serde_json::json!({
        "schema": "pi.e2e.failure_digest.v1",
        "suite": "e2e_provider_scenarios",
        "exit_code": 1,
        "root_cause_class": "assertion_failure",
        "root_cause_message": "assertion `left == right` failed",
        "remediation_summary": "Inspect assertion preconditions and fixture state.",
        "impacted_tests": ["provider_streaming_round_trip", "provider_error_handling"],
        "remediation_pointer": {
            "replay_command": "./scripts/e2e/run_all.sh --profile focused --skip-lint --suite e2e_provider_scenarios",
            "suite_replay_command": "cargo test --test e2e_provider_scenarios -- --nocapture",
            "targeted_test_replay_command": "cargo test --test e2e_provider_scenarios provider_streaming_round_trip -- --nocapture"
        },
        "artifact_paths": {
            "result": "e2e_provider_scenarios/result.json",
            "output": "e2e_provider_scenarios/output.log",
            "test_log": "e2e_provider_scenarios/test-log.jsonl",
            "artifact_index": "e2e_provider_scenarios/artifact-index.jsonl"
        },
        "first_assertion": {
            "thread": "main",
            "file": "tests/e2e_provider_scenarios.rs",
            "line": 142,
            "column": 5,
            "message": "assertion `left == right` failed"
        }
    });

    // Validate structure
    assert_eq!(
        digest["schema"].as_str().unwrap(),
        "pi.e2e.failure_digest.v1"
    );
    assert!(digest["exit_code"].as_u64().unwrap() > 0);
    assert!(digest["root_cause_class"].is_string());
    assert!(!digest["impacted_tests"].as_array().unwrap().is_empty());

    // Validate replay commands
    let rp = &digest["remediation_pointer"];
    assert!(
        rp["replay_command"]
            .as_str()
            .unwrap()
            .contains("run_all.sh")
    );
    assert!(
        rp["suite_replay_command"]
            .as_str()
            .unwrap()
            .contains("cargo test")
    );
    assert!(
        rp["targeted_test_replay_command"]
            .as_str()
            .unwrap()
            .contains("provider_streaming_round_trip")
    );

    // Validate artifact paths
    let ap = &digest["artifact_paths"];
    assert!(ap["result"].as_str().unwrap().ends_with("result.json"));
    assert!(ap["output"].as_str().unwrap().ends_with("output.log"));
}

/// Validate that a synthetic failure diagnostics index aggregates correctly.
#[test]
fn synthetic_diagnostics_index_aggregates_failures() {
    let index = serde_json::json!({
        "schema": "pi.e2e.failure_diagnostics_index.v1",
        "generated_at": "2026-02-13T00:00:00Z",
        "correlation_id": "test-replay-001",
        "total_failed_suites": 2,
        "root_cause_distribution": {
            "assertion_failure": 1,
            "timeout": 1
        },
        "digests": [
            {
                "suite": "e2e_provider_scenarios",
                "root_cause_class": "assertion_failure",
                "impacted_test_count": 2
            },
            {
                "suite": "e2e_tui",
                "root_cause_class": "timeout",
                "impacted_test_count": 1
            }
        ]
    });

    let digests = index["digests"].as_array().unwrap();
    assert_eq!(digests.len(), 2);
    assert_eq!(
        index["total_failed_suites"].as_u64().unwrap(),
        digests.len() as u64
    );

    // Correlation ID should be propagated
    assert!(index["correlation_id"].is_string());
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 9: Cross-reference matrix ↔ test targets
// ═══════════════════════════════════════════════════════════════════════════

/// Replay commands reference suites that exist as test targets.
#[test]
fn replay_command_suites_are_valid_test_targets() {
    let matrix = load_matrix();
    for row in matrix_rows(&matrix) {
        let wf_id = row["workflow_id"].as_str().unwrap_or("unknown");
        let cmd = row["replay_command"].as_str().unwrap_or("");

        // Extract suite names from --suite flags
        let suites: Vec<&str> = cmd
            .split("--suite ")
            .skip(1)
            .map(|s| s.split_whitespace().next().unwrap_or(""))
            .filter(|s| !s.is_empty())
            .collect();

        for suite in &suites {
            // Each suite should have a corresponding test file
            let test_path = format!("tests/{suite}.rs");
            let exists = std::path::Path::new(&test_path).exists();
            // Allow planned suites that don't exist yet
            let status = row["status"].as_str().unwrap_or("");
            if status == "planned" {
                continue;
            }
            assert!(
                exists,
                "workflow {wf_id}: replay suite '{suite}' has no test file at {test_path}"
            );
        }
    }
}

/// Matrix test_paths match the suites in replay_command.
#[test]
fn matrix_test_paths_consistent_with_replay_suites() {
    let matrix = load_matrix();
    for row in matrix_rows(&matrix) {
        let wf_id = row["workflow_id"].as_str().unwrap_or("unknown");
        let status = row["status"].as_str().unwrap_or("");
        if status == "planned" {
            continue;
        }

        let cmd = row["replay_command"].as_str().unwrap_or("");
        let test_paths: Vec<&str> = row["test_paths"]
            .as_array()
            .map(|a| a.iter().filter_map(Value::as_str).collect())
            .unwrap_or_default();

        // Every test_path's stem should appear as --suite in the replay command
        for path in &test_paths {
            let stem = std::path::Path::new(path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("");
            assert!(
                cmd.contains(stem),
                "workflow {wf_id}: test_path {path} (stem={stem}) not in replay_command: {cmd}"
            );
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 10: Run_all.sh artifact structure
// ═══════════════════════════════════════════════════════════════════════════

/// Verify run_all.sh emits required artifacts per suite.
#[test]
fn run_all_emits_required_per_suite_artifacts() {
    let script = load_run_all_script();
    let required = [
        "result.json",
        "output.log",
        "test-log.jsonl",
        "artifact-index.jsonl",
    ];
    for artifact in &required {
        assert!(
            script.contains(artifact),
            "run_all.sh must emit per-suite {artifact}"
        );
    }
}

/// Verify run_all.sh emits required per-run artifacts.
#[test]
fn run_all_emits_required_per_run_artifacts() {
    let script = load_run_all_script();
    let required = ["summary.json", "environment.json", "evidence_contract.json"];
    for artifact in &required {
        assert!(
            script.contains(artifact),
            "run_all.sh must emit per-run {artifact}"
        );
    }
}

/// Verify evidence_contract.json generation exists.
#[test]
fn evidence_contract_generation_exists() {
    let script = load_run_all_script();
    assert!(
        script.contains("evidence_contract"),
        "run_all.sh must generate evidence_contract.json"
    );
}

/// Verify that redaction is applied to replay artifacts.
#[test]
fn replay_artifacts_are_redacted() {
    let script = load_run_all_script();
    assert!(
        script.contains("redact_secrets") || script.contains("redact"),
        "run_all.sh must redact secrets from replay artifacts"
    );
}
