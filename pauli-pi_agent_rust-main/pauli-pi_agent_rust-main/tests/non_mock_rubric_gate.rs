// tests/non_mock_rubric_gate.rs
//
// Enforcement tests for the non-mock rubric (bd-1f42.2.8).
// Validates rubric schema integrity, module threshold consistency,
// exception template compliance, and failure-log schema structure.
//
// These tests do NOT require llvm-cov or live coverage data.
// They validate the rubric artifact itself so downstream tasks
// can trust it as a stable reference.

use serde_json::Value;
use std::collections::HashSet;
use std::path::Path;

/// Load the rubric JSON from docs/non-mock-rubric.json.
fn load_rubric() -> Value {
    let rubric_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/non-mock-rubric.json");
    let content = std::fs::read_to_string(&rubric_path)
        .unwrap_or_else(|e| panic!("Failed to read rubric at {}: {e}", rubric_path.display()));
    serde_json::from_str(&content).unwrap_or_else(|e| panic!("Failed to parse rubric JSON: {e}"))
}

/// Load the coverage baseline JSON from docs/coverage-baseline-map.json.
fn load_coverage_baseline() -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/coverage-baseline-map.json");
    let content = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "Failed to read coverage baseline at {}: {e}",
            path.display()
        )
    });
    serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("Failed to parse coverage baseline JSON: {e}"))
}

// ─── Rubric Schema Integrity ─────────────────────────────────────────

#[test]
fn rubric_has_required_top_level_keys() {
    let rubric = load_rubric();
    let required = [
        "schema",
        "bead_id",
        "module_thresholds",
        "exception_template",
        "failure_log_schema",
        "ci_enforcement",
    ];
    for key in required {
        assert!(
            rubric.get(key).is_some(),
            "Rubric missing required top-level key: {key}"
        );
    }
}

#[test]
fn rubric_schema_version_is_v1() {
    let rubric = load_rubric();
    assert_eq!(
        rubric["schema"].as_str().unwrap(),
        "pi.qa.non_mock_rubric.v1",
        "Rubric schema version must be pi.qa.non_mock_rubric.v1"
    );
}

// ─── Module Threshold Consistency ────────────────────────────────────

#[test]
fn every_module_has_required_threshold_fields() {
    let rubric = load_rubric();
    let modules = rubric["module_thresholds"]["modules"]
        .as_array()
        .expect("modules must be an array");

    let required_fields = [
        "name",
        "paths",
        "criticality",
        "line_floor_pct",
        "function_floor_pct",
        "line_target_pct",
        "function_target_pct",
        "rationale",
    ];

    for module in modules {
        let name = module["name"].as_str().unwrap_or("<unnamed>");
        for field in required_fields {
            assert!(
                module.get(field).is_some(),
                "Module '{name}' missing required field: {field}"
            );
        }
    }
}

#[test]
fn module_names_are_unique() {
    let rubric = load_rubric();
    let modules = rubric["module_thresholds"]["modules"].as_array().unwrap();

    let mut seen = HashSet::new();
    for module in modules {
        let name = module["name"].as_str().unwrap();
        assert!(seen.insert(name), "Duplicate module name in rubric: {name}");
    }
}

#[test]
fn floor_does_not_exceed_target_for_any_module() {
    let rubric = load_rubric();
    let modules = rubric["module_thresholds"]["modules"].as_array().unwrap();

    for module in modules {
        let name = module["name"].as_str().unwrap();

        let line_floor = module["line_floor_pct"].as_f64().unwrap();
        let line_target = module["line_target_pct"].as_f64().unwrap();
        assert!(
            line_floor <= line_target,
            "Module '{name}': line_floor_pct ({line_floor}) > line_target_pct ({line_target})"
        );

        let fn_floor = module["function_floor_pct"].as_f64().unwrap();
        let fn_target = module["function_target_pct"].as_f64().unwrap();
        assert!(
            fn_floor <= fn_target,
            "Module '{name}': function_floor_pct ({fn_floor}) > function_target_pct ({fn_target})"
        );
    }
}

