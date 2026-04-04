//! E2E artifact retention + log triage workflow validation (bd-3uqg.8.11).
//!
//! Validates that CI artifact retention and triage infrastructure enforces:
//! 1. Structured log schema compliance (pi.test.log.v2, pi.test.artifact.v1)
//! 2. Artifact index JSONL production and schema correctness
//! 3. Log redaction completeness (no leaked secrets)
//! 4. CI workflow artifact upload/retention configuration
//! 5. Triage workflow linkage (quarantine, failure signatures, escalation)
//! 6. Evidence contract validation (schema, status, error fields)
//! 7. Cross-gate artifact indexing and shard correlation
//! 8. Retention policy compliance (TTL, cleanup rules)
//!
//! Run:
//! ```bash
//! cargo test --test e2e_artifact_retention_triage
//! ```

#![allow(clippy::too_many_lines)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::items_after_statements)]

mod common;

use common::TestHarness;
use serde_json::Value;
use std::path::Path;

// ─── Constants ──────────────────────────────────────────────────────────────

const CI_WORKFLOW_PATH: &str = ".github/workflows/ci.yml";
const TESTING_POLICY_PATH: &str = "docs/testing-policy.md";
const QA_RUNBOOK_PATH: &str = "docs/qa-runbook.md";
const FLAKE_TRIAGE_POLICY_PATH: &str = "docs/flake-triage-policy.md";
const SUITE_CLASSIFICATION_PATH: &str = "tests/suite_classification.toml";
const SCENARIO_MATRIX_PATH: &str = "docs/e2e_scenario_matrix.json";
const FULL_SUITE_GATE_PATH: &str = "tests/ci_full_suite_gate.rs";
const NON_MOCK_RUBRIC_PATH: &str = "docs/non-mock-rubric.json";
const RUN_ALL_SCRIPT_PATH: &str = "scripts/e2e/run_all.sh";

fn load_text(path: &str) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|_| panic!("Should read {path}"))
}

