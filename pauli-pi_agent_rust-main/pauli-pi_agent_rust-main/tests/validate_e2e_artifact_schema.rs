//! Structured log/artifact schema validation for provider e2e workflows (bd-3uqg.14.3.3).
//!
//! Validates that provider e2e and quality-gate claims include enforceable
//! structured logging artifacts tied to executable schemas, not prose-only
//! assertions. Cross-references:
//! - JSONL schema definitions (pi.test.log.v2, pi.test.artifact.v1)
//! - Correlation ID model (trace_id, span_id, parent_span_id)
//! - Redaction completeness (10 sensitive key patterns)
//! - Artifact contract (docs/provider_e2e_artifact_contract.json)
//! - Scenario matrix required artifacts
//! - Evidence contract, failure digest, replay bundle schemas
//!
//! Run:
//! ```bash
//! cargo test --test validate_e2e_artifact_schema
//! ```

#![allow(clippy::too_many_lines)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::items_after_statements)]

mod common;

use common::logging::{
    find_unredacted_keys, redact_json_value, validate_jsonl, validate_jsonl_line,
    validate_jsonl_line_v2_only, validate_jsonl_v2_only,
};
use common::{
    EVIDENCE_CONTRACT_SCHEMA_V1, FAILURE_DIGEST_SCHEMA_V1, PARITY_TEST_LOGGING_CONTRACT_SCHEMA_V1,
    TEST_ARTIFACT_SCHEMA_V1, TEST_LOG_SCHEMA_V2, TestHarness,
};
use serde_json::Value;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn load_text(path: &Path) -> Option<String> {
    std::fs::read_to_string(path).ok()
}