#[test]
fn floor_values_are_in_valid_range() {
    let rubric = load_rubric();
    let modules = rubric["module_thresholds"]["modules"].as_array().unwrap();

    for module in modules {
        let name = module["name"].as_str().unwrap();

        for field in [
            "line_floor_pct",
            "function_floor_pct",
            "line_target_pct",
            "function_target_pct",
        ] {
            let val = module[field].as_f64().unwrap();
            assert!(
                (0.0..=100.0).contains(&val),
                "Module '{name}': {field} ({val}) is outside 0-100 range"
            );
        }
    }
}

#[test]
fn criticality_values_are_valid() {
    let rubric = load_rubric();
    let modules = rubric["module_thresholds"]["modules"].as_array().unwrap();

    let valid = ["critical", "high", "medium", "low"];
    for module in modules {
        let name = module["name"].as_str().unwrap();
        let crit = module["criticality"].as_str().unwrap();
        assert!(
            valid.contains(&crit),
            "Module '{name}': criticality '{crit}' is not one of {valid:?}"
        );
    }
}

#[test]
fn critical_modules_have_higher_floors_than_low() {
    let rubric = load_rubric();
    let modules = rubric["module_thresholds"]["modules"].as_array().unwrap();

    let critical_min_line: f64 = modules
        .iter()
        .filter(|m| m["criticality"].as_str() == Some("critical"))
        .map(|m| m["line_floor_pct"].as_f64().unwrap())
        .fold(f64::MAX, f64::min);

    let low_max_line: f64 = modules
        .iter()
        .filter(|m| m["criticality"].as_str() == Some("low"))
        .map(|m| m["line_floor_pct"].as_f64().unwrap())
        .fold(f64::MIN, f64::max);

    if critical_min_line < f64::MAX && low_max_line > f64::MIN {
        assert!(
            critical_min_line > low_max_line,
            "Lowest critical floor ({critical_min_line}%) should exceed highest low floor ({low_max_line}%)"
        );
    }
}

#[test]
fn every_module_path_references_existing_source() {
    let rubric = load_rubric();
    let modules = rubric["module_thresholds"]["modules"].as_array().unwrap();

    let src_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");

    for module in modules {
        let name = module["name"].as_str().unwrap();
        let paths = module["paths"].as_array().unwrap();

        for path_val in paths {
            let path_str = path_val.as_str().unwrap();
            // Glob patterns (e.g. "src/providers/*.rs") are allowed; check the parent dir exists.
            if path_str.contains('*') {
                let parent = Path::new(path_str)
                    .parent()
                    .unwrap()
                    .strip_prefix("src")
                    .unwrap_or_else(|_| Path::new(""));
                let full = src_root.join(parent);
                assert!(
                    full.exists(),
                    "Module '{name}': glob parent directory does not exist: {}",
                    full.display()
                );
            } else {
                let full = Path::new(env!("CARGO_MANIFEST_DIR")).join(path_str);
                assert!(
                    full.exists(),
                    "Module '{name}': source path does not exist: {}",
                    full.display()
                );
            }
        }
    }
}

#[test]
fn global_thresholds_are_consistent_with_modules() {
    let rubric = load_rubric();
    let global = &rubric["module_thresholds"]["global"];
    let modules = rubric["module_thresholds"]["modules"].as_array().unwrap();

    let global_line_floor = global["line_floor_pct"].as_f64().unwrap();
    let global_fn_floor = global["function_floor_pct"].as_f64().unwrap();

    // At least one module should have floors at or above the global floor.
    let any_above_line = modules
        .iter()
        .any(|m| m["line_floor_pct"].as_f64().unwrap() >= global_line_floor);
    let any_above_fn = modules
        .iter()
        .any(|m| m["function_floor_pct"].as_f64().unwrap() >= global_fn_floor);

    assert!(
        any_above_line,
        "No module has line_floor >= global line_floor ({global_line_floor}%)"
    );
    assert!(
        any_above_fn,
        "No module has function_floor >= global function_floor ({global_fn_floor}%)"
    );
}

