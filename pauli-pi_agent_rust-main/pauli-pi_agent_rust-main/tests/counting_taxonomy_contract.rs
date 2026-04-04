#![forbid(unsafe_code)]

use serde_json::Value;
use std::collections::HashSet;
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::tempdir;

const GENERATOR_SCRIPT: &str = "scripts/ci/generate_parity_evidence.py";
const VALIDATOR_SCRIPT: &str = "scripts/ci/validate_counting_taxonomy.py";
const CONTRACT_PATH: &str = "docs/counting-taxonomy-contract.json";

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn script_output_debug(output: &Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    format!(
        "status={:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        stdout,
        stderr
    )
}

fn run_python_script(script_rel_path: &str, args: &[&str]) -> Output {
    Command::new("python3")
        .current_dir(repo_root())
        .arg(script_rel_path)
        .args(args)
        .output()
        .expect("python3 script execution should succeed")
}

fn write_fixture_parity_log(path: &Path) {
    let suites = [
        "json_mode_parity",
        "cross_surface_parity",
        "config_precedence",
        "vcr_parity_validation",
        "e2e_cross_provider_parity",
    ];

    let mut lines = String::new();
    for suite in suites {
        let _ = writeln!(
            lines,
            "Running tests/{suite}.rs (target/debug/deps/{suite}-abcdef)"
        );
        lines.push_str(
            "test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s\n",
        );
    }

    fs::write(path, lines).expect("should write parity fixture log");
}

fn load_json(path: &Path) -> Value {
    serde_json::from_str(&fs::read_to_string(path).expect("json file should exist"))
        .expect("json should parse")
}

fn required_labels(contract: &Value, dimension: &str) -> HashSet<String> {
    contract["required_dimensions"][dimension]["required_granularity_labels"]
        .as_array()
        .expect("required_granularity_labels should be array")
        .iter()
        .map(|v| {
            v.as_str()
                .expect("granularity label should be string")
                .to_string()
        })
        .collect()
}

#[test]
fn counting_taxonomy_contract_declares_required_dimensions_and_labels() {
    let contract_path = repo_root().join(CONTRACT_PATH);
    let contract = load_json(&contract_path);

    assert_eq!(
        contract["schema"].as_str(),
        Some("pi.qa.counting_taxonomy_contract.v1")
    );
    assert_eq!(
        contract["taxonomy_schema"].as_str(),
        Some("pi.qa.counting_taxonomy.v1")
    );

    let dimensions = contract["required_dimensions"]
        .as_object()
        .expect("required_dimensions should be object");
    assert!(dimensions.contains_key("loc"));
    assert!(dimensions.contains_key("providers"));
    assert!(dimensions.contains_key("extensions"));

    assert_eq!(
        required_labels(&contract, "loc"),
        HashSet::from(["loc_raw_lines".to_string(), "loc_logical_lines".to_string()])
    );
    assert_eq!(
        required_labels(&contract, "providers"),
        HashSet::from([
            "provider_canonical_ids".to_string(),
            "provider_alias_ids".to_string(),
            "provider_families".to_string()
        ])
    );
    assert_eq!(
        required_labels(&contract, "extensions"),
        HashSet::from([
            "extension_official_subset".to_string(),
            "extension_community_subset".to_string(),
            "extension_full_corpus".to_string()
        ])
    );
}

#[test]
fn parity_evidence_emits_counting_taxonomy_and_validator_accepts() {
    let dir = tempdir().expect("tempdir");
    let log_path = dir.path().join("parity.log");
    let artifact_path = dir.path().join("parity_evidence.json");
    let contract_path = repo_root().join(CONTRACT_PATH);

    write_fixture_parity_log(&log_path);

    let generate = run_python_script(
        GENERATOR_SCRIPT,
        &[
            "--log",
            &log_path.to_string_lossy(),
            "--output",
            &artifact_path.to_string_lossy(),
        ],
    );
    assert!(
        generate.status.success(),
        "generator failed:\n{}",
        script_output_debug(&generate)
    );

    let evidence = load_json(&artifact_path);
    let contract = load_json(&contract_path);

    let taxonomy = evidence["counting_taxonomy"]
        .as_object()
        .expect("counting_taxonomy must exist");
    assert_eq!(
        taxonomy.get("schema").and_then(Value::as_str),
        Some("pi.qa.counting_taxonomy.v1")
    );

    let dimensions = taxonomy
        .get("dimensions")
        .and_then(Value::as_object)
        .expect("counting_taxonomy.dimensions must be object");

    for dim_name in ["loc", "providers", "extensions"] {
        let metrics = dimensions
            .get(dim_name)
            .and_then(|v| v.get("metrics"))
            .and_then(Value::as_array)
            .expect("dimension metrics should exist");

        let labels: HashSet<String> = metrics
            .iter()
            .filter_map(|metric| metric.get("granularity_label"))
            .filter_map(Value::as_str)
            .map(ToString::to_string)
            .collect();

        let required = required_labels(&contract, dim_name);
        assert!(
            required.is_subset(&labels),
            "dimension {dim_name} missing labels. required={required:?} got={labels:?}"
        );
    }

    let validate = run_python_script(
        VALIDATOR_SCRIPT,
        &[
            "--artifact",
            &artifact_path.to_string_lossy(),
            "--contract",
            &contract_path.to_string_lossy(),
        ],
    );
    assert!(
        validate.status.success(),
        "validator should pass on generated artifact:\n{}",
        script_output_debug(&validate)
    );
}

#[test]
fn counting_taxonomy_validator_rejects_missing_required_labels() {
    let dir = tempdir().expect("tempdir");
    let log_path = dir.path().join("parity.log");
    let valid_artifact_path = dir.path().join("valid.json");
    let invalid_artifact_path = dir.path().join("invalid.json");
    let contract_path = repo_root().join(CONTRACT_PATH);

    write_fixture_parity_log(&log_path);

    let generate = run_python_script(
        GENERATOR_SCRIPT,
        &[
            "--log",
            &log_path.to_string_lossy(),
            "--output",
            &valid_artifact_path.to_string_lossy(),
        ],
    );
    assert!(
        generate.status.success(),
        "generator failed:\n{}",
        script_output_debug(&generate)
    );

    let mut evidence = load_json(&valid_artifact_path);
    let providers_metrics = evidence["counting_taxonomy"]["dimensions"]["providers"]["metrics"]
        .as_array_mut()
        .expect("providers metrics should exist");
    providers_metrics.retain(|m| {
        m.get("granularity_label").and_then(Value::as_str) != Some("provider_alias_ids")
    });

    fs::write(
        &invalid_artifact_path,
        serde_json::to_string_pretty(&evidence).expect("serialize invalid evidence"),
    )
    .expect("write invalid artifact");

    let validate = run_python_script(
        VALIDATOR_SCRIPT,
        &[
            "--artifact",
            &invalid_artifact_path.to_string_lossy(),
            "--contract",
            &contract_path.to_string_lossy(),
        ],
    );
    assert!(
        !validate.status.success(),
        "validator should fail when a required label is missing:\n{}",
        script_output_debug(&validate)
    );
    let stderr = String::from_utf8_lossy(&validate.stderr);
    assert!(
        stderr.contains("provider_alias_ids"),
        "validator stderr should mention missing provider_alias_ids label, got:\n{stderr}"
    );
}