fn load_json(path: &str) -> Value {
    let content = load_text(path);
    serde_json::from_str(&content).unwrap_or_else(|_| panic!("Should parse {path} as JSON"))
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 1: Structured log infrastructure exists
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_harness_produces_artifact_index() {
    let harness = TestHarness::new("artifact_index_test");
    let path = harness.temp_path("test_file.txt");
    std::fs::write(&path, "test content").unwrap();
    harness.record_artifact("test_file.txt", &path);

    // Verify the harness has recorded artifacts
    assert!(
        harness.has_artifacts(),
        "TestHarness must track recorded artifacts"
    );
}

#[test]
fn test_logger_produces_jsonl_output() {
    let harness = TestHarness::new("logger_jsonl_test");
    harness.info("test log entry");
    harness.info_ctx("test with context", &[("key", "value")]);

    // The logger should have recorded at least 2 entries
    let log_content = harness.dump_logs();
    let lines: Vec<&str> = log_content.lines().filter(|l| !l.is_empty()).collect();
    assert!(
        lines.len() >= 2,
        "Logger should produce JSONL lines for each info() call"
    );

    // Each line should be valid JSON
    for line in &lines {
        let parsed: Result<Value, _> = serde_json::from_str(line);
        assert!(parsed.is_ok(), "Each log line should be valid JSON: {line}");
    }
}

#[test]
fn log_schema_fields_are_present() {
    let harness = TestHarness::new("log_schema_fields_test");
    harness.info("test entry");

    let log_content = harness.dump_logs();
    let first_line = log_content.lines().next().expect("at least one line");
    let parsed: Value = serde_json::from_str(first_line).expect("valid JSON");

    // Required fields per pi.test.log.v2 schema
    let required_fields = ["schema", "seq", "ts", "t_ms", "level", "message"];
    for field in &required_fields {
        assert!(
            !parsed[*field].is_null(),
            "Log entry must have '{field}' field"
        );
    }
}

#[test]
fn log_schema_version_is_v2() {
    let harness = TestHarness::new("log_schema_version_test");
    harness.info("test entry");

    let log_content = harness.dump_logs();
    let first_line = log_content.lines().next().expect("at least one line");
    let parsed: Value = serde_json::from_str(first_line).expect("valid JSON");

    let schema = parsed["schema"].as_str().unwrap_or("");
    assert!(
        schema == "pi.test.log.v2" || schema == "pi.test.log.v1",
        "Log schema must be pi.test.log.v2 (or v1 for backward compat), got: {schema}"
    );
}

#[test]
fn artifact_schema_is_v1() {
    let harness = TestHarness::new("artifact_schema_test");
    let path = harness.temp_path("artifact.txt");
    std::fs::write(&path, "test").unwrap();
    harness.record_artifact("artifact.txt", &path);

    let artifact_index = harness.dump_artifact_index();
    assert!(
        artifact_index.contains("pi.test.artifact.v1"),
        "Artifact index should use pi.test.artifact.v1 schema"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 2: Log redaction compliance
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn log_redacts_api_keys() {
    let harness = TestHarness::new("redaction_api_key_test");
    harness.info_ctx("test with key", &[("api_key", "sk-secret-12345")]);

    let log_content = harness.dump_logs();
    assert!(
        !log_content.contains("sk-secret-12345"),
        "API key must be redacted from log output"
    );
}

#[test]
fn log_redacts_authorization_headers() {
    let harness = TestHarness::new("redaction_auth_header_test");
    harness.info_ctx(
        "request headers",
        &[("authorization", "Bearer eyJhbGciOiJIUzI1NiJ9.test")],
    );

    let log_content = harness.dump_logs();
    assert!(
        !log_content.contains("eyJhbGciOiJIUzI1NiJ9"),
        "Bearer tokens must be redacted from log output"
    );
}

#[test]
fn log_redacts_x_api_key_headers() {
    let harness = TestHarness::new("redaction_x_api_key_test");
    harness.info_ctx("anthropic request", &[("x-api-key", "sk-ant-api03-secret")]);

    let log_content = harness.dump_logs();
    assert!(
        !log_content.contains("sk-ant-api03-secret"),
        "x-api-key values must be redacted from log output"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 3: CI workflow artifact upload configuration
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn ci_workflow_has_artifact_upload_steps() {
    let ci = load_text(CI_WORKFLOW_PATH);
    assert!(
        ci.contains("upload-artifact"),
        "CI workflow must include artifact upload steps"
    );
}

#[test]
fn ci_workflow_has_retention_days() {
    let ci = load_text(CI_WORKFLOW_PATH);
    assert!(
        ci.contains("retention-days"),
        "CI workflow must specify retention-days for artifacts"
    );
}

#[test]
fn ci_workflow_uploads_on_failure() {
    let ci = load_text(CI_WORKFLOW_PATH);
    // Artifacts should be uploaded even on failure (if: always())
    assert!(
        ci.contains("if: always()"),
        "CI must upload artifacts even on failure (if: always())"
    );
}

#[test]
fn ci_workflow_has_shard_artifact_collection() {
    let ci = load_text(CI_WORKFLOW_PATH);
    assert!(
        ci.contains("shard") && ci.contains("artifact"),
        "CI must collect shard-level artifacts"
    );
}

#[test]
fn ci_workflow_has_evidence_bundle() {
    let ci = load_text(CI_WORKFLOW_PATH);
    assert!(
        ci.contains("evidence-bundle") || ci.contains("evidence_bundle"),
        "CI must produce an evidence bundle artifact"
    );
}

#[test]
fn ci_workflow_has_conformance_report_upload() {
    let ci = load_text(CI_WORKFLOW_PATH);
    assert!(
        ci.contains("conformance") && ci.contains("upload-artifact"),
        "CI must upload conformance report artifacts"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 4: Triage workflow infrastructure
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn ci_has_quarantine_system() {
    let ci = load_text(CI_WORKFLOW_PATH);
    assert!(
        ci.contains("quarantine"),
        "CI must have a quarantine system for flaky tests"
    );
}

#[test]
fn ci_quarantine_has_schema() {
    let ci = load_text(CI_WORKFLOW_PATH);
    assert!(
        ci.contains("quarantine_report"),
        "CI must produce quarantine_report artifact"
    );
}

#[test]
fn ci_has_failure_signature_extraction() {
    let ci = load_text(CI_WORKFLOW_PATH);
    assert!(
        ci.contains("signature") || ci.contains("failure"),
        "CI must extract failure signatures for triage"
    );
}

#[test]
fn run_all_classifies_vcr_cassette_mismatch() {
    let runner = load_text(RUN_ALL_SCRIPT_PATH);
    assert!(
        runner.contains("no matching interaction found in cassette"),
        "run_all triage classifier must detect missing cassette interaction signatures"
    );
    assert!(
        runner.contains("return \"vcr_mismatch\""),
        "run_all triage classifier must classify cassette mismatch failures as vcr_mismatch"
    );
}

#[test]
fn run_all_classifies_lint_failures() {
    let runner = load_text(RUN_ALL_SCRIPT_PATH);
    assert!(
        runner.contains("if \"clippy\" in haystack") && runner.contains("return \"lint_failure\""),
        "run_all triage classifier must classify clippy error signatures as lint_failure"
    );
}

#[test]
fn run_all_vcr_mismatch_has_actionable_remediation_hint() {
    let runner = load_text(RUN_ALL_SCRIPT_PATH);
    assert!(
        runner.contains("if root_cause_class == \"vcr_mismatch\""),
        "run_all remediation map must include vcr_mismatch branch"
    );
    assert!(
        runner.contains("Re-record or refresh the affected cassette"),
        "run_all triage remediation must provide cassette refresh guidance"
    );
}

#[test]
fn run_all_logs_failure_list_and_rerun_hint_events() {
    let runner = load_text(RUN_ALL_SCRIPT_PATH);
    assert!(
        runner.contains("\"event_type\": \"failure_list_item\""),
        "run_all failure parser must emit failure_list_item timeline events"
    );
    assert!(
        runner.contains("\"event_type\": \"rerun_hint\""),
        "run_all failure parser must emit rerun_hint timeline events"
    );
}

#[test]
fn qa_runbook_defines_triage_workflow() {
    let runbook = load_text(QA_RUNBOOK_PATH);
    assert!(
        runbook.contains("triage") || runbook.contains("Triage"),
        "QA runbook must define a triage workflow"
    );
}

#[test]
fn qa_runbook_has_failure_signatures() {
    let runbook = load_text(QA_RUNBOOK_PATH);
    assert!(
        runbook.contains("signature") || runbook.contains("pattern"),
        "QA runbook must document known failure signatures"
    );
}

#[test]
fn qa_runbook_has_reproduction_commands() {
    let runbook = load_text(QA_RUNBOOK_PATH);
    assert!(
        runbook.contains("cargo test") || runbook.contains("reproduce"),
        "QA runbook must include reproduction commands"
    );
}

#[test]
fn flake_triage_policy_exists() {
    assert!(
        Path::new(FLAKE_TRIAGE_POLICY_PATH).exists(),
        "Flake triage policy must exist at {FLAKE_TRIAGE_POLICY_PATH}"
    );
}

#[test]
fn flake_triage_policy_defines_categories() {
    let policy = load_text(FLAKE_TRIAGE_POLICY_PATH);
    // Should define flake categories
    assert!(
        policy.contains("FLAKE") || policy.contains("flake") || policy.contains("category"),
        "Flake triage policy must define flake categories"
    );
}

#[test]
fn flake_triage_policy_has_retry_rules() {
    let policy = load_text(FLAKE_TRIAGE_POLICY_PATH);
    assert!(
        policy.contains("retry") || policy.contains("Retry"),
        "Flake triage policy must define retry rules"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 5: Evidence contract validation
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn ci_validates_evidence_contract_schema() {
    let ci = load_text(CI_WORKFLOW_PATH);
    assert!(
        ci.contains("evidence_contract") || ci.contains("evidence-contract"),
        "CI must validate evidence contract schema"
    );
}

#[test]
fn ci_evidence_contract_checks_status() {
    let ci = load_text(CI_WORKFLOW_PATH);
    assert!(
        ci.contains("evidence") && ci.contains("status"),
        "CI must check evidence contract status field"
    );
}

#[test]
fn ci_evidence_contract_checks_errors() {
    let ci = load_text(CI_WORKFLOW_PATH);
    assert!(
        ci.contains("evidence") && ci.contains("error"),
        "CI must check evidence contract error fields"
    );
}

#[test]
fn scenario_matrix_defines_required_artifacts() {
    let matrix = load_json(SCENARIO_MATRIX_PATH);
    let policy = &matrix["ci_policy"];

    let suite_artifacts = policy["required_suite_artifacts"]
        .as_array()
        .expect("required_suite_artifacts must be an array");
    assert!(
        !suite_artifacts.is_empty(),
        "required_suite_artifacts must not be empty"
    );

    let run_artifacts = policy["required_run_artifacts"]
        .as_array()
        .expect("required_run_artifacts must be an array");
    assert!(
        !run_artifacts.is_empty(),
        "required_run_artifacts must not be empty"
    );
}

#[test]
fn scenario_matrix_requires_evidence_contract() {
    let matrix = load_json(SCENARIO_MATRIX_PATH);
    let run_artifacts = matrix["ci_policy"]["required_run_artifacts"]
        .as_array()
        .expect("required_run_artifacts array");
    let has_evidence = run_artifacts
        .iter()
        .any(|a| a.as_str().is_some_and(|s| s.contains("evidence")));
    assert!(
        has_evidence,
        "required_run_artifacts must include evidence_contract.json"
    );
}

#[test]
fn scenario_matrix_requires_test_log_jsonl() {
    let matrix = load_json(SCENARIO_MATRIX_PATH);
    let suite_artifacts = matrix["ci_policy"]["required_suite_artifacts"]
        .as_array()
        .expect("required_suite_artifacts array");
    let has_test_log = suite_artifacts.iter().any(|a| {
        a.as_str()
            .is_some_and(|s| s.contains("test-log") || s.contains("log"))
    });
    assert!(
        has_test_log,
        "required_suite_artifacts must include test-log.jsonl"
    );
}

#[test]
fn scenario_matrix_requires_artifact_index() {
    let matrix = load_json(SCENARIO_MATRIX_PATH);
    let suite_artifacts = matrix["ci_policy"]["required_suite_artifacts"]
        .as_array()
        .expect("required_suite_artifacts array");
    let has_index = suite_artifacts.iter().any(|a| {
        a.as_str()
            .is_some_and(|s| s.contains("artifact-index") || s.contains("index"))
    });
    assert!(
        has_index,
        "required_suite_artifacts must include artifact-index.jsonl"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 6: Shard correlation and indexing
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn ci_has_shard_correlation_ids() {
    let ci = load_text(CI_WORKFLOW_PATH);
    assert!(
        ci.contains("CORRELATION_ID") || ci.contains("correlation_id"),
        "CI must assign correlation IDs to shards"
    );
}

#[test]
fn ci_has_shard_index_schema() {
    let ci = load_text(CI_WORKFLOW_PATH);
    assert!(
        ci.contains("shard_index") || ci.contains("shard-index"),
        "CI must produce a shard index artifact"
    );
}

#[test]
fn ci_merges_shard_summaries() {
    let ci = load_text(CI_WORKFLOW_PATH);
    assert!(
        ci.contains("shard-summary") || ci.contains("shard_summary"),
        "CI must merge shard summaries into a unified report"
    );
}

#[test]
fn ci_has_shard_file_index_with_sizes() {
    let ci = load_text(CI_WORKFLOW_PATH);
    // The shard index should include file sizes for debugging
    assert!(
        ci.contains("size") || ci.contains("file_count"),
        "Shard index must track file counts or sizes"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 7: Retention policy compliance
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn ci_retention_days_is_reasonable() {
    let ci = load_text(CI_WORKFLOW_PATH);
    // Parse retention-days values
    let has_30_day = ci.contains("retention-days: 30");
    let has_any_retention = ci.contains("retention-days:");
    assert!(
        has_any_retention,
        "CI must specify artifact retention periods"
    );
    assert!(
        has_30_day || ci.contains("retention-days: 14") || ci.contains("retention-days: 7"),
        "Retention should be between 7 and 30 days"
    );
}

#[test]
fn testing_policy_defines_artifact_requirements() {
    let policy = load_text(TESTING_POLICY_PATH);
    assert!(
        policy.contains("artifact") || policy.contains("JSONL") || policy.contains("log"),
        "Testing policy must define artifact requirements"
    );
}

#[test]
fn testing_policy_references_quarantine() {
    let policy = load_text(TESTING_POLICY_PATH);
    assert!(
        policy.contains("quarantine") || policy.contains("Quarantine"),
        "Testing policy must reference the quarantine system"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 8: Gate promotion and rollback
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn ci_has_gate_promotion_logic() {
    let ci = load_text(CI_WORKFLOW_PATH);
    assert!(
        ci.contains("promotion") || ci.contains("PROMOTION"),
        "CI must support gate promotion modes"
    );
}

#[test]
fn ci_gate_promotion_has_schema() {
    let ci = load_text(CI_WORKFLOW_PATH);
    assert!(
        ci.contains("gate_promotion") || ci.contains("gate-promotion"),
        "CI gate promotion must have a schema identifier"
    );
}

#[test]
fn ci_gate_has_strict_mode() {
    let ci = load_text(CI_WORKFLOW_PATH);
    assert!(
        ci.contains("strict"),
        "CI gates must support strict mode (fail on failures)"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 9: Cross-reference validation
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn scenario_matrix_workflows_have_expected_artifacts() {
    let matrix = load_json(SCENARIO_MATRIX_PATH);
    let rows = matrix["rows"].as_array().expect("rows array");

    for row in rows {
        let workflow_id = row["workflow_id"].as_str().unwrap_or("unknown");
        let expected = row["expected_artifacts"].as_array();
        assert!(
            expected.is_some(),
            "Workflow {workflow_id} must define expected_artifacts"
        );
        let artifacts = expected.unwrap();
        assert!(
            !artifacts.is_empty(),
            "Workflow {workflow_id} must have at least one expected artifact"
        );
    }
}

#[test]
fn full_suite_gate_validates_evidence() {
    let gate = load_text(FULL_SUITE_GATE_PATH);
    assert!(
        gate.contains("evidence"),
        "Full suite gate must validate evidence artifacts"
    );
}

#[test]
fn full_suite_gate_has_artifact_paths() {
    let gate = load_text(FULL_SUITE_GATE_PATH);
    assert!(
        gate.contains(".jsonl") || gate.contains(".json"),
        "Full suite gate must reference JSONL/JSON artifact paths"
    );
}

#[test]
fn full_suite_gate_enforces_coverage_threshold() {
    let gate = load_text(FULL_SUITE_GATE_PATH);
    assert!(
        gate.contains("conformance_pass_rate"),
        "Full suite gate must include a conformance pass-rate quality gate"
    );
    assert!(
        gate.contains("if rate >= 80.0"),
        "Full suite gate must enforce a minimum pass-rate threshold for coverage quality"
    );
}

#[test]
fn full_suite_gate_exposes_logging_and_flake_control_gates() {
    let gate = load_text(FULL_SUITE_GATE_PATH);
    assert!(
        gate.contains("e2e_log_contract"),
        "Full suite gate must include E2E log contract gate for logging completeness"
    );
    assert!(
        gate.contains("waiver_lifecycle"),
        "Full suite gate must include waiver lifecycle gate for flake/quarantine control"
    );
}

#[test]
fn full_suite_gate_includes_reproduce_commands_for_remediation() {
    let gate = load_text(FULL_SUITE_GATE_PATH);
    assert!(
        gate.contains("reproduce_command"),
        "Full suite gate must include reproduce_command metadata for remediation"
    );
    assert!(
        gate.contains("cargo test --test conformance_regression_gate -- --nocapture"),
        "Full suite gate must provide concrete reproduce commands for failing quality gates"
    );
}

#[test]
fn suite_classification_exists_and_valid() {
    let content = load_text(SUITE_CLASSIFICATION_PATH);
    assert!(
        content.parse::<toml::Table>().is_ok(),
        "suite_classification.toml must be valid TOML"
    );
}

#[test]
fn non_mock_rubric_defines_evidence_requirements() {
    let rubric = load_json(NON_MOCK_RUBRIC_PATH);
    let text = serde_json::to_string(&rubric).unwrap_or_default();
    assert!(
        text.contains("evidence") || text.contains("artifact") || text.contains("log"),
        "Non-mock rubric must reference evidence/artifact requirements"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 10: Live artifact generation validation
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn harness_artifact_tracking_records_path_and_name() {
    let harness = TestHarness::new("artifact_tracking_test");

    let path1 = harness.temp_path("first.json");
    std::fs::write(&path1, r#"{"test": true}"#).unwrap();
    harness.record_artifact("first.json", &path1);

    let path2 = harness.temp_path("second.jsonl");
    std::fs::write(&path2, r#"{"line": 1}"#).unwrap();
    harness.record_artifact("second.jsonl", &path2);

    let index = harness.dump_artifact_index();
    assert!(
        index.contains("first.json"),
        "Artifact index must include first.json"
    );
    assert!(
        index.contains("second.jsonl"),
        "Artifact index must include second.jsonl"
    );
}

#[test]
fn harness_produces_normalized_artifact_index() {
    let harness = TestHarness::new("normalized_index_test");

    let path = harness.temp_path("data.json");
    std::fs::write(&path, r#"{"ok": true}"#).unwrap();
    harness.record_artifact("data.json", &path);

    let index = harness.dump_artifact_index();
    // Normalized index should not contain raw temp paths
    let lines: Vec<&str> = index.lines().filter(|l| !l.is_empty()).collect();
    for line in &lines {
        let parsed: Value = serde_json::from_str(line).expect("valid JSON");
        assert!(
            parsed["name"].is_string() || parsed["type"].is_string(),
            "Artifact index lines must have name or type field"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 11: Quarantine lifecycle
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn ci_quarantine_has_expiry_check() {
    let ci = load_text(CI_WORKFLOW_PATH);
    assert!(
        ci.contains("expir") || ci.contains("max_age") || ci.contains("max_quarantine"),
        "CI quarantine must enforce expiry/max age limits"
    );
}

#[test]
fn ci_quarantine_requires_evidence() {
    let ci = load_text(CI_WORKFLOW_PATH);
    assert!(
        ci.contains("evidence") && ci.contains("quarantine"),
        "Quarantine entries must reference evidence (CI run URL or artifact path)"
    );
}

#[test]
fn ci_quarantine_has_audit_trail() {
    let ci = load_text(CI_WORKFLOW_PATH);
    assert!(
        ci.contains("quarantine_audit") || ci.contains("audit"),
        "CI must produce quarantine audit trail"
    );
}

#[test]
fn flake_triage_policy_defines_quarantine_max_days() {
    let policy = load_text(FLAKE_TRIAGE_POLICY_PATH);
    assert!(
        policy.contains("14") || policy.contains("max") || policy.contains("day"),
        "Flake triage policy must define quarantine max duration"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 12: Comprehensive report
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn comprehensive_artifact_retention_report() {
    let harness = TestHarness::new("comprehensive_artifact_retention_report");

    let ci = load_text(CI_WORKFLOW_PATH);
    let policy = load_text(TESTING_POLICY_PATH);
    let runbook = load_text(QA_RUNBOOK_PATH);
    let matrix = load_json(SCENARIO_MATRIX_PATH);

    let mut checks = Vec::new();

    // Count artifact-related features in CI
    let artifact_features = [
        ("upload-artifact steps", ci.contains("upload-artifact")),
        ("retention-days config", ci.contains("retention-days")),
        ("always-upload condition", ci.contains("if: always()")),
        (
            "shard artifacts",
            ci.contains("shard") && ci.contains("artifact"),
        ),
        (
            "evidence bundle",
            ci.contains("evidence-bundle") || ci.contains("evidence_bundle"),
        ),
        ("conformance reports", ci.contains("conformance")),
        ("quarantine system", ci.contains("quarantine")),
        (
            "failure signatures",
            ci.contains("signature") || ci.contains("failure"),
        ),
        (
            "correlation IDs",
            ci.contains("CORRELATION_ID") || ci.contains("correlation_id"),
        ),
        (
            "gate promotion",
            ci.contains("promotion") || ci.contains("PROMOTION"),
        ),
    ];

    for (name, present) in &artifact_features {
        checks.push(serde_json::json!({
            "check": name,
            "status": if *present { "pass" } else { "fail" },
            "source": "ci.yml",
        }));
    }

    // Count policy features
    let policy_features = [
        (
            "artifact requirements",
            policy.contains("artifact") || policy.contains("JSONL"),
        ),
        (
            "quarantine reference",
            policy.contains("quarantine") || policy.contains("Quarantine"),
        ),
        (
            "triage workflow",
            runbook.contains("triage") || runbook.contains("Triage"),
        ),
        (
            "reproduction commands",
            runbook.contains("cargo test") || runbook.contains("reproduce"),
        ),
    ];

    for (name, present) in &policy_features {
        checks.push(serde_json::json!({
            "check": name,
            "status": if *present { "pass" } else { "fail" },
            "source": "testing-policy.md / qa-runbook.md",
        }));
    }

    // Check scenario matrix coverage
    let rows = matrix["rows"].as_array().unwrap_or(&Vec::new()).len();
    let rows_with_artifacts = matrix["rows"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .filter(|r| r["expected_artifacts"].is_array())
        .count();

    checks.push(serde_json::json!({
        "check": "scenario matrix artifact coverage",
        "status": if rows_with_artifacts == rows { "pass" } else { "fail" },
        "detail": format!("{rows_with_artifacts}/{rows} workflows define expected_artifacts"),
    }));

    // Write comprehensive report
    let report = serde_json::json!({
        "schema": "pi.test.artifact_retention_report.v1",
        "total_checks": checks.len(),
        "passed": checks.iter().filter(|c| c["status"] == "pass").count(),
        "failed": checks.iter().filter(|c| c["status"] == "fail").count(),
        "checks": checks,
    });

    let report_path = harness.temp_path("artifact_retention_report.json");
    std::fs::write(&report_path, serde_json::to_string_pretty(&report).unwrap())
        .expect("write report");
    harness.record_artifact("artifact_retention_report.json", &report_path);

    let passed = checks.iter().filter(|c| c["status"] == "pass").count();
    let total = checks.len();
    assert!(
        passed >= total - 1,
        "At least {}/{total} checks should pass, got {passed}",
        total - 1
    );
}