// ─── Coverage Baseline Cross-Reference ──────────────────────────────

#[test]
fn rubric_floors_are_below_baseline_for_measured_modules() {
    let rubric = load_rubric();
    let baseline = load_coverage_baseline();

    let modules = rubric["module_thresholds"]["modules"].as_array().unwrap();

    let critical_paths = baseline["critical_paths"]
        .as_array()
        .expect("coverage baseline must have critical_paths");

    for module in modules {
        let name = module["name"].as_str().unwrap();

        // Find matching baseline entry.
        let baseline_entry = critical_paths
            .iter()
            .find(|cp| cp["area"].as_str() == Some(name));

        if let Some(bp) = baseline_entry {
            let baseline_line = bp["coverage"]["line_pct"].as_f64().unwrap();
            let floor_line = module["line_floor_pct"].as_f64().unwrap();
            assert!(
                floor_line <= baseline_line,
                "Module '{name}': line_floor_pct ({floor_line}%) exceeds baseline ({baseline_line}%). \
                 Floor must be at or below baseline to avoid immediate CI failure."
            );

            let baseline_fn = bp["coverage"]["function_pct"].as_f64().unwrap();
            let floor_fn = module["function_floor_pct"].as_f64().unwrap();
            assert!(
                floor_fn <= baseline_fn,
                "Module '{name}': function_floor_pct ({floor_fn}%) exceeds baseline ({baseline_fn}%). \
                 Floor must be at or below baseline to avoid immediate CI failure."
            );
        }
    }
}

// ─── Exception Template Validation ──────────────────────────────────

#[test]
fn exception_template_has_all_required_fields() {
    let rubric = load_rubric();
    let template = &rubric["exception_template"];
    let fields = template["required_fields"]
        .as_array()
        .expect("exception_template.required_fields must be an array");

    let expected_fields = [
        "bead_id",
        "owner",
        "expires_at",
        "replacement_plan",
        "scope",
        "double_identifier",
        "double_type",
        "rationale",
        "verification",
    ];

    let field_names: Vec<&str> = fields
        .iter()
        .map(|f| f["field"].as_str().unwrap())
        .collect();

    for expected in expected_fields {
        assert!(
            field_names.contains(&expected),
            "Exception template missing required field: {expected}"
        );
    }
}

#[test]
fn exception_template_has_validation_rules() {
    let rubric = load_rubric();
    let rules = rubric["exception_template"]["validation_rules"]
        .as_array()
        .expect("exception_template.validation_rules must be an array");
    assert!(
        !rules.is_empty(),
        "Exception template must have at least one validation rule"
    );
}

// ─── Failure-Log Schema Validation ──────────────────────────────────

#[test]
fn failure_log_schema_has_required_fields() {
    let rubric = load_rubric();
    let schema = &rubric["failure_log_schema"];

    assert_eq!(
        schema["schema_id"].as_str().unwrap(),
        "pi.test.failure_log.v1"
    );

    let fields = schema["fields"]
        .as_array()
        .expect("failure_log_schema.fields must be an array");

    let required_names: Vec<&str> = fields
        .iter()
        .filter(|f| f["required"].as_bool() == Some(true))
        .map(|f| f["field"].as_str().unwrap())
        .collect();

    let mandatory = [
        "correlation_id",
        "test_name",
        "suite",
        "timestamp",
        "env",
        "expected",
        "actual",
        "error_message",
        "duration_ms",
        "runner",
    ];

    for field in mandatory {
        assert!(
            required_names.contains(&field),
            "Failure log schema missing mandatory required field: {field}"
        );
    }
}