fn load_json(path: &Path) -> Option<Value> {
    let text = load_text(path)?;
    serde_json::from_str(&text).ok()
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 1: Artifact contract schema exists and is well-formed
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn artifact_contract_exists() {
    let path = repo_root().join("docs/provider_e2e_artifact_contract.json");
    assert!(
        path.exists(),
        "Provider e2e artifact contract must exist at docs/provider_e2e_artifact_contract.json"
    );
}

#[test]
fn artifact_contract_is_valid_json() {
    let path = repo_root().join("docs/provider_e2e_artifact_contract.json");
    let content = load_text(&path).expect("read artifact contract");
    let parsed: Result<Value, _> = serde_json::from_str(&content);
    assert!(parsed.is_ok(), "Artifact contract must be valid JSON");
}

#[test]
fn artifact_contract_has_schema_field() {
    let path = repo_root().join("docs/provider_e2e_artifact_contract.json");
    let contract = load_json(&path).expect("parse artifact contract");
    let schema = contract["schema"].as_str().unwrap_or("");
    assert_eq!(
        schema, "pi.qa.provider_e2e_artifact_contract.v1",
        "Contract must use schema pi.qa.provider_e2e_artifact_contract.v1"
    );
}

#[test]
fn artifact_contract_defines_jsonl_schemas() {
    let path = repo_root().join("docs/provider_e2e_artifact_contract.json");
    let contract = load_json(&path).expect("parse artifact contract");
    let schemas = contract["jsonl_schemas"]
        .as_object()
        .expect("jsonl_schemas must be object");

    let expected = [
        "pi.test.log.v1",
        TEST_LOG_SCHEMA_V2,
        TEST_ARTIFACT_SCHEMA_V1,
    ];
    for schema in &expected {
        assert!(
            schemas.contains_key(*schema),
            "Contract must define schema: {schema}"
        );
    }
}

#[test]
fn artifact_contract_defines_redaction_policy() {
    let path = repo_root().join("docs/provider_e2e_artifact_contract.json");
    let contract = load_json(&path).expect("parse artifact contract");
    let policy = &contract["redaction_policy"];
    assert!(policy.is_object(), "Contract must define redaction_policy");

    let keys = policy["sensitive_keys"]
        .as_array()
        .expect("sensitive_keys array");
    assert!(
        keys.len() >= 10,
        "Redaction policy must list at least 10 sensitive key patterns"
    );
}

#[test]
fn artifact_contract_defines_correlation_model() {
    let path = repo_root().join("docs/provider_e2e_artifact_contract.json");
    let contract = load_json(&path).expect("parse artifact contract");
    let model = &contract["correlation_model"];
    assert!(model.is_object(), "Contract must define correlation_model");
    assert!(
        model["trace_id"].is_object(),
        "Correlation model must define trace_id"
    );
    assert!(
        model["span_id"].is_object(),
        "Correlation model must define span_id"
    );
}

#[test]
fn artifact_contract_defines_per_suite_artifacts() {
    let path = repo_root().join("docs/provider_e2e_artifact_contract.json");
    let contract = load_json(&path).expect("parse artifact contract");
    let suite = &contract["per_suite_artifacts"];
    assert!(
        suite.is_object(),
        "Contract must define per_suite_artifacts"
    );

    let required = suite["required"]
        .as_array()
        .expect("required artifacts array");
    let names: Vec<&str> = required.iter().filter_map(|a| a["name"].as_str()).collect();
    assert!(
        names.contains(&"test-log.jsonl"),
        "Per-suite artifacts must require test-log.jsonl"
    );
    assert!(
        names.contains(&"artifact-index.jsonl"),
        "Per-suite artifacts must require artifact-index.jsonl"
    );
}

#[test]
fn artifact_contract_defines_per_run_artifacts() {
    let path = repo_root().join("docs/provider_e2e_artifact_contract.json");
    let contract = load_json(&path).expect("parse artifact contract");
    let run = &contract["per_run_artifacts"];
    assert!(run.is_object(), "Contract must define per_run_artifacts");

    let required = run["required"].as_array().expect("required run artifacts");
    let names: Vec<&str> = required.iter().filter_map(|a| a["name"].as_str()).collect();
    assert!(
        names.contains(&"evidence_contract.json"),
        "Per-run artifacts must require evidence_contract.json"
    );
    assert!(
        names.contains(&"environment.json"),
        "Per-run artifacts must require environment.json"
    );
}

#[test]
fn artifact_contract_documents_gaps() {
    let path = repo_root().join("docs/provider_e2e_artifact_contract.json");
    let contract = load_json(&path).expect("parse artifact contract");
    let gaps = contract["gaps_identified"]
        .as_array()
        .expect("gaps_identified array");
    assert!(
        !gaps.is_empty(),
        "Contract must document identified gaps with severity"
    );
    for gap in gaps {
        assert!(gap["id"].is_string(), "Each gap must have an id");
        assert!(gap["severity"].is_string(), "Each gap must have a severity");
        assert!(
            gap["recommendation"].is_string(),
            "Each gap must have a recommendation"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 2: JSONL schema validation is executable (not prose-only)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn validate_jsonl_line_accepts_valid_v2_record() {
    let record = serde_json::json!({
        "schema": "pi.test.log.v2",
        "type": "log",
        "trace_id": "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4",
        "seq": 1,
        "ts": "2026-02-13T00:00:00Z",
        "t_ms": 0,
        "level": "info",
        "category": "test",
        "message": "test entry"
    });
    let line = serde_json::to_string(&record).unwrap();
    assert!(
        validate_jsonl_line(&line, 1).is_ok(),
        "Valid v2 record must pass validation"
    );
}

#[test]
fn validate_jsonl_line_rejects_missing_trace_id_in_v2() {
    let record = serde_json::json!({
        "schema": "pi.test.log.v2",
        "type": "log",
        "seq": 1,
        "ts": "2026-02-13T00:00:00Z",
        "t_ms": 0,
        "level": "info",
        "category": "test",
        "message": "no trace_id"
    });
    let line = serde_json::to_string(&record).unwrap();
    let result = validate_jsonl_line(&line, 1);
    assert!(
        result.is_err(),
        "v2 record without trace_id must fail validation"
    );
}

#[test]
fn validate_jsonl_line_accepts_valid_artifact_record() {
    let record = serde_json::json!({
        "schema": "pi.test.artifact.v1",
        "type": "artifact",
        "seq": 1,
        "ts": "2026-02-13T00:00:00Z",
        "t_ms": 100,
        "name": "test-output.json",
        "path": "/tmp/test-output.json"
    });
    let line = serde_json::to_string(&record).unwrap();
    assert!(
        validate_jsonl_line(&line, 1).is_ok(),
        "Valid artifact record must pass validation"
    );
}

#[test]
fn validate_jsonl_line_rejects_unknown_schema() {
    let record = serde_json::json!({
        "schema": "pi.test.unknown.v99",
        "type": "log",
        "seq": 1
    });
    let line = serde_json::to_string(&record).unwrap();
    let result = validate_jsonl_line(&line, 1);
    assert!(result.is_err(), "Unknown schema must fail validation");
}

#[test]
fn validate_jsonl_line_rejects_non_numeric_seq() {
    let record = serde_json::json!({
        "schema": "pi.test.log.v2",
        "type": "log",
        "trace_id": "aaaa",
        "seq": "not-a-number",
        "ts": "2026-02-13T00:00:00Z",
        "t_ms": 0,
        "level": "info",
        "category": "test",
        "message": "bad seq"
    });
    let line = serde_json::to_string(&record).unwrap();
    let result = validate_jsonl_line(&line, 1);
    assert!(
        result.is_err(),
        "Non-numeric seq field must fail validation"
    );
}

#[test]
fn validate_jsonl_batch_catches_all_errors() {
    let content = [
        r#"{"schema":"pi.test.log.v2","type":"log","trace_id":"abc","seq":1,"ts":"2026-01-01T00:00:00Z","t_ms":0,"level":"info","category":"t","message":"ok"}"#,
        r#"{"schema":"pi.test.unknown.v1","type":"bad"}"#,
        r"not valid json at all",
        r#"{"schema":"pi.test.log.v2","type":"log","seq":2}"#,
    ].join("\n");

    let errors = validate_jsonl(&content);
    assert!(
        errors.len() >= 2,
        "Batch validation must catch multiple errors, got {}",
        errors.len()
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 3: Harness produces schema-compliant JSONL
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn harness_log_output_passes_schema_validation() {
    let harness = TestHarness::new("schema_validation_test");
    harness.info("first entry");
    harness.info("second entry");
    harness.info_ctx("with context", &[("key", "value")]);

    let output = harness.dump_logs();
    let errors = validate_jsonl(&output);
    assert!(
        errors.is_empty(),
        "Harness JSONL output must pass schema validation. Errors: {errors:?}"
    );
}

#[test]
fn harness_artifact_index_passes_schema_validation() {
    let harness = TestHarness::new("artifact_index_schema_test");
    let path = harness.temp_path("data.json");
    std::fs::write(&path, r#"{"ok": true}"#).unwrap();
    harness.record_artifact("data.json", &path);

    let index = harness.dump_artifact_index();
    let errors = validate_jsonl(&index);
    assert!(
        errors.is_empty(),
        "Artifact index JSONL must pass schema validation. Errors: {errors:?}"
    );
}

#[test]
fn harness_logs_use_v2_schema() {
    let harness = TestHarness::new("v2_schema_test");
    harness.info("check version");

    let output = harness.dump_logs();
    for line in output.lines().filter(|l| !l.is_empty()) {
        let parsed: Value = serde_json::from_str(line).expect("valid JSON");
        let schema = parsed["schema"].as_str().unwrap_or("");
        assert_eq!(
            schema, TEST_LOG_SCHEMA_V2,
            "All new harness logs must use {TEST_LOG_SCHEMA_V2}, got {schema}",
        );
    }
}

#[test]
fn harness_logs_have_monotonic_seq() {
    let harness = TestHarness::new("monotonic_seq_test");
    harness.info("one");
    harness.info("two");
    harness.info("three");

    let output = harness.dump_logs();
    let mut prev_seq = 0u64;
    for line in output.lines().filter(|l| !l.is_empty()) {
        let parsed: Value = serde_json::from_str(line).expect("valid JSON");
        let seq = parsed["seq"].as_u64().expect("seq is number");
        assert!(
            seq > prev_seq,
            "seq must be monotonically increasing: got {seq} after {prev_seq}"
        );
        prev_seq = seq;
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 4: Correlation ID enforcement
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn harness_logs_contain_trace_id() {
    let harness = TestHarness::new("trace_id_test");
    harness.info("check trace_id");

    let output = harness.dump_logs();
    let first_line = output.lines().next().expect("at least one line");
    let parsed: Value = serde_json::from_str(first_line).expect("valid JSON");
    let trace_id = parsed["trace_id"]
        .as_str()
        .expect("trace_id must be string");
    assert!(!trace_id.is_empty(), "trace_id must be non-empty");
}

#[test]
fn trace_id_is_consistent_within_logger() {
    let harness = TestHarness::new("trace_id_consistency_test");
    harness.info("first");
    harness.info("second");
    harness.info("third");

    let output = harness.dump_logs();
    let trace_ids: HashSet<String> = output
        .lines()
        .filter(|l| !l.is_empty())
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .filter_map(|v| v["trace_id"].as_str().map(String::from))
        .collect();

    assert_eq!(
        trace_ids.len(),
        1,
        "All logs from same harness must share one trace_id, got {trace_ids:?}"
    );
}

#[test]
fn different_harnesses_have_different_trace_ids() {
    let h1 = TestHarness::new("trace_unique_1");
    h1.info("from h1");
    let h2 = TestHarness::new("trace_unique_2");
    h2.info("from h2");

    let get_trace_id = |harness: &TestHarness| -> String {
        let output = harness.dump_logs();
        let first = output.lines().next().unwrap();
        let parsed: Value = serde_json::from_str(first).unwrap();
        parsed["trace_id"].as_str().unwrap().to_string()
    };

    let id1 = get_trace_id(&h1);
    let id2 = get_trace_id(&h2);
    assert_ne!(
        id1, id2,
        "Different harness instances must have distinct trace_ids"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 5: Redaction enforcement
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn redaction_covers_all_sensitive_key_patterns() {
    let sensitive_keys = [
        "api_key",
        "api-key",
        "authorization",
        "bearer",
        "cookie",
        "credential",
        "password",
        "private_key",
        "secret",
        "token",
    ];

    for key in &sensitive_keys {
        let harness = TestHarness::new(format!("redact_{key}"));
        harness.info_ctx("test", &[(key, "super-secret-value-12345")]);
        let output = harness.dump_logs();
        assert!(
            !output.contains("super-secret-value-12345"),
            "Sensitive key '{key}' must be redacted from log output"
        );
        assert!(
            output.contains("[REDACTED]"),
            "Redacted key '{key}' must show [REDACTED] placeholder"
        );
    }
}

#[test]
fn deep_json_redaction_handles_nested_objects() {
    let mut value = serde_json::json!({
        "request": {
            "headers": {
                "Authorization": "Bearer secret123",
                "Content-Type": "application/json"
            },
            "body": {
                "nested": {
                    "api_key": "sk-secret-key"
                }
            }
        }
    });

    redact_json_value(&mut value);

    let text = serde_json::to_string(&value).unwrap();
    assert!(
        !text.contains("secret123"),
        "Nested authorization must be redacted"
    );
    assert!(
        !text.contains("sk-secret-key"),
        "Nested api_key must be redacted"
    );
    assert!(
        text.contains("application/json"),
        "Non-sensitive fields must be preserved"
    );
}

#[test]
fn find_unredacted_keys_detects_all_leaks() {
    let value = serde_json::json!({
        "headers": {
            "Authorization": "Bearer leaked-token",
            "x-api-key": "sk-ant-leaked",
            "Content-Type": "text/plain"
        },
        "body": {
            "password": "p@ssw0rd",
            "username": "safe-value"
        }
    });

    let unredacted = find_unredacted_keys(&value);
    assert!(
        unredacted.len() >= 3,
        "Must detect at least 3 unredacted sensitive keys, found: {unredacted:?}"
    );
}

#[test]
fn find_unredacted_keys_passes_when_all_redacted() {
    let value = serde_json::json!({
        "headers": {
            "Authorization": "[REDACTED]",
            "api_key": "[REDACTED]"
        },
        "body": {
            "password": "[REDACTED]",
            "data": "safe"
        }
    });

    let unredacted = find_unredacted_keys(&value);
    assert!(
        unredacted.is_empty(),
        "Properly redacted data must pass: {unredacted:?}"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 6: Scenario matrix artifact requirements are enforceable
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn scenario_matrix_required_suite_artifacts_match_contract() {
    let matrix_path = repo_root().join("docs/e2e_scenario_matrix.json");
    let matrix = load_json(&matrix_path).expect("load scenario matrix");

    let contract_path = repo_root().join("docs/provider_e2e_artifact_contract.json");
    let contract = load_json(&contract_path).expect("load artifact contract");

    let matrix_artifacts: HashSet<String> = matrix["ci_policy"]["required_suite_artifacts"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();

    let contract_artifacts: HashSet<String> = contract["per_suite_artifacts"]["required"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .filter_map(|v| v["name"].as_str().map(String::from))
        .collect();

    // Contract must cover all matrix requirements
    for artifact in &matrix_artifacts {
        assert!(
            contract_artifacts.contains(artifact),
            "Contract must define artifact '{artifact}' required by scenario matrix"
        );
    }
}

#[test]
fn scenario_matrix_required_run_artifacts_match_contract() {
    let matrix_path = repo_root().join("docs/e2e_scenario_matrix.json");
    let matrix = load_json(&matrix_path).expect("load scenario matrix");

    let contract_path = repo_root().join("docs/provider_e2e_artifact_contract.json");
    let contract = load_json(&contract_path).expect("load artifact contract");

    let matrix_artifacts: HashSet<String> = matrix["ci_policy"]["required_run_artifacts"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();

    let contract_artifacts: HashSet<String> = contract["per_run_artifacts"]["required"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .filter_map(|v| v["name"].as_str().map(String::from))
        .collect();

    for artifact in &matrix_artifacts {
        assert!(
            contract_artifacts.contains(artifact),
            "Contract must define run artifact '{artifact}' required by scenario matrix"
        );
    }
}

#[test]
fn scenario_matrix_every_workflow_has_expected_artifacts() {
    let matrix_path = repo_root().join("docs/e2e_scenario_matrix.json");
    let matrix = load_json(&matrix_path).expect("load scenario matrix");

    let rows = matrix["rows"].as_array().expect("rows array");
    let required_suite = matrix["ci_policy"]["required_suite_artifacts"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect::<HashSet<_>>();

    for row in rows {
        let wf_id = row["workflow_id"].as_str().unwrap_or("unknown");
        let expected = row["expected_artifacts"]
            .as_array()
            .unwrap_or(&Vec::new())
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect::<HashSet<_>>();

        // Each workflow must include all required suite artifacts
        for required in &required_suite {
            assert!(
                expected.contains(required),
                "Workflow '{wf_id}' must list required artifact '{required}'"
            );
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 7: Replay bundle schema validation
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn replay_bundle_has_required_fields() {
    let path = repo_root().join("tests/full_suite_gate/replay_bundle.json");
    if let Some(bundle) = load_json(&path) {
        let required_fields = [
            "schema",
            "generated_at",
            "correlation_id",
            "source_summary_path",
            "one_command_replay",
            "environment",
            "summary",
        ];
        for field in &required_fields {
            assert!(
                bundle.get(*field).is_some(),
                "replay_bundle must have field: {field}"
            );
        }

        // Validate environment sub-fields
        let env_fields = ["profile", "rustc_version", "git_sha", "git_branch", "os"];
        let env = &bundle["environment"];
        for field in &env_fields {
            assert!(
                env.get(*field).is_some(),
                "replay_bundle.environment must have field: {field}"
            );
        }
    }
    // If replay_bundle doesn't exist, that's OK - it's generated during full runs
}

#[test]
fn replay_bundle_schema_version_is_v1() {
    let path = repo_root().join("tests/full_suite_gate/replay_bundle.json");
    if let Some(bundle) = load_json(&path) {
        let schema = bundle["schema"].as_str().unwrap_or("");
        assert_eq!(
            schema, "pi.e2e.replay_bundle.v1",
            "Replay bundle must use schema pi.e2e.replay_bundle.v1"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 8: Evidence contract and failure digest schema validation
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn failure_digest_schema_defined_in_runner_script() {
    let script_path = repo_root().join("scripts/e2e/run_all.sh");
    let content = load_text(&script_path).expect("run_all.sh must exist");

    // Failure digest must define these fields in the script
    let required = [
        "schema",
        "suite",
        "root_cause_class",
        "impacted_scenario_ids",
        "first_failing_assertion",
        "remediation_pointer",
    ];

    for field in &required {
        assert!(
            content.contains(field),
            "run_all.sh must define failure_digest field: {field}"
        );
    }
}

#[test]
fn failure_digest_root_cause_taxonomy_is_complete() {
    let script_path = repo_root().join("scripts/e2e/run_all.sh");
    let content = load_text(&script_path).expect("run_all.sh must exist");

    // Contract-defined root cause taxonomy
    let taxonomy = [
        "timeout",
        "assertion_failure",
        "permission_denied",
        "network_io",
        "missing_file",
        "panic",
    ];

    for cause in &taxonomy {
        assert!(
            content.contains(cause),
            "run_all.sh must classify root cause: {cause}"
        );
    }
}

#[test]
fn evidence_contract_referenced_in_runner() {
    let script_path = repo_root().join("scripts/e2e/run_all.sh");
    let content = load_text(&script_path).expect("run_all.sh must exist");
    assert!(
        content.contains("evidence_contract"),
        "run_all.sh must generate evidence_contract.json"
    );
    assert!(
        content.contains("environment.json"),
        "run_all.sh must generate environment.json"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 9: CI workflow enforces artifact schemas
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn ci_workflow_captures_jsonl_artifacts() {
    let ci_path = repo_root().join(".github/workflows/ci.yml");
    let content = load_text(&ci_path).expect("CI workflow must exist");

    assert!(
        content.contains("**/*.jsonl"),
        "CI must capture JSONL artifacts (test-log.jsonl, artifact-index.jsonl)"
    );
    assert!(
        content.contains("**/*.json"),
        "CI must capture JSON artifacts (result.json, evidence_contract.json)"
    );
}

#[test]
fn ci_workflow_has_correlation_id_support() {
    let ci_path = repo_root().join(".github/workflows/ci.yml");
    let content = load_text(&ci_path).expect("CI workflow must exist");
    assert!(
        content.contains("CORRELATION_ID") || content.contains("correlation_id"),
        "CI workflow must support correlation IDs for cross-shard tracing"
    );
}

#[test]
fn ci_workflow_artifact_upload_covers_provider_tests() {
    let ci_path = repo_root().join(".github/workflows/ci.yml");
    let content = load_text(&ci_path).expect("CI workflow must exist");
    assert!(
        content.contains("provider-test-artifacts") || content.contains("provider_test_artifacts"),
        "CI must upload provider test artifacts"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 10: Provider-specific artifact validation
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn artifact_contract_defines_provider_variants() {
    let path = repo_root().join("docs/provider_e2e_artifact_contract.json");
    let contract = load_json(&path).expect("parse artifact contract");
    let variants = contract["provider_specific_requirements"]["provider_variants"]
        .as_object()
        .expect("provider_variants must be object");

    let expected_providers = ["anthropic", "openai", "google", "cohere", "azure"];
    for provider in &expected_providers {
        assert!(
            variants.contains_key(*provider),
            "Contract must define variant for provider: {provider}"
        );
    }
}

#[test]
fn artifact_contract_provider_variants_have_credential_source() {
    let path = repo_root().join("docs/provider_e2e_artifact_contract.json");
    let contract = load_json(&path).expect("parse artifact contract");
    let variants = contract["provider_specific_requirements"]["provider_variants"]
        .as_object()
        .expect("provider_variants");

    for (provider, variant) in variants {
        assert!(
            variant["credential_source"].is_string(),
            "Provider '{provider}' variant must have credential_source"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 11: Cross-reference validation (tests exist that enforce schemas)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_infrastructure_has_jsonl_validation_function() {
    let logging_path = repo_root().join("tests/common/logging.rs");
    let content = load_text(&logging_path).expect("logging.rs must exist");
    assert!(
        content.contains("fn validate_jsonl_line"),
        "tests/common/logging.rs must define validate_jsonl_line()"
    );
    assert!(
        content.contains("fn validate_jsonl"),
        "tests/common/logging.rs must define validate_jsonl()"
    );
}

#[test]
fn test_infrastructure_has_redaction_functions() {
    let logging_path = repo_root().join("tests/common/logging.rs");
    let content = load_text(&logging_path).expect("logging.rs must exist");
    assert!(
        content.contains("fn redact_json_value"),
        "tests/common/logging.rs must define redact_json_value()"
    );
    assert!(
        content.contains("fn find_unredacted_keys"),
        "tests/common/logging.rs must define find_unredacted_keys()"
    );
}

#[test]
fn existing_artifact_retention_tests_cover_provider_workflows() {
    let test_files = [
        "tests/e2e_artifact_retention_triage.rs",
        "tests/ci_artifact_retention.rs",
        "tests/e2e_replay_bundles.rs",
    ];

    for file in &test_files {
        let path = repo_root().join(file);
        assert!(path.exists(), "Artifact validation test must exist: {file}");
    }
}

#[test]
fn provider_e2e_test_files_use_test_harness() {
    let provider_test_files = [
        "tests/e2e_provider_failure_injection.rs",
        "tests/e2e_cross_provider_parity.rs",
    ];

    for file in &provider_test_files {
        let path = repo_root().join(file);
        if path.exists() {
            let content = load_text(&path).unwrap_or_default();
            assert!(
                content.contains("TestHarness"),
                "{file} must use TestHarness for structured artifact output"
            );
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 12: Comprehensive validation report
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn comprehensive_artifact_schema_validation_report() {
    let harness = TestHarness::new("artifact_schema_validation_report");

    let mut checks: Vec<Value> = Vec::new();

    // Check 1: Contract exists
    let contract_exists = repo_root()
        .join("docs/provider_e2e_artifact_contract.json")
        .exists();
    checks.push(serde_json::json!({
        "check": "artifact_contract_exists",
        "status": if contract_exists { "pass" } else { "fail" },
    }));

    // Check 2: Schema validation function exists
    let logging_path = repo_root().join("tests/common/logging.rs");
    let logging = load_text(&logging_path).unwrap_or_default();
    let has_validator = logging.contains("fn validate_jsonl_line");
    checks.push(serde_json::json!({
        "check": "jsonl_schema_validator_exists",
        "status": if has_validator { "pass" } else { "fail" },
    }));

    // Check 3: Redaction functions exist
    let has_redaction =
        logging.contains("fn redact_json_value") && logging.contains("fn find_unredacted_keys");
    checks.push(serde_json::json!({
        "check": "redaction_functions_exist",
        "status": if has_redaction { "pass" } else { "fail" },
    }));

    // Check 4: Scenario matrix defines required artifacts
    let matrix_path = repo_root().join("docs/e2e_scenario_matrix.json");
    let matrix_ok = load_json(&matrix_path).is_some_and(|m| {
        m["ci_policy"]["required_suite_artifacts"]
            .as_array()
            .is_some_and(|a| a.len() >= 4)
    });
    checks.push(serde_json::json!({
        "check": "scenario_matrix_required_artifacts",
        "status": if matrix_ok { "pass" } else { "fail" },
    }));

    // Check 5: CI captures JSONL artifacts
    let ci_path = repo_root().join(".github/workflows/ci.yml");
    let ci = load_text(&ci_path).unwrap_or_default();
    let ci_captures = ci.contains("**/*.jsonl") && ci.contains("**/*.json");
    checks.push(serde_json::json!({
        "check": "ci_captures_structured_artifacts",
        "status": if ci_captures { "pass" } else { "fail" },
    }));

    // Check 6: Replay bundle exists with valid schema
    let replay_ok = load_json(&repo_root().join("tests/full_suite_gate/replay_bundle.json"))
        .is_some_and(|b| b["schema"].as_str() == Some("pi.e2e.replay_bundle.v1"));
    checks.push(serde_json::json!({
        "check": "replay_bundle_schema_valid",
        "status": if replay_ok { "pass" } else { "skip" },
    }));

    // Check 7: Harness produces valid JSONL
    let h = TestHarness::new("inner_validation");
    h.info("test");
    let valid_jsonl = validate_jsonl(&h.dump_logs()).is_empty();
    checks.push(serde_json::json!({
        "check": "harness_produces_valid_jsonl",
        "status": if valid_jsonl { "pass" } else { "fail" },
    }));

    // Check 8: Correlation IDs in CI
    let ci_corr = ci.contains("CORRELATION_ID") || ci.contains("correlation_id");
    checks.push(serde_json::json!({
        "check": "ci_correlation_id_support",
        "status": if ci_corr { "pass" } else { "fail" },
    }));

    let passed = checks.iter().filter(|c| c["status"] == "pass").count();
    let total = checks.len();
    let skipped = checks.iter().filter(|c| c["status"] == "skip").count();

    let report = serde_json::json!({
        "schema": "pi.qa.artifact_schema_validation_report.v1",
        "bead": "bd-3uqg.14.3.3",
        "total_checks": total,
        "passed": passed,
        "skipped": skipped,
        "failed": total - passed - skipped,
        "checks": checks,
    });

    let report_path = harness.temp_path("artifact_schema_validation_report.json");
    std::fs::write(&report_path, serde_json::to_string_pretty(&report).unwrap())
        .expect("write report");
    harness.record_artifact("artifact_schema_validation_report.json", &report_path);

    eprintln!("\n=== Artifact Schema Validation Report ===");
    eprintln!("  Passed: {passed}/{total}");
    eprintln!("  Skipped: {skipped}");
    eprintln!("  Failed: {}", total - passed - skipped);
    eprintln!("  Report: {}", report_path.display());

    assert!(
        passed >= total - skipped - 1,
        "At least {}/{total} checks must pass (excluding skips), got {passed}",
        total - skipped - 1
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 13: Formal evidence contract schema (pi.qa.evidence_contract.v1)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn evidence_contract_schema_exists() {
    let path = repo_root().join("docs/evidence-contract-schema.json");
    assert!(
        path.exists(),
        "Formal evidence contract schema must exist at docs/evidence-contract-schema.json"
    );
}

#[test]
fn evidence_contract_schema_is_valid_json() {
    let path = repo_root().join("docs/evidence-contract-schema.json");
    let content = load_text(&path).expect("read evidence contract schema");
    let parsed: Result<Value, _> = serde_json::from_str(&content);
    assert!(
        parsed.is_ok(),
        "Evidence contract schema must be valid JSON: {:?}",
        parsed.err()
    );
}

#[test]
fn evidence_contract_schema_has_correct_id() {
    let path = repo_root().join("docs/evidence-contract-schema.json");
    let schema = load_json(&path).expect("parse schema");
    assert_eq!(
        schema["$id"].as_str().unwrap_or(""),
        EVIDENCE_CONTRACT_SCHEMA_V1,
        "Schema $id must be {EVIDENCE_CONTRACT_SCHEMA_V1}",
    );
}

#[test]
fn evidence_contract_schema_defines_required_fields() {
    let path = repo_root().join("docs/evidence-contract-schema.json");
    let schema = load_json(&path).expect("parse schema");
    let required: Vec<&str> = schema["required"]
        .as_array()
        .expect("required array")
        .iter()
        .filter_map(|v| v.as_str())
        .collect();

    let expected = [
        "schema",
        "generated_at",
        "correlation_id",
        "run_summary",
        "environment",
        "suite_evidence",
        "aggregate_artifacts",
    ];
    for field in &expected {
        assert!(
            required.contains(field),
            "Schema must list '{field}' as required"
        );
    }
}

#[test]
fn evidence_contract_schema_defines_suite_entry() {
    let path = repo_root().join("docs/evidence-contract-schema.json");
    let schema = load_json(&path).expect("parse schema");
    let suite_entry = &schema["definitions"]["suite_entry"];
    assert!(
        suite_entry.is_object(),
        "Schema must define a suite_entry in definitions"
    );

    let required: Vec<&str> = suite_entry["required"]
        .as_array()
        .expect("suite_entry required array")
        .iter()
        .filter_map(|v| v.as_str())
        .collect();

    assert!(
        required.contains(&"suite_id"),
        "suite_entry must require suite_id"
    );
    assert!(
        required.contains(&"status"),
        "suite_entry must require status"
    );
    assert!(
        required.contains(&"artifacts"),
        "suite_entry must require artifacts"
    );
    assert!(
        required.contains(&"elapsed_ms"),
        "suite_entry must require elapsed_ms"
    );
}

#[test]
fn evidence_contract_schema_defines_failure_digest() {
    let path = repo_root().join("docs/evidence-contract-schema.json");
    let schema = load_json(&path).expect("parse schema");
    let digest = &schema["definitions"]["failure_digest"];
    assert!(
        digest.is_object(),
        "Schema must define a failure_digest in definitions"
    );

    let root_cause_enum = &digest["properties"]["root_cause_class"]["enum"];
    let causes: Vec<&str> = root_cause_enum
        .as_array()
        .expect("root_cause_class enum")
        .iter()
        .filter_map(|v| v.as_str())
        .collect();

    let expected_causes = [
        "timeout",
        "assertion_failure",
        "permission_denied",
        "network_io",
        "missing_file",
        "panic",
    ];
    for cause in &expected_causes {
        assert!(
            causes.contains(cause),
            "failure_digest root_cause_class must include '{cause}'"
        );
    }
}

#[test]
fn evidence_contract_schema_defines_parity_contract_overlay() {
    let path = repo_root().join("docs/evidence-contract-schema.json");
    let schema = load_json(&path).expect("parse schema");
    let overlay = &schema["definitions"]["parity_contract"];
    assert!(
        overlay.is_object(),
        "Schema must define a parity_contract overlay in definitions"
    );

    assert_eq!(
        overlay["properties"]["schema"]["const"]
            .as_str()
            .unwrap_or(""),
        PARITY_TEST_LOGGING_CONTRACT_SCHEMA_V1,
        "parity_contract schema const must match the documented contract version"
    );

    assert_eq!(
        overlay["properties"]["log_record_schema"]["const"]
            .as_str()
            .unwrap_or(""),
        TEST_LOG_SCHEMA_V2,
        "parity_contract log_record_schema must pin {TEST_LOG_SCHEMA_V2}",
    );
    assert_eq!(
        overlay["properties"]["artifact_record_schema"]["const"]
            .as_str()
            .unwrap_or(""),
        TEST_ARTIFACT_SCHEMA_V1,
        "parity_contract artifact_record_schema must pin {TEST_ARTIFACT_SCHEMA_V1}",
    );
    assert_eq!(
        overlay["properties"]["failure_digest_schema"]["const"]
            .as_str()
            .unwrap_or(""),
        FAILURE_DIGEST_SCHEMA_V1,
        "parity_contract failure_digest_schema must pin {FAILURE_DIGEST_SCHEMA_V1}",
    );
}

#[test]
fn evidence_contract_schema_suite_artifacts_match_contract() {
    // Verify that the evidence contract schema's suite_artifacts fields
    // match the per_suite_artifacts defined in the artifact contract
    let schema_path = repo_root().join("docs/evidence-contract-schema.json");
    let schema = load_json(&schema_path).expect("parse schema");

    let contract_path = repo_root().join("docs/provider_e2e_artifact_contract.json");
    let contract = load_json(&contract_path).expect("parse artifact contract");

    // Map contract artifact names to schema field names
    let contract_names: Vec<&str> = contract["per_suite_artifacts"]["required"]
        .as_array()
        .expect("required suite artifacts")
        .iter()
        .filter_map(|a| a["name"].as_str())
        .collect();

    let schema_artifact_props = schema["definitions"]["suite_artifacts"]["properties"]
        .as_object()
        .expect("suite_artifacts properties");

    // Each contract artifact must have a corresponding schema field
    // Contract names: output.log, result.json, test-log.jsonl, artifact-index.jsonl
    // Schema fields: output_log, result, test_log, artifact_index
    let name_to_field: Vec<(&str, &str)> = vec![
        ("output.log", "output_log"),
        ("result.json", "result"),
        ("test-log.jsonl", "test_log"),
        ("artifact-index.jsonl", "artifact_index"),
    ];

    for (contract_name, schema_field) in &name_to_field {
        assert!(
            contract_names.contains(contract_name),
            "Artifact contract must define '{contract_name}'"
        );
        assert!(
            schema_artifact_props.contains_key(*schema_field),
            "Evidence schema must define field '{schema_field}' for artifact '{contract_name}'"
        );
    }
}

#[test]
fn evidence_contract_schema_environment_matches_replay_bundle() {
    // The evidence contract environment fields should be a superset of
    // the replay bundle environment fields defined in the artifact contract
    let schema_path = repo_root().join("docs/evidence-contract-schema.json");
    let schema = load_json(&schema_path).expect("parse schema");

    let contract_path = repo_root().join("docs/provider_e2e_artifact_contract.json");
    let contract = load_json(&contract_path).expect("parse artifact contract");

    let replay_env_fields: Vec<&str> =
        contract["failure_diagnostics"]["replay_bundle"]["environment_fields"]
            .as_array()
            .expect("replay bundle environment_fields")
            .iter()
            .filter_map(|v| v.as_str())
            .collect();

    let schema_env_props = schema["properties"]["environment"]["properties"]
        .as_object()
        .expect("environment properties");

    for field in &replay_env_fields {
        assert!(
            schema_env_props.contains_key(*field),
            "Evidence schema environment must include replay_bundle field '{field}'"
        );
    }
}

#[test]
fn synthetic_evidence_contract_validates_against_schema() {
    // Build a synthetic evidence_contract.json and verify it has all required fields
    let schema_path = repo_root().join("docs/evidence-contract-schema.json");
    let schema = load_json(&schema_path).expect("parse schema");

    let required: Vec<String> = schema["required"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();

    let synthetic = serde_json::json!({
        "schema": EVIDENCE_CONTRACT_SCHEMA_V1,
        "generated_at": "2026-02-13T00:00:00Z",
        "correlation_id": "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4",
        "run_summary": {
            "total_suites": 6,
            "passed": 5,
            "failed": 1,
            "skipped": 0,
            "elapsed_ms": 45000
        },
        "environment": {
            "profile": "debug",
            "rustc_version": "1.85.0-nightly",
            "git_sha": "abc1234",
            "git_branch": "main",
            "os": "linux"
        },
        "suite_evidence": [
            {
                "suite_id": "anthropic",
                "status": "pass",
                "artifacts": {
                    "test_log": "artifacts/anthropic/test-log.jsonl",
                    "artifact_index": "artifacts/anthropic/artifact-index.jsonl",
                    "result": "artifacts/anthropic/result.json",
                    "output_log": "artifacts/anthropic/output.log"
                },
                "trace_id": "deadbeef01234567deadbeef01234567",
                "elapsed_ms": 8000
            },
            {
                "suite_id": "openai",
                "status": "fail",
                "artifacts": {
                    "test_log": "artifacts/openai/test-log.jsonl",
                    "artifact_index": "artifacts/openai/artifact-index.jsonl",
                    "result": "artifacts/openai/result.json",
                    "output_log": "artifacts/openai/output.log"
                },
                "elapsed_ms": 12000,
                "failure_digest": {
                    "schema": FAILURE_DIGEST_SCHEMA_V1,
                    "suite": "openai",
                    "root_cause_class": "assertion_failure",
                    "impacted_scenario_ids": ["SC-OAI-001"],
                    "first_failing_assertion": "expected status 200, got 500",
                    "remediation_pointer": {
                        "replay_command": "cargo test --test e2e_openai -- --nocapture"
                    }
                }
            }
        ],
        "parity_contract": {
            "schema": PARITY_TEST_LOGGING_CONTRACT_SCHEMA_V1,
            "suite_taxonomy_ref": "tests/suite_classification.toml",
            "log_record_schema": TEST_LOG_SCHEMA_V2,
            "artifact_record_schema": TEST_ARTIFACT_SCHEMA_V1,
            "failure_digest_schema": FAILURE_DIGEST_SCHEMA_V1,
            "trace_model": {
                "correlation_id_field": "correlation_id",
                "trace_id_field": "trace_id",
                "span_id_field": "span_id",
                "parent_span_id_field": "parent_span_id"
            },
            "triage_required_fields": [
                "root_cause_class",
                "first_failing_assertion",
                "remediation_pointer.replay_command"
            ]
        },
        "aggregate_artifacts": {
            "summary": "artifacts/summary.json",
            "environment": "artifacts/environment.json",
            "replay_bundle": "artifacts/replay_bundle.json"
        }
    });

    // Verify all top-level required fields present
    for field in &required {
        assert!(
            synthetic.get(field).is_some(),
            "Synthetic evidence contract must have required field '{field}'"
        );
    }

    // Verify schema value matches const
    assert_eq!(
        synthetic["schema"].as_str().unwrap(),
        EVIDENCE_CONTRACT_SCHEMA_V1
    );

    // Verify run_summary fields
    let summary_required = ["total_suites", "passed", "failed", "skipped", "elapsed_ms"];
    for field in &summary_required {
        assert!(
            synthetic["run_summary"].get(*field).is_some(),
            "run_summary must have field '{field}'"
        );
    }

    // Verify environment required fields
    let env_required = ["profile", "rustc_version", "git_sha", "git_branch", "os"];
    for field in &env_required {
        assert!(
            synthetic["environment"].get(*field).is_some(),
            "environment must have field '{field}'"
        );
    }

    // Verify suite_evidence entries
    let suites = synthetic["suite_evidence"].as_array().unwrap();
    assert_eq!(suites.len(), 2);
    for suite in suites {
        assert!(suite["suite_id"].is_string(), "suite must have suite_id");
        assert!(suite["status"].is_string(), "suite must have status");
        assert!(suite["artifacts"].is_object(), "suite must have artifacts");
        assert!(
            suite["elapsed_ms"].is_number(),
            "suite must have elapsed_ms"
        );

        // Verify artifact paths
        let artifacts = &suite["artifacts"];
        for field in &["test_log", "artifact_index", "result", "output_log"] {
            assert!(
                artifacts.get(*field).is_some(),
                "suite artifacts must have field '{field}'"
            );
        }
    }

    // Verify the failing suite has a failure_digest
    let failing = &suites[1];
    assert_eq!(failing["status"].as_str().unwrap(), "fail");
    let digest = &failing["failure_digest"];
    assert!(digest.is_object(), "failing suite must have failure_digest");
    assert_eq!(digest["schema"].as_str().unwrap(), FAILURE_DIGEST_SCHEMA_V1);
    assert_eq!(
        digest["root_cause_class"].as_str().unwrap(),
        "assertion_failure"
    );

    // Verify optional parity contract overlay shape.
    let contract = &synthetic["parity_contract"];
    assert_eq!(
        contract["schema"].as_str().unwrap(),
        PARITY_TEST_LOGGING_CONTRACT_SCHEMA_V1
    );
    assert_eq!(
        contract["trace_model"]["correlation_id_field"]
            .as_str()
            .unwrap(),
        "correlation_id"
    );
    assert_eq!(
        contract["trace_model"]["trace_id_field"].as_str().unwrap(),
        "trace_id"
    );
}

// ============================================================================
// § 14 — V2-only schema enforcement (DISC-021 / GAP-5 / bd-38m8w)
// ============================================================================

#[test]
fn v2_only_enforcement_rejects_v1_log_records() {
    let v1_record = serde_json::json!({
        "schema": "pi.test.log.v1",
        "type": "log",
        "seq": 1,
        "ts": "2026-02-13T00:00:00Z",
        "t_ms": 0,
        "level": "info",
        "category": "test",
        "message": "this should fail v2-only"
    });
    let line = serde_json::to_string(&v1_record).unwrap();
    let result = validate_jsonl_line_v2_only(&line, 1);
    assert!(
        result.is_err(),
        "v1 log records must be rejected by v2-only validation"
    );
    let err = result.unwrap_err();
    assert!(
        err.message.contains("deprecated"),
        "Error must mention deprecation: {err}"
    );
}

#[test]
fn v2_only_enforcement_accepts_v2_log_records() {
    let v2_record = serde_json::json!({
        "schema": "pi.test.log.v2",
        "type": "log",
        "trace_id": "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4",
        "seq": 1,
        "ts": "2026-02-13T00:00:00Z",
        "t_ms": 0,
        "level": "info",
        "category": "test",
        "message": "v2 record"
    });
    let line = serde_json::to_string(&v2_record).unwrap();
    assert!(
        validate_jsonl_line_v2_only(&line, 1).is_ok(),
        "v2 log records must pass v2-only validation"
    );
}

#[test]
fn v2_only_enforcement_accepts_artifact_v1_records() {
    // pi.test.artifact.v1 is the CURRENT artifact schema (not deprecated).
    let artifact = serde_json::json!({
        "schema": "pi.test.artifact.v1",
        "type": "artifact",
        "seq": 1,
        "ts": "2026-02-13T00:00:00Z",
        "t_ms": 100,
        "name": "output.log",
        "path": "/tmp/output.log"
    });
    let line = serde_json::to_string(&artifact).unwrap();
    assert!(
        validate_jsonl_line_v2_only(&line, 1).is_ok(),
        "Artifact v1 records must pass v2-only validation (not deprecated)"
    );
}

#[test]
fn harness_output_passes_v2_only_enforcement() {
    // Prove that TestHarness always emits v2-only compliant JSONL.
    let harness = TestHarness::new("v2_only_enforcement_test");
    harness.info("test entry");
    harness.info_ctx("action", &[("key", "value")]);
    let path = harness.temp_path("artifact.json");
    std::fs::write(&path, "{}").unwrap();
    harness.record_artifact("artifact.json", &path);

    let log_output = harness.dump_logs();
    let errors = validate_jsonl_v2_only(&log_output);
    assert!(
        errors.is_empty(),
        "Harness log output must be v2-only compliant: {errors:?}"
    );

    let artifact_output = harness.dump_artifact_index();
    let artifact_errors = validate_jsonl_v2_only(&artifact_output);
    assert!(
        artifact_errors.is_empty(),
        "Harness artifact output must be v2-only compliant: {artifact_errors:?}"
    );
}

#[test]
fn v2_only_batch_enforcement_catches_all_v1_records() {
    let mixed_content = [
        r#"{"schema":"pi.test.log.v2","type":"log","trace_id":"abc","seq":1,"ts":"x","t_ms":0,"level":"info","category":"c","message":"ok"}"#,
        r#"{"schema":"pi.test.log.v1","type":"log","seq":2,"ts":"x","t_ms":0,"level":"info","category":"c","message":"v1-bad"}"#,
        r#"{"schema":"pi.test.artifact.v1","type":"artifact","seq":3,"ts":"x","t_ms":0,"name":"a","path":"/tmp/a"}"#,
        r#"{"schema":"pi.test.log.v1","type":"log","seq":4,"ts":"x","t_ms":0,"level":"warn","category":"c","message":"v1-also-bad"}"#,
    ]
    .join("\n");

    let errors = validate_jsonl_v2_only(&mixed_content);
    assert_eq!(
        errors.len(),
        2,
        "Both v1 log records must be rejected: {errors:?}"
    );
    assert_eq!(errors[0].line, 2);
    assert_eq!(errors[1].line, 4);
}

#[test]
fn backward_compat_validate_jsonl_still_accepts_v1() {
    // Grandfathered path: validate_jsonl() (non-strict) still accepts v1.
    let v1_record = r#"{"schema":"pi.test.log.v1","type":"log","seq":1,"ts":"x","t_ms":0,"level":"info","category":"c","message":"grandfathered"}"#;
    let errors = validate_jsonl(v1_record);
    assert!(
        errors.is_empty(),
        "Standard validate_jsonl must still accept v1 for backward compat"
    );
}

// ============================================================================
// § 15 — Artifact-index path cross-validation (DISC-019 / GAP-3 / bd-z5vt4)
// ============================================================================

#[test]
fn artifact_index_cross_validation_detects_missing_paths() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let existing_file = dir.path().join("output.log");
    std::fs::write(&existing_file, "test output").expect("write test file");

    let existing = existing_file.display();
    let missing = dir.path().join("nonexistent.log").display().to_string();

    let artifact_index = format!(
        concat!(
            r#"{{"schema":"pi.test.artifact.v1","type":"artifact","seq":1,"ts":"x","t_ms":100,"name":"output","path":"{existing}"}}"#,
            "\n",
            r#"{{"schema":"pi.test.artifact.v1","type":"artifact","seq":2,"ts":"x","t_ms":200,"name":"missing_artifact","path":"{missing}"}}"#,
            "\n",
        ),
        existing = existing,
        missing = missing,
    );

    let warnings = common::logging::validate_artifact_index_paths(&artifact_index, dir.path());

    assert_eq!(warnings.len(), 1, "should detect exactly one missing path");
    assert_eq!(warnings[0].name, "missing_artifact");
    assert!(
        warnings[0].path.contains("nonexistent.log"),
        "warning path should reference the missing file"
    );
}

#[test]
fn artifact_index_cross_validation_passes_when_all_paths_exist() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let file_a = dir.path().join("result.json");
    let file_b = dir.path().join("test-log.jsonl");
    std::fs::write(&file_a, "{}").expect("write file a");
    std::fs::write(&file_b, "").expect("write file b");

    let artifact_index = format!(
        concat!(
            r#"{{"schema":"pi.test.artifact.v1","type":"artifact","seq":1,"ts":"x","t_ms":0,"name":"result","path":"{a}"}}"#,
            "\n",
            r#"{{"schema":"pi.test.artifact.v1","type":"artifact","seq":2,"ts":"x","t_ms":0,"name":"test_log","path":"{b}"}}"#,
            "\n",
        ),
        a = file_a.display(),
        b = file_b.display(),
    );

    let warnings = common::logging::validate_artifact_index_paths(&artifact_index, dir.path());
    assert!(warnings.is_empty(), "all paths exist, no warnings expected");
}

#[test]
fn artifact_index_cross_validation_handles_relative_paths() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let sub = dir.path().join("artifacts");
    std::fs::create_dir_all(&sub).expect("create subdir");
    std::fs::write(sub.join("output.log"), "data").expect("write file");

    let artifact_index = concat!(
        r#"{"schema":"pi.test.artifact.v1","type":"artifact","seq":1,"ts":"x","t_ms":0,"name":"output","path":"artifacts/output.log"}"#,
        "\n",
    );

    let warnings = common::logging::validate_artifact_index_paths(artifact_index, dir.path());
    assert!(
        warnings.is_empty(),
        "relative path should resolve against artifact_dir"
    );
}

#[test]
fn artifact_index_cross_validation_warns_on_missing_relative_path() {
    let dir = tempfile::tempdir().expect("create temp dir");

    let artifact_index = concat!(
        r#"{"schema":"pi.test.artifact.v1","type":"artifact","seq":1,"ts":"x","t_ms":0,"name":"ghost","path":"does/not/exist.log"}"#,
        "\n",
    );

    let warnings = common::logging::validate_artifact_index_paths(artifact_index, dir.path());
    assert_eq!(warnings.len(), 1);
    assert_eq!(warnings[0].name, "ghost");
    assert_eq!(warnings[0].path, "does/not/exist.log");
    assert_eq!(warnings[0].line, 1);
}

#[test]
fn artifact_index_cross_validation_skips_non_artifact_records() {
    let dir = tempfile::tempdir().expect("create temp dir");

    // Mix a v2 log record (should be skipped) with an artifact record
    let artifact_index = concat!(
        r#"{"schema":"pi.test.log.v2","type":"log","trace_id":"abc","seq":1,"ts":"x","t_ms":0,"level":"info","category":"c","message":"m"}"#,
        "\n",
        r#"{"schema":"pi.test.artifact.v1","type":"artifact","seq":2,"ts":"x","t_ms":0,"name":"missing","path":"nope.log"}"#,
        "\n",
    );

    let warnings = common::logging::validate_artifact_index_paths(artifact_index, dir.path());
    assert_eq!(
        warnings.len(),
        1,
        "only the artifact record should be checked"
    );
    assert_eq!(warnings[0].name, "missing");
}