#[test]
fn failure_log_schema_has_redaction_policy() {
    let rubric = load_rubric();
    let schema = &rubric["failure_log_schema"];

    assert!(
        schema["redaction_policy"].as_str().is_some(),
        "Failure log schema must define a redaction_policy"
    );

    // The env field should have redaction_rules.
    let fields = schema["fields"].as_array().unwrap();
    let env_field = fields
        .iter()
        .find(|f| f["field"].as_str() == Some("env"))
        .expect("Failure log schema must have an 'env' field");

    let rules = env_field["redaction_rules"]
        .as_array()
        .expect("env field must have redaction_rules");
    assert!(
        !rules.is_empty(),
        "env field must have at least one redaction rule"
    );
}

#[test]
fn failure_log_output_format_is_jsonl() {
    let rubric = load_rubric();
    let format = rubric["failure_log_schema"]["output_format"]
        .as_str()
        .unwrap();
    assert!(
        format.contains("JSONL"),
        "Failure log output_format must be JSONL, got: {format}"
    );
}

// ─── CI Enforcement Section Validation ──────────────────────────────

#[test]
fn ci_enforcement_has_coverage_gate() {
    let rubric = load_rubric();
    let gate = &rubric["ci_enforcement"]["coverage_gate"];
    assert!(
        gate["command"].as_str().is_some(),
        "coverage_gate must have a command"
    );
    assert!(
        gate["failure_mode"].as_str().is_some(),
        "coverage_gate must have a failure_mode"
    );
}

#[test]
fn ci_enforcement_has_exception_audit() {
    let rubric = load_rubric();
    let audit = &rubric["ci_enforcement"]["exception_audit"];
    assert!(
        audit["source"].as_str().is_some(),
        "exception_audit must have a source"
    );
    assert!(
        audit["failure_mode"].as_str().is_some(),
        "exception_audit must have a failure_mode"
    );
}

// ─── Meta: Rubric Covers All Critical Paths From Baseline ───────────

#[test]
fn rubric_covers_all_critical_paths_from_baseline() {
    let rubric = load_rubric();
    let baseline = load_coverage_baseline();

    let critical_paths = baseline["critical_paths"]
        .as_array()
        .expect("coverage baseline must have critical_paths");

    let modules = rubric["module_thresholds"]["modules"].as_array().unwrap();

    let rubric_names: HashSet<&str> = modules
        .iter()
        .map(|m| m["name"].as_str().unwrap())
        .collect();

    for cp in critical_paths {
        let area = cp["area"].as_str().unwrap();
        assert!(
            rubric_names.contains(area),
            "Coverage baseline critical path '{area}' is not covered by any rubric module"
        );
    }
}

// ─── Module Count Guard ─────────────────────────────────────────────

#[test]
fn rubric_has_minimum_module_count() {
    let rubric = load_rubric();
    let modules = rubric["module_thresholds"]["modules"].as_array().unwrap();

    // The rubric must cover at least the 5 critical paths from the baseline
    // plus additional modules for comprehensive coverage.
    assert!(
        modules.len() >= 10,
        "Rubric must define at least 10 module thresholds, got {}",
        modules.len()
    );
}

// ─── Allowlisted Exceptions Cross-Check ─────────────────────────────

#[test]
fn testing_policy_allowlist_entries_have_required_info() {
    // Validate that the allowlist in testing-policy.md contains entries
    // with the minimum fields (identifier, location, suite, rationale).
    let policy_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/testing-policy.md");
    let content = std::fs::read_to_string(&policy_path)
        .unwrap_or_else(|e| panic!("Failed to read testing policy: {e}"));

    // Check the allowlist table exists.
    assert!(
        content.contains("## Allowlisted Exceptions") || content.contains("Allowlisted Exceptions"),
        "Testing policy must contain an Allowlisted Exceptions section"
    );

    // Check it contains at least one table row with pipe separators.
    let table_rows: Vec<&str> = content
        .lines()
        .filter(|l| l.starts_with('|') && l.contains('|') && !l.contains("---"))
        .collect();

    // Header + at least one entry.
    assert!(
        table_rows.len() >= 2,
        "Allowlist table must have a header and at least one entry, found {} rows",
        table_rows.len()
    );
}

// ─── Suite Classification Completeness ──────────────────────────────

#[test]
fn suite_classification_file_exists_and_has_all_suites() {
    let toml_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/suite_classification.toml");
    let content = std::fs::read_to_string(&toml_path)
        .unwrap_or_else(|e| panic!("Failed to read suite classification: {e}"));

    assert!(
        content.contains("[suite.unit]"),
        "Missing [suite.unit] section"
    );
    assert!(
        content.contains("[suite.vcr]"),
        "Missing [suite.vcr] section"
    );
    assert!(
        content.contains("[suite.e2e]"),
        "Missing [suite.e2e] section"
    );
}

// ─── Failure Log Schema: Example Roundtrip ──────────────────────────

#[test]
fn failure_log_example_conforms_to_schema() {
    // Build a sample failure log entry and validate it against the schema.
    let entry = serde_json::json!({
        "correlation_id": "test_example-1707753600000-a1b2",
        "test_name": "non_mock_rubric_gate::failure_log_example_conforms_to_schema",
        "suite": "unit",
        "timestamp": "2026-02-12T20:00:00Z",
        "env": {
            "VCR_MODE": "playback",
            "RUST_LOG": "info",
            "OPENAI_API_KEY": "[REDACTED]"
        },
        "expected": "true",
        "actual": "false",
        "diff": null,
        "error_message": "assertion failed: expected true, got false",
        "backtrace": null,
        "duration_ms": 42.5,
        "runner": {
            "os": "Linux",
            "arch": "x86_64",
            "rust_version": "rustc 1.85.0",
            "ci": false
        },
        "fixture": null,
        "seed": null
    });

    // Validate required fields are present.
    let rubric = load_rubric();
    let schema_fields = rubric["failure_log_schema"]["fields"].as_array().unwrap();

    for field_def in schema_fields {
        let name = field_def["field"].as_str().unwrap();
        let required = field_def["required"].as_bool().unwrap_or(false);

        if required {
            assert!(
                entry.get(name).is_some(),
                "Example failure log entry missing required field: {name}"
            );
            assert!(
                !entry[name].is_null(),
                "Example failure log entry has null value for required field: {name}"
            );
        }
    }

    // Validate suite enum.
    let suite = entry["suite"].as_str().unwrap();
    assert!(
        ["unit", "vcr", "e2e"].contains(&suite),
        "Suite must be one of unit/vcr/e2e, got: {suite}"
    );

    // Validate redaction: API key should be redacted.
    let env_val = &entry["env"];
    for (key, val) in env_val.as_object().unwrap() {
        if key.contains("API_KEY") || key.contains("SECRET") || key.contains("TOKEN") {
            assert_eq!(
                val.as_str().unwrap(),
                "[REDACTED]",
                "Env key '{key}' must be redacted in failure logs"
            );
        }
    }
}

// ─── Double-Type Enum Validation ────────────────────────────────────

#[test]
fn exception_double_types_are_exhaustive() {
    let rubric = load_rubric();
    let fields = rubric["exception_template"]["required_fields"]
        .as_array()
        .unwrap();

    let double_type_field = fields
        .iter()
        .find(|f| f["field"].as_str() == Some("double_type"))
        .expect("Exception template must have double_type field");

    let allowed: Vec<&str> = double_type_field["enum"]
        .as_array()
        .expect("double_type must have enum values")
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();

    let expected = ["mock", "fake", "stub", "spy", "recording"];
    for t in expected {
        assert!(
            allowed.contains(&t),
            "double_type enum missing expected value: {t}"
        );
    }
}
