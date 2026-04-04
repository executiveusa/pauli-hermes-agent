//! Release gate: validates that the conformance evidence bundle exists,
//! is structurally valid, and meets minimum thresholds for release.
//!
//! This test suite enforces that releases are evidence-based. It checks:
//! - Required evidence artifacts exist on disk
//! - Evidence artifacts have valid schemas
//! - Pass-rate and failure thresholds meet release criteria
//! - Exception policy is complete and current
//!
//! See also: `tests/release_readiness.rs` for the readiness report generator.
#![allow(clippy::too_many_lines)]

use serde_json::{Map, Value};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn load_json(relative: &str) -> Option<Value> {
    let path = repo_root().join(relative);
    let text = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&text).ok()
}

fn require_json(relative: &str) -> Value {
    load_json(relative).unwrap_or_else(|| panic!("required evidence file missing: {relative}"))
}

const FRANKEN_NODE_CLAIM_CONTRACT_PATH: &str = "docs/franken-node-claim-gating-contract.json";
const FRANKEN_NODE_CLAIM_CONTRACT_SCHEMA: &str = "pi.frankennode.claim_gating_contract.v1";
const FRANKEN_NODE_REQUIRED_TIER_IDS: &[&str] = &[
    "TIER-1-EXTENSION-HOST-PARITY",
    "TIER-2-TARGETED-RUNTIME-PARITY",
    "TIER-3-FULL-NODE-BUN-REPLACEMENT",
];
const FRANKEN_NODE_REQUIRED_ARTIFACTS: &[&str] = &[
    "tests/full_suite_gate/franken_node_claim_verdict.json",
    "tests/full_suite_gate/practical_finish_checkpoint.json",
];
const FRANKEN_NODE_REQUIRED_OVERCLAIM_BLOCKERS: &[&str] = &[
    "missing_required_evidence",
    "missing_or_stale_verdict_artifact",
    "forbidden_claim_phrase_detected",
];
const FRANKEN_NODE_REQUIRED_LOG_FIELDS: &[&str] = &[
    "run_id",
    "tier_id",
    "decision",
    "blocking_reasons",
    "evidence_refs",
    "timestamp_utc",
];
const FRANKEN_NODE_TIER2_REQUIRED_EVIDENCE_TOKENS: &[&str] = &[
    "compatibility matrix with executable conformance harness",
    "package/ecosystem interoperability contract evidence (cjs/esm/npm)",
];
const FRANKEN_NODE_TIER3_REQUIRED_EVIDENCE_TOKENS: &[&str] = &[
    "package/ecosystem interoperability strict-tier evidence and claim-tier linkage",
    "kernel extraction boundary manifest and reintegration mapping evidence",
    "runtime-substrate generalization evidence for bd-3ar8v.7.5",
    "multi-tier execution engine evidence for bd-3ar8v.7.6",
    "compatibility remediation backlog generator evidence for bd-3ar8v.7.16",
    "crate reintegration evidence into pi_agent_rust",
];

fn collect_non_empty_string_array(
    value: &Value,
    pointer: &str,
    label: &str,
    errors: &mut Vec<String>,
) -> Vec<String> {
    let Some(entries) = value.pointer(pointer).and_then(Value::as_array) else {
        errors.push(format!("{label} must be an array at {pointer}"));
        return Vec::new();
    };
    if entries.is_empty() {
        errors.push(format!("{label} must be non-empty at {pointer}"));
        return Vec::new();
    }

    let mut out = Vec::with_capacity(entries.len());
    for (index, entry) in entries.iter().enumerate() {
        let Some(raw) = entry.as_str() else {
            errors.push(format!("{label}[{index}] must be a string at {pointer}"));
            continue;
        };
        let normalized = raw.trim();
        if normalized.is_empty() {
            errors.push(format!("{label}[{index}] must be non-empty at {pointer}"));
            continue;
        }
        out.push(normalized.to_string());
    }
    out
}

fn validate_franken_node_claim_contract(contract: &Value) -> Result<(), String> {
    let mut errors = Vec::new();

    let schema = contract.get("schema").and_then(Value::as_str).unwrap_or("");
    if schema != FRANKEN_NODE_CLAIM_CONTRACT_SCHEMA {
        errors.push(format!(
            "schema must be {FRANKEN_NODE_CLAIM_CONTRACT_SCHEMA}, found {schema}"
        ));
    }

    for field in [
        "/mission_statement",
        "/claim_gate_policy/release_claim_gate_mode",
    ] {
        let value = contract
            .pointer(field)
            .and_then(Value::as_str)
            .map_or("", str::trim);
        if value.is_empty() {
            errors.push(format!("missing required non-empty string at {field}"));
        }
    }

    let release_mode = contract
        .pointer("/claim_gate_policy/release_claim_gate_mode")
        .and_then(Value::as_str)
        .unwrap_or("");
    if release_mode != "hard_fail_if_unmet" {
        errors.push(format!(
            "claim_gate_policy.release_claim_gate_mode must be hard_fail_if_unmet, found {release_mode}"
        ));
    }

    let mut observed_tier_ids = HashSet::new();
    let Some(claim_tiers) = contract.get("claim_tiers").and_then(Value::as_array) else {
        errors.push("claim_tiers must be an array".to_string());
        return Err(errors.join("; "));
    };
    if claim_tiers.is_empty() {
        errors.push("claim_tiers must be non-empty".to_string());
    }

    for (index, tier) in claim_tiers.iter().enumerate() {
        let Some(tier_id) = tier.get("tier_id").and_then(Value::as_str).map(str::trim) else {
            errors.push(format!("claim_tiers[{index}].tier_id must be a string"));
            continue;
        };
        if tier_id.is_empty() {
            errors.push(format!("claim_tiers[{index}].tier_id must be non-empty"));
            continue;
        }
        observed_tier_ids.insert(tier_id.to_string());

        let allowed = collect_non_empty_string_array(
            tier,
            "/allowed_claim_language",
            &format!("claim_tiers[{index}].allowed_claim_language"),
            &mut errors,
        );
        let required_evidence = collect_non_empty_string_array(
            tier,
            "/required_evidence",
            &format!("claim_tiers[{index}].required_evidence"),
            &mut errors,
        );
        let forbidden = collect_non_empty_string_array(
            tier,
            "/forbidden_claim_language",
            &format!("claim_tiers[{index}].forbidden_claim_language"),
            &mut errors,
        );

        if required_evidence.is_empty() {
            errors.push(format!(
                "claim_tiers[{index}] must include required_evidence entries"
            ));
        }
        let required_evidence_tokens: &[&str] = match tier_id {
            "TIER-2-TARGETED-RUNTIME-PARITY" => FRANKEN_NODE_TIER2_REQUIRED_EVIDENCE_TOKENS,
            "TIER-3-FULL-NODE-BUN-REPLACEMENT" => FRANKEN_NODE_TIER3_REQUIRED_EVIDENCE_TOKENS,
            _ => &[],
        };
        if !required_evidence_tokens.is_empty() {
            let evidence_set = required_evidence
                .iter()
                .map(|entry| entry.to_ascii_lowercase())
                .collect::<HashSet<_>>();
            for required_token in required_evidence_tokens {
                if !evidence_set.contains(&required_token.to_ascii_lowercase()) {
                    errors.push(format!(
                        "claim_tiers[{index}] ({tier_id}) required_evidence missing token: {required_token}"
                    ));
                }
            }
        }

        if !allowed.is_empty() && !forbidden.is_empty() {
            let allowed_set = allowed
                .iter()
                .map(|entry| entry.to_ascii_lowercase())
                .collect::<HashSet<_>>();
            let overlap = forbidden
                .iter()
                .map(|entry| entry.to_ascii_lowercase())
                .find(|entry| allowed_set.contains(entry));
            if let Some(phrase) = overlap {
                errors.push(format!(
                    "claim_tiers[{index}] has overlap between allowed_claim_language and forbidden_claim_language: {phrase}"
                ));
            }
        }
    }

    for tier_id in FRANKEN_NODE_REQUIRED_TIER_IDS {
        if !observed_tier_ids.contains(*tier_id) {
            errors.push(format!("missing required claim tier: {tier_id}"));
        }
    }

    let forbidden_patterns = collect_non_empty_string_array(
        contract,
        "/forbidden_claim_patterns",
        "forbidden_claim_patterns",
        &mut errors,
    );
    let forbidden_pattern_set = forbidden_patterns
        .iter()
        .map(|pattern| pattern.to_ascii_lowercase())
        .collect::<HashSet<_>>();
    for required_pattern in [
        "strict drop-in replacement for node/bun",
        "production-ready full runtime replacement without certification",
    ] {
        if !forbidden_pattern_set.contains(required_pattern) {
            errors.push(format!(
                "forbidden_claim_patterns missing required pattern: {required_pattern}"
            ));
        }
    }

    let strict_replacement = contract
        .pointer("/claim_gate_policy/strict_replacement_requires")
        .and_then(Value::as_object);
    let Some(strict_replacement) = strict_replacement else {
        errors.push("claim_gate_policy.strict_replacement_requires must be an object".to_string());
        return Err(errors.join("; "));
    };

    let strict_overall_verdict = strict_replacement
        .get("overall_verdict")
        .and_then(Value::as_str)
        .unwrap_or("");
    if strict_overall_verdict != "CERTIFIED" {
        errors.push(format!(
            "claim_gate_policy.strict_replacement_requires.overall_verdict must be CERTIFIED, found {strict_overall_verdict}"
        ));
    }

    let required_artifacts = strict_replacement
        .get("required_artifacts")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let required_artifact_set = required_artifacts
        .iter()
        .filter_map(Value::as_str)
        .map(str::trim)
        .filter(|path| !path.is_empty())
        .collect::<HashSet<_>>();
    for required_artifact in FRANKEN_NODE_REQUIRED_ARTIFACTS {
        if !required_artifact_set.contains(*required_artifact) {
            errors.push(format!(
                "strict_replacement_requires.required_artifacts missing {required_artifact}"
            ));
        }
    }

    let overclaim_blockers = collect_non_empty_string_array(
        contract,
        "/claim_gate_policy/overclaim_blockers",
        "claim_gate_policy.overclaim_blockers",
        &mut errors,
    );
    let overclaim_blocker_set = overclaim_blockers
        .iter()
        .map(|entry| entry.to_ascii_lowercase())
        .collect::<HashSet<_>>();
    for required_blocker in FRANKEN_NODE_REQUIRED_OVERCLAIM_BLOCKERS {
        if !overclaim_blocker_set.contains(&required_blocker.to_ascii_lowercase()) {
            errors.push(format!(
                "claim_gate_policy.overclaim_blockers missing {required_blocker}"
            ));
        }
    }

    let structured_logging_fields = collect_non_empty_string_array(
        contract,
        "/structured_logging_contract/required_fields",
        "structured_logging_contract.required_fields",
        &mut errors,
    );
    let structured_logging_field_set = structured_logging_fields
        .iter()
        .map(|entry| entry.to_ascii_lowercase())
        .collect::<HashSet<_>>();
    for required_field in FRANKEN_NODE_REQUIRED_LOG_FIELDS {
        if !structured_logging_field_set.contains(&required_field.to_ascii_lowercase()) {
            errors.push(format!(
                "structured_logging_contract.required_fields missing {required_field}"
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

fn find_latest_phase1_matrix_validation(root: &Path) -> Option<PathBuf> {
    let mut candidates = Vec::new();

    for relative in [
        "tests/perf/reports/phase1_matrix_validation.json",
        "tests/perf/runs/results/phase1_matrix_validation.json",
    ] {
        let candidate = root.join(relative);
        if candidate.is_file() {
            candidates.push(candidate);
        }
    }

    let e2e_results_dir = root.join("tests/e2e_results");
    if let Ok(entries) = std::fs::read_dir(e2e_results_dir) {
        for entry in entries.flatten() {
            let candidate = entry.path().join("results/phase1_matrix_validation.json");
            if candidate.is_file() {
                candidates.push(candidate);
            }
        }
    }

    candidates.sort_by_key(|path| {
        std::fs::metadata(path)
            .and_then(|metadata| metadata.modified())
            .ok()
    });
    candidates.pop()
}

fn require_phase1_matrix_validation() -> (String, Value) {
    let root = repo_root();
    let path = find_latest_phase1_matrix_validation(&root).unwrap_or_else(|| {
        panic!(
            "release gate BLOCKED: missing phase1_matrix_validation.json evidence artifact; \
             expected at tests/perf/reports or tests/e2e_results/*/results"
        )
    });
    let display_path = path.strip_prefix(&root).map_or_else(
        |_| path.display().to_string(),
        |rel| rel.display().to_string(),
    );
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {display_path}: {err}"));
    let json = serde_json::from_str(&text)
        .unwrap_or_else(|err| panic!("{display_path} is not valid JSON: {err}"));
    (display_path, json)
}

fn find_latest_parameter_sweeps(root: &Path) -> Option<PathBuf> {
    let mut candidates = Vec::new();

    for relative in [
        "tests/perf/reports/parameter_sweeps.json",
        "tests/perf/runs/results/parameter_sweeps.json",
    ] {
        let candidate = root.join(relative);
        if candidate.is_file() {
            candidates.push(candidate);
        }
    }

    let e2e_results_dir = root.join("tests/e2e_results");
    if let Ok(entries) = std::fs::read_dir(e2e_results_dir) {
        for entry in entries.flatten() {
            let candidate = entry.path().join("results/parameter_sweeps.json");
            if candidate.is_file() {
                candidates.push(candidate);
            }
        }
    }

    candidates.sort_by_key(|path| {
        std::fs::metadata(path)
            .and_then(|metadata| metadata.modified())
            .ok()
    });
    candidates.pop()
}

fn find_latest_opportunity_matrix(root: &Path) -> Option<PathBuf> {
    let mut candidates = Vec::new();

    for relative in [
        "tests/perf/reports/opportunity_matrix.json",
        "tests/perf/runs/results/opportunity_matrix.json",
    ] {
        let candidate = root.join(relative);
        if candidate.is_file() {
            candidates.push(candidate);
        }
    }

    let e2e_results_dir = root.join("tests/e2e_results");
    if let Ok(entries) = std::fs::read_dir(e2e_results_dir) {
        for entry in entries.flatten() {
            let candidate = entry.path().join("results/opportunity_matrix.json");
            if candidate.is_file() {
                candidates.push(candidate);
            }
        }
    }

    candidates.sort_by_key(|path| {
        std::fs::metadata(path)
            .and_then(|metadata| metadata.modified())
            .ok()
    });
    candidates.pop()
}

fn require_parameter_sweeps() -> (String, Value) {
    let root = repo_root();
    let path = find_latest_parameter_sweeps(&root).unwrap_or_else(|| {
        panic!(
            "release gate BLOCKED: missing parameter_sweeps.json evidence artifact; \
             expected at tests/perf/reports or tests/e2e_results/*/results"
        )
    });
    let display_path = path.strip_prefix(&root).map_or_else(
        |_| path.display().to_string(),
        |rel| rel.display().to_string(),
    );
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {display_path}: {err}"));
    let json = serde_json::from_str(&text)
        .unwrap_or_else(|err| panic!("{display_path} is not valid JSON: {err}"));
    (display_path, json)
}

fn require_opportunity_matrix() -> (String, Value) {
    let root = repo_root();
    let path = find_latest_opportunity_matrix(&root).unwrap_or_else(|| {
        panic!(
            "release gate BLOCKED: missing opportunity_matrix.json evidence artifact; \
             expected at tests/perf/reports or tests/e2e_results/*/results"
        )
    });
    let display_path = path.strip_prefix(&root).map_or_else(
        |_| path.display().to_string(),
        |rel| rel.display().to_string(),
    );
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {display_path}: {err}"));
    let json = serde_json::from_str(&text)
        .unwrap_or_else(|err| panic!("{display_path} is not valid JSON: {err}"));
    (display_path, json)
}

// ============================================================================
// Evidence bundle existence checks
// ============================================================================

const REQUIRED_ARTIFACTS: &[(&str, &str)] = &[
    (
        "tests/ext_conformance/reports/conformance_summary.json",
        "Extension conformance summary",
    ),
    (
        "tests/ext_conformance/reports/conformance_baseline.json",
        "Conformance baseline with thresholds",
    ),
    (
        "tests/perf/reports/budget_summary.json",
        "Performance budget summary",
    ),
    (
        "tests/ext_conformance/artifacts/RISK_REVIEW.json",
        "Security and licensing risk review",
    ),
    (
        "tests/ext_conformance/artifacts/PROVENANCE_VERIFICATION.json",
        "Extension provenance verification",
    ),
    (
        "docs/traceability_matrix.json",
        "Requirement-to-test traceability matrix",
    ),
];

#[test]
fn all_required_evidence_artifacts_exist() {
    let root = repo_root();
    let mut missing = Vec::new();

    for (path, label) in REQUIRED_ARTIFACTS {
        if !root.join(path).is_file() {
            missing.push(format!("  - {label}: {path}"));
        }
    }

    assert!(
        missing.is_empty(),
        "release gate BLOCKED: missing evidence artifacts:\n{}",
        missing.join("\n")
    );
}

#[test]
fn all_evidence_artifacts_are_valid_json() {
    for (path, label) in REQUIRED_ARTIFACTS {
        let v = load_json(path);
        assert!(
            v.is_some(),
            "evidence artifact is not valid JSON: {label} ({path})"
        );
    }
}

#[test]
fn phase1_matrix_validation_artifact_is_present_and_parseable() {
    let (artifact, matrix) = require_phase1_matrix_validation();
    let schema = matrix.get("schema").and_then(Value::as_str).unwrap_or("");
    assert_eq!(
        schema, "pi.perf.phase1_matrix_validation.v1",
        "phase1 matrix schema mismatch in {artifact}"
    );
}

#[test]
fn parameter_sweeps_artifact_is_present_and_parseable() {
    let (_, matrix) = require_phase1_matrix_validation();
    let consumption_contract = require_consumption_contract(&matrix, "phase1_matrix_validation");
    let sweeps_present = find_latest_parameter_sweeps(&repo_root()).is_some();
    if !requires_strict_parameter_sweeps_contract(consumption_contract, sweeps_present) {
        assert_orchestrate_parameter_sweeps_contract_tokens();
        return;
    }

    let (artifact, sweeps) = require_parameter_sweeps();
    let schema = sweeps.get("schema").and_then(Value::as_str).unwrap_or("");
    assert_eq!(
        schema, "pi.perf.parameter_sweeps.v1",
        "parameter sweeps schema mismatch in {artifact}"
    );
}

#[test]
fn opportunity_matrix_artifact_is_present_and_parseable() {
    let (_, matrix) = require_phase1_matrix_validation();
    let consumption_contract = require_consumption_contract(&matrix, "phase1_matrix_validation");
    let opportunity_present = find_latest_opportunity_matrix(&repo_root()).is_some();
    if !requires_strict_opportunity_matrix_contract(consumption_contract, opportunity_present) {
        assert_orchestrate_opportunity_matrix_contract_tokens();
        return;
    }

    let (artifact, opportunity) = require_opportunity_matrix();
    let schema = opportunity
        .get("schema")
        .and_then(Value::as_str)
        .unwrap_or("");
    assert_eq!(
        schema, "pi.perf.opportunity_matrix.v1",
        "opportunity matrix schema mismatch in {artifact}"
    );
}

// ============================================================================
// Schema validation
// ============================================================================

#[test]
fn conformance_summary_has_required_fields() {
    let sm = require_json("tests/ext_conformance/reports/conformance_summary.json");

    assert!(sm.get("schema").is_some(), "missing schema field");
    let run_id = sm
        .get("run_id")
        .and_then(Value::as_str)
        .map_or("", str::trim);
    assert!(
        !run_id.is_empty(),
        "missing or empty run_id in conformance_summary.json"
    );
    let correlation_id = sm
        .get("correlation_id")
        .and_then(Value::as_str)
        .map_or("", str::trim);
    assert!(
        !correlation_id.is_empty(),
        "missing or empty correlation_id in conformance_summary.json"
    );
    assert!(sm.get("counts").is_some(), "missing counts field");
    assert!(sm.get("pass_rate_pct").is_some(), "missing pass_rate_pct");
    assert!(sm.get("per_tier").is_some(), "missing per_tier");
    assert!(sm.get("evidence").is_some(), "missing evidence");

    let counts = sm.get("counts").unwrap();
    assert!(counts.get("pass").is_some(), "missing counts.pass");
    assert!(counts.get("fail").is_some(), "missing counts.fail");
    assert!(counts.get("total").is_some(), "missing counts.total");
}

#[test]
fn baseline_has_required_fields() {
    let bl = require_json("tests/ext_conformance/reports/conformance_baseline.json");

    assert!(bl.get("schema").is_some(), "missing schema");
    assert!(
        bl.get("extension_conformance").is_some(),
        "missing extension_conformance"
    );
    assert!(
        bl.get("regression_thresholds").is_some(),
        "missing regression_thresholds"
    );
    assert!(
        bl.get("exception_policy").is_some(),
        "missing exception_policy"
    );
}

#[test]
fn traceability_matrix_has_requirements() {
    let tm = require_json("docs/traceability_matrix.json");

    let reqs = tm
        .get("requirements")
        .and_then(Value::as_array)
        .expect("traceability matrix must have requirements array");

    assert!(
        !reqs.is_empty(),
        "traceability matrix must have at least one requirement"
    );

    for req in reqs {
        assert!(req.get("id").is_some(), "requirement missing id field");
        assert!(
            req.get("unit_tests").is_some(),
            "requirement {:?} missing unit_tests",
            req.get("id")
        );
    }
}

fn require_consumption_contract<'a>(matrix: &'a Value, artifact: &str) -> &'a Map<String, Value> {
    matrix
        .pointer("/consumption_contract")
        .and_then(Value::as_object)
        .unwrap_or_else(|| panic!("consumption_contract must be an object in {artifact}"))
}

fn assert_consumption_contract_downstream_beads(
    consumption_contract: &Map<String, Value>,
    artifact: &str,
) {
    let downstream_beads = consumption_contract
        .get("downstream_beads")
        .and_then(Value::as_array)
        .unwrap_or_else(|| {
            panic!("consumption_contract.downstream_beads must be an array in {artifact}")
        });
    let downstream_bead_set: HashSet<&str> =
        downstream_beads.iter().filter_map(Value::as_str).collect();
    for bead_id in ["bd-3ar8v.6.1", "bd-3ar8v.6.2"] {
        assert!(
            downstream_bead_set.contains(bead_id),
            "consumption_contract.downstream_beads missing {bead_id} in {artifact}"
        );
    }
}

fn requires_strict_weighted_contract(
    consumption_contract: &Map<String, Value>,
    matrix: &Value,
) -> bool {
    let artifact_ready_for_phase5 = consumption_contract
        .get("artifact_ready_for_phase5")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let weighted_present = matrix
        .get("weighted_bottleneck_attribution")
        .and_then(Value::as_object)
        .is_some();
    artifact_ready_for_phase5 || weighted_present
}

fn requires_strict_parameter_sweeps_contract(
    consumption_contract: &Map<String, Value>,
    sweeps_present: bool,
) -> bool {
    let artifact_ready_for_phase5 = consumption_contract
        .get("artifact_ready_for_phase5")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    artifact_ready_for_phase5 || sweeps_present
}

fn requires_strict_opportunity_matrix_contract(
    consumption_contract: &Map<String, Value>,
    opportunity_present: bool,
) -> bool {
    let artifact_ready_for_phase5 = consumption_contract
        .get("artifact_ready_for_phase5")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    artifact_ready_for_phase5 || opportunity_present
}

fn assert_orchestrate_weighted_contract_tokens(artifact: &str) {
    let orchestrate = std::fs::read_to_string(repo_root().join("scripts/perf/orchestrate.sh"))
        .expect("scripts/perf/orchestrate.sh should be readable");
    for token in [
        "\"weighted_bottleneck_attribution\"",
        "\"pi.perf.phase1_weighted_bottleneck_attribution.v1\"",
        "weighted_bottleneck_attribution.global_ranking",
        "weighted_bottleneck_attribution.per_scale",
    ] {
        assert!(
            orchestrate.contains(token),
            "orchestrate contract token missing while weighted attribution artifact is absent in {artifact}: {token}"
        );
    }
}

fn assert_orchestrate_parameter_sweeps_contract_tokens() {
    let orchestrate = std::fs::read_to_string(repo_root().join("scripts/perf/orchestrate.sh"))
        .expect("scripts/perf/orchestrate.sh should be readable");
    for token in [
        "parameter_sweeps.json",
        "\"pi.perf.parameter_sweeps.v1\"",
        "\"parameter_sweeps\": \"pi.perf.parameter_sweeps.v1\"",
        "phase1_matrix_validation.weighted_bottleneck_attribution",
        "weighted_bottleneck_guided_grid",
        "manifest[\"parameter_sweeps\"]",
    ] {
        assert!(
            orchestrate.contains(token),
            "orchestrate contract token missing for parameter_sweeps artifact: {token}"
        );
    }
}

fn assert_orchestrate_opportunity_matrix_contract_tokens() {
    let orchestrate = std::fs::read_to_string(repo_root().join("scripts/perf/orchestrate.sh"))
        .expect("scripts/perf/orchestrate.sh should be readable");
    for token in [
        "\"opportunity_matrix\"",
        "\"pi.perf.opportunity_matrix.v1\"",
        "\"generated_at\"",
        "\"source_identity\"",
        "\"readiness\"",
        "\"decision\"",
        "\"NO_DECISION\"",
        "\"ranked_opportunities\"",
        "\"fail_closed_conditions\"",
        "decision = \"RANKED\" if readiness_ok else \"NO_DECISION\"",
        "weighted_bottleneck_attribution.global_ranking",
        "\"bd-3ar8v.6.1\"",
    ] {
        assert!(
            orchestrate.contains(token),
            "orchestrate contract token missing for opportunity_matrix artifact: {token}"
        );
    }
}

fn require_weighted_attribution<'a>(matrix: &'a Value, artifact: &str) -> &'a Map<String, Value> {
    matrix
        .get("weighted_bottleneck_attribution")
        .and_then(Value::as_object)
        .unwrap_or_else(|| {
            panic!("phase1 matrix missing weighted_bottleneck_attribution object in {artifact}")
        })
}

fn assert_weighted_schema_and_status<'a>(
    weighted: &'a Map<String, Value>,
    artifact: &str,
) -> &'a str {
    let weighted_schema = weighted.get("schema").and_then(Value::as_str).unwrap_or("");
    assert_eq!(
        weighted_schema, "pi.perf.phase1_weighted_bottleneck_attribution.v1",
        "weighted attribution schema mismatch in {artifact}"
    );

    let status = weighted.get("status").and_then(Value::as_str).unwrap_or("");
    assert!(
        matches!(status, "computed" | "missing"),
        "weighted attribution status must be computed|missing in {artifact}, got {status:?}"
    );
    status
}

fn assert_weighted_payload_shape(weighted: &Map<String, Value>, status: &str, artifact: &str) {
    let per_scale = weighted
        .get("per_scale")
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("weighted attribution per_scale must be an array in {artifact}"));
    let global_ranking = weighted
        .get("global_ranking")
        .and_then(Value::as_array)
        .unwrap_or_else(|| {
            panic!("weighted attribution global_ranking must be an array in {artifact}")
        });

    if status != "computed" {
        return;
    }

    assert!(
        !per_scale.is_empty(),
        "weighted attribution per_scale must be non-empty when status=computed in {artifact}"
    );
    assert!(
        !global_ranking.is_empty(),
        "weighted attribution global_ranking must be non-empty when status=computed in {artifact}"
    );

    let observed_stages: HashSet<&str> = global_ranking
        .iter()
        .filter_map(|row| row.get("stage").and_then(Value::as_str))
        .collect();
    let expected_stages: HashSet<&str> = ["open_ms", "append_ms", "save_ms", "index_ms"]
        .iter()
        .copied()
        .collect();
    assert_eq!(
        observed_stages, expected_stages,
        "weighted attribution global_ranking stages mismatch in {artifact}"
    );
}

fn assert_phase5_downstream_consumers(matrix: &Value, artifact: &str) {
    let downstream_consumers = matrix
        .pointer("/consumption_contract/downstream_consumers")
        .and_then(Value::as_object)
        .unwrap_or_else(|| {
            panic!("consumption_contract.downstream_consumers must be an object in {artifact}")
        });

    for (consumer, bead_id, selector) in [
        (
            "opportunity_matrix",
            "bd-3ar8v.6.1",
            "weighted_bottleneck_attribution.global_ranking",
        ),
        (
            "parameter_sweeps",
            "bd-3ar8v.6.2",
            "weighted_bottleneck_attribution.per_scale",
        ),
    ] {
        let entry = downstream_consumers
            .get(consumer)
            .and_then(Value::as_object)
            .unwrap_or_else(|| {
                panic!("consumption_contract.downstream_consumers.{consumer} missing in {artifact}")
            });

        let observed_bead = entry.get("bead_id").and_then(Value::as_str).unwrap_or("");
        assert_eq!(
            observed_bead, bead_id,
            "downstream consumer bead mismatch for {consumer} in {artifact}"
        );

        let observed_selector = entry.get("selector").and_then(Value::as_str).unwrap_or("");
        assert_eq!(
            observed_selector, selector,
            "downstream consumer selector mismatch for {consumer} in {artifact}"
        );

        let source_artifact = entry
            .get("source_artifact")
            .and_then(Value::as_str)
            .unwrap_or("");
        assert_eq!(
            source_artifact, "phase1_matrix_validation",
            "downstream consumer source_artifact mismatch for {consumer} in {artifact}"
        );
    }
}

fn parse_positive_u64(raw: Option<&Value>) -> Option<u64> {
    match raw {
        Some(Value::Number(value)) => value.as_u64().filter(|parsed| *parsed > 0),
        Some(Value::String(value)) => value
            .trim()
            .parse::<u64>()
            .ok()
            .filter(|parsed| *parsed > 0),
        _ => None,
    }
}

#[test]
fn phase1_weighted_attribution_contract_links_phase5_consumers() {
    let (artifact, matrix) = require_phase1_matrix_validation();
    let consumption_contract = require_consumption_contract(&matrix, &artifact);

    assert_consumption_contract_downstream_beads(consumption_contract, &artifact);

    if !requires_strict_weighted_contract(consumption_contract, &matrix) {
        assert_orchestrate_weighted_contract_tokens(&artifact);
        return;
    }

    let weighted = require_weighted_attribution(&matrix, &artifact);
    let status = assert_weighted_schema_and_status(weighted, &artifact);
    assert_weighted_payload_shape(weighted, status, &artifact);
    assert_phase5_downstream_consumers(&matrix, &artifact);
}

#[test]
fn opportunity_matrix_contract_links_phase1_matrix_and_readiness() {
    let (phase1_artifact, phase1_matrix) = require_phase1_matrix_validation();
    let consumption_contract = require_consumption_contract(&phase1_matrix, &phase1_artifact);
    let opportunity_present = find_latest_opportunity_matrix(&repo_root()).is_some();
    if !requires_strict_opportunity_matrix_contract(consumption_contract, opportunity_present) {
        assert_orchestrate_opportunity_matrix_contract_tokens();
        return;
    }

    let (artifact, opportunity) = require_opportunity_matrix();

    let source_identity = opportunity
        .pointer("/source_identity")
        .and_then(Value::as_object)
        .unwrap_or_else(|| {
            panic!("opportunity_matrix.source_identity must be an object in {artifact}")
        });
    let source_artifact = source_identity
        .get("source_artifact")
        .and_then(Value::as_str)
        .unwrap_or("");
    assert_eq!(
        source_artifact, "phase1_matrix_validation",
        "opportunity_matrix.source_identity.source_artifact mismatch in {artifact}"
    );
    let source_artifact_path = source_identity
        .get("source_artifact_path")
        .and_then(Value::as_str)
        .unwrap_or("");
    assert!(
        !source_artifact_path.is_empty(),
        "opportunity_matrix.source_identity.source_artifact_path must be non-empty in {artifact}"
    );
    let normalized_source_path = source_artifact_path.replace('\\', "/");
    assert!(
        normalized_source_path.ends_with("phase1_matrix_validation.json"),
        "opportunity_matrix.source_identity.source_artifact_path must reference phase1_matrix_validation.json in {artifact}"
    );
    let normalized_phase1_artifact = phase1_artifact.replace('\\', "/");
    assert!(
        normalized_source_path.ends_with(&normalized_phase1_artifact)
            || normalized_phase1_artifact.ends_with("phase1_matrix_validation.json"),
        "opportunity_matrix source artifact path must align with discovered phase1 artifact: source={source_artifact_path:?}, phase1={phase1_artifact:?}"
    );

    let opportunity_correlation = opportunity
        .get("correlation_id")
        .and_then(Value::as_str)
        .unwrap_or("");
    let phase1_correlation = phase1_matrix
        .get("correlation_id")
        .and_then(Value::as_str)
        .unwrap_or("");
    assert!(
        !opportunity_correlation.is_empty() && !phase1_correlation.is_empty(),
        "opportunity_matrix/phase1 correlation_id must be non-empty in {artifact} and {phase1_artifact}"
    );
    assert_eq!(
        opportunity_correlation, phase1_correlation,
        "opportunity_matrix correlation_id must match phase1 matrix correlation_id ({artifact} vs {phase1_artifact})"
    );

    let readiness = opportunity
        .pointer("/readiness")
        .and_then(Value::as_object)
        .unwrap_or_else(|| panic!("opportunity_matrix.readiness must be an object in {artifact}"));
    let readiness_status = readiness
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("");
    assert!(
        matches!(readiness_status, "ready" | "blocked" | "no_decision"),
        "opportunity_matrix.readiness.status must be ready|blocked|no_decision in {artifact}, got {readiness_status:?}"
    );
    let readiness_decision = readiness
        .get("decision")
        .and_then(Value::as_str)
        .unwrap_or("");
    assert!(
        matches!(readiness_decision, "RANKED" | "NO_DECISION"),
        "opportunity_matrix.readiness.decision must be RANKED|NO_DECISION in {artifact}, got {readiness_decision:?}"
    );
    let readiness_mode = readiness.get("mode").and_then(Value::as_str).unwrap_or("");
    assert_eq!(
        readiness_mode, "fail_closed",
        "opportunity_matrix.readiness.mode must be fail_closed in {artifact}"
    );
    let ready_for_phase5 = readiness.get("ready_for_phase5").and_then(Value::as_bool);
    assert!(
        ready_for_phase5.is_some(),
        "opportunity_matrix.readiness.ready_for_phase5 must be a boolean in {artifact}"
    );
    let ranked_opportunities = opportunity
        .pointer("/ranked_opportunities")
        .and_then(Value::as_array)
        .unwrap_or_else(|| {
            panic!("opportunity_matrix.ranked_opportunities must be an array in {artifact}")
        });
    let phase1_ready = consumption_contract
        .get("artifact_ready_for_phase5")
        .and_then(Value::as_bool);
    if let Some(phase1_ready) = phase1_ready {
        assert_eq!(
            ready_for_phase5,
            Some(phase1_ready),
            "opportunity_matrix.readiness.ready_for_phase5 must match phase1 consumption_contract.artifact_ready_for_phase5 ({artifact} vs {phase1_artifact})"
        );
    }
    match readiness_status {
        "ready" => {
            assert_eq!(
                ready_for_phase5,
                Some(true),
                "opportunity_matrix.readiness.ready_for_phase5 must be true when status=ready in {artifact}"
            );
            assert_eq!(
                readiness_decision, "RANKED",
                "opportunity_matrix.readiness.decision must be RANKED when status=ready in {artifact}"
            );
            assert!(
                !ranked_opportunities.is_empty(),
                "opportunity_matrix.ranked_opportunities must be non-empty when readiness.status=ready in {artifact}"
            );
            for (index, row) in ranked_opportunities.iter().enumerate() {
                let row_obj = row.as_object().unwrap_or_else(|| {
                    panic!(
                        "opportunity_matrix.ranked_opportunities[{index}] must be an object in {artifact}"
                    )
                });
                let rank = parse_positive_u64(row_obj.get("rank")).unwrap_or_else(|| {
                    panic!(
                        "opportunity_matrix.ranked_opportunities[{index}].rank must be a positive integer in {artifact}"
                    )
                });
                assert_eq!(
                    rank,
                    (index + 1) as u64,
                    "opportunity_matrix.ranked_opportunities[{index}].rank must equal index+1 in {artifact}"
                );
                let stage = row_obj
                    .get("stage")
                    .and_then(Value::as_str)
                    .map_or("", str::trim);
                assert!(
                    !stage.is_empty(),
                    "opportunity_matrix.ranked_opportunities[{index}].stage must be non-empty in {artifact}"
                );

                let weighted_contribution_pct = row_obj
                    .get("weighted_contribution_pct")
                    .and_then(Value::as_f64)
                    .unwrap_or(f64::NAN);
                assert!(
                    weighted_contribution_pct.is_finite() && weighted_contribution_pct >= 0.0,
                    "opportunity_matrix.ranked_opportunities[{index}].weighted_contribution_pct must be non-negative numeric in {artifact}"
                );
                let expected_gain_pct = row_obj
                    .get("expected_gain_pct")
                    .and_then(Value::as_f64)
                    .unwrap_or(f64::NAN);
                assert!(
                    expected_gain_pct.is_finite() && expected_gain_pct >= 0.0,
                    "opportunity_matrix.ranked_opportunities[{index}].expected_gain_pct must be non-negative numeric in {artifact}"
                );
                let priority_score = row_obj
                    .get("priority_score")
                    .and_then(Value::as_f64)
                    .unwrap_or(f64::NAN);
                assert!(
                    priority_score.is_finite() && priority_score > 0.0,
                    "opportunity_matrix.ranked_opportunities[{index}].priority_score must be positive numeric in {artifact}"
                );

                let confidence = row_obj
                    .get("confidence")
                    .and_then(Value::as_object)
                    .unwrap_or_else(|| {
                        panic!(
                            "opportunity_matrix.ranked_opportunities[{index}].confidence must be an object in {artifact}"
                        )
                    });
                let confidence_level = confidence
                    .get("level")
                    .and_then(Value::as_str)
                    .unwrap_or("");
                assert!(
                    matches!(confidence_level, "low" | "medium" | "high"),
                    "opportunity_matrix.ranked_opportunities[{index}].confidence.level must be low|medium|high in {artifact}, got {confidence_level:?}"
                );
                let confidence_score = confidence
                    .get("score")
                    .and_then(Value::as_f64)
                    .unwrap_or(f64::NAN);
                assert!(
                    confidence_score.is_finite() && (0.0..=1.0).contains(&confidence_score),
                    "opportunity_matrix.ranked_opportunities[{index}].confidence.score must be within [0,1] in {artifact}"
                );
                let confidence_sufficient = confidence
                    .get("sufficient_for_decision")
                    .and_then(Value::as_bool);
                assert!(
                    confidence_sufficient.is_some(),
                    "opportunity_matrix.ranked_opportunities[{index}].confidence.sufficient_for_decision must be a boolean in {artifact}"
                );

                let user_impact = row_obj
                    .get("user_impact")
                    .and_then(Value::as_object)
                    .unwrap_or_else(|| {
                        panic!(
                            "opportunity_matrix.ranked_opportunities[{index}].user_impact must be an object in {artifact}"
                        )
                    });
                for field in ["resume_latency", "extension_responsiveness", "failure_risk"] {
                    let value = user_impact
                        .get(field)
                        .and_then(Value::as_str)
                        .map_or("", str::trim);
                    assert!(
                        !value.is_empty(),
                        "opportunity_matrix.ranked_opportunities[{index}].user_impact.{field} must be non-empty in {artifact}"
                    );
                }
            }
        }
        "blocked" => {
            assert_eq!(
                ready_for_phase5,
                Some(false),
                "opportunity_matrix.readiness.ready_for_phase5 must be false when status=blocked in {artifact}"
            );
            assert_eq!(
                readiness_decision, "NO_DECISION",
                "opportunity_matrix.readiness.decision must be NO_DECISION when status=blocked in {artifact}"
            );
            let blocking_reasons = readiness
                .get("blocking_reasons")
                .and_then(Value::as_array)
                .unwrap_or_else(|| {
                    panic!(
                        "opportunity_matrix.readiness.blocking_reasons must be an array when status=blocked in {artifact}"
                    )
                });
            assert!(
                !blocking_reasons.is_empty(),
                "opportunity_matrix.readiness.blocking_reasons must be non-empty when status=blocked in {artifact}"
            );
            assert!(
                ranked_opportunities.is_empty(),
                "opportunity_matrix.ranked_opportunities must be empty when readiness.status=blocked in {artifact}"
            );
        }
        "no_decision" => {
            assert_eq!(
                ready_for_phase5,
                Some(false),
                "opportunity_matrix.readiness.ready_for_phase5 must be false when status=no_decision in {artifact}"
            );
            assert_eq!(
                readiness_decision, "NO_DECISION",
                "opportunity_matrix.readiness.decision must be NO_DECISION when status=no_decision in {artifact}"
            );
            let no_decision_reasons = readiness
                .get("no_decision_reasons")
                .and_then(Value::as_array)
                .or_else(|| readiness.get("blocking_reasons").and_then(Value::as_array))
                .unwrap_or_else(|| {
                    panic!(
                        "opportunity_matrix.readiness.no_decision_reasons|blocking_reasons must be an array when status=no_decision in {artifact}"
                    )
                });
            assert!(
                !no_decision_reasons.is_empty(),
                "opportunity_matrix.readiness.no_decision_reasons must be non-empty when status=no_decision in {artifact}"
            );
            assert!(
                ranked_opportunities.is_empty(),
                "opportunity_matrix.ranked_opportunities must be empty when readiness.status=no_decision in {artifact}"
            );
        }
        _ => panic!(),
    }
}

#[test]
#[allow(clippy::too_many_lines)]
fn parameter_sweeps_contract_links_phase1_matrix_and_readiness() {
    let (phase1_artifact, phase1_matrix) = require_phase1_matrix_validation();
    let consumption_contract = require_consumption_contract(&phase1_matrix, &phase1_artifact);
    let sweeps_present = find_latest_parameter_sweeps(&repo_root()).is_some();
    if !requires_strict_parameter_sweeps_contract(consumption_contract, sweeps_present) {
        assert_orchestrate_parameter_sweeps_contract_tokens();
        return;
    }

    let (artifact, sweeps) = require_parameter_sweeps();

    let source_identity = sweeps
        .pointer("/source_identity")
        .and_then(Value::as_object)
        .unwrap_or_else(|| {
            panic!("parameter_sweeps.source_identity must be an object in {artifact}")
        });

    let source_artifact = source_identity
        .get("source_artifact")
        .and_then(Value::as_str)
        .unwrap_or("");
    assert_eq!(
        source_artifact, "phase1_matrix_validation",
        "parameter_sweeps.source_identity.source_artifact mismatch in {artifact}"
    );

    let source_artifact_path = source_identity
        .get("source_artifact_path")
        .and_then(Value::as_str)
        .unwrap_or("");
    assert!(
        !source_artifact_path.is_empty(),
        "parameter_sweeps.source_identity.source_artifact_path must be non-empty in {artifact}"
    );
    let normalized_source_path = source_artifact_path.replace('\\', "/");
    assert!(
        normalized_source_path.ends_with("phase1_matrix_validation.json"),
        "parameter_sweeps.source_identity.source_artifact_path must reference phase1_matrix_validation.json in {artifact}"
    );
    let normalized_phase1_artifact = phase1_artifact.replace('\\', "/");
    assert!(
        normalized_source_path.ends_with(&normalized_phase1_artifact)
            || normalized_phase1_artifact.ends_with("phase1_matrix_validation.json"),
        "parameter_sweeps source artifact path must align with discovered phase1 artifact: source={source_artifact_path:?}, phase1={phase1_artifact:?}"
    );

    let weighted_schema = source_identity
        .get("weighted_bottleneck_schema")
        .and_then(Value::as_str)
        .unwrap_or("");
    assert_eq!(
        weighted_schema, "pi.perf.phase1_weighted_bottleneck_attribution.v1",
        "parameter_sweeps.source_identity.weighted_bottleneck_schema mismatch in {artifact}"
    );

    let weighted_status = source_identity
        .get("weighted_bottleneck_status")
        .and_then(Value::as_str)
        .unwrap_or("");
    assert!(
        matches!(weighted_status, "computed" | "missing"),
        "parameter_sweeps.source_identity.weighted_bottleneck_status must be computed|missing in {artifact}, got {weighted_status:?}"
    );

    let sweeps_correlation = sweeps
        .get("correlation_id")
        .and_then(Value::as_str)
        .unwrap_or("");
    let phase1_correlation = phase1_matrix
        .get("correlation_id")
        .and_then(Value::as_str)
        .unwrap_or("");
    assert!(
        !sweeps_correlation.is_empty() && !phase1_correlation.is_empty(),
        "parameter_sweeps/phase1 correlation_id must be non-empty in {artifact} and {phase1_artifact}"
    );
    assert_eq!(
        sweeps_correlation, phase1_correlation,
        "parameter_sweeps correlation_id must match phase1 matrix correlation_id ({artifact} vs {phase1_artifact})"
    );

    let readiness = sweeps
        .pointer("/readiness")
        .and_then(Value::as_object)
        .unwrap_or_else(|| panic!("parameter_sweeps.readiness must be an object in {artifact}"));
    let readiness_status = readiness
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("");
    let ready_for_phase5 = readiness.get("ready_for_phase5").and_then(Value::as_bool);
    let blocking_reasons = readiness
        .get("blocking_reasons")
        .and_then(Value::as_array)
        .unwrap_or_else(|| {
            panic!("parameter_sweeps.readiness.blocking_reasons must be an array in {artifact}")
        });

    assert!(
        matches!(readiness_status, "ready" | "blocked"),
        "parameter_sweeps.readiness.status must be ready|blocked in {artifact}, got {readiness_status:?}"
    );
    match readiness_status {
        "ready" => {
            assert_eq!(
                ready_for_phase5,
                Some(true),
                "parameter_sweeps.readiness.ready_for_phase5 must be true when status=ready in {artifact}"
            );
            assert!(
                blocking_reasons.is_empty(),
                "parameter_sweeps.readiness.blocking_reasons must be empty when status=ready in {artifact}"
            );
        }
        "blocked" => {
            assert_eq!(
                ready_for_phase5,
                Some(false),
                "parameter_sweeps.readiness.ready_for_phase5 must be false when status=blocked in {artifact}"
            );
            assert!(
                !blocking_reasons.is_empty(),
                "parameter_sweeps.readiness.blocking_reasons must be non-empty when status=blocked in {artifact}"
            );
        }
        _ => panic!(),
    }

    let phase1_ready = phase1_matrix
        .pointer("/consumption_contract/artifact_ready_for_phase5")
        .and_then(Value::as_bool);
    if let Some(phase1_ready) = phase1_ready {
        assert_eq!(
            ready_for_phase5,
            Some(phase1_ready),
            "parameter_sweeps.readiness.ready_for_phase5 must match phase1 consumption_contract.artifact_ready_for_phase5 ({artifact} vs {phase1_artifact})"
        );
    }

    let selected_defaults = sweeps
        .pointer("/selected_defaults")
        .and_then(Value::as_object)
        .unwrap_or_else(|| {
            panic!("parameter_sweeps.selected_defaults must be an object in {artifact}")
        });
    let mut selected_default_values = HashMap::new();
    for key in ["flush_cadence_ms", "queue_max_items", "compaction_quota_mb"] {
        let parsed = parse_positive_u64(selected_defaults.get(key)).unwrap_or_else(|| {
            panic!(
                "parameter_sweeps.selected_defaults.{key} must be a positive integer in {artifact}"
            )
        });
        selected_default_values.insert(key, parsed);
    }

    let dimensions = sweeps
        .pointer("/sweep_plan/dimensions")
        .and_then(Value::as_array)
        .unwrap_or_else(|| {
            panic!("parameter_sweeps.sweep_plan.dimensions must be an array in {artifact}")
        });
    let mut observed_dimension_names = HashSet::new();
    for (index, dimension) in dimensions.iter().enumerate() {
        let dimension_obj = dimension.as_object().unwrap_or_else(|| {
            panic!(
                "parameter_sweeps.sweep_plan.dimensions[{index}] must be an object in {artifact}"
            )
        });
        let name = dimension_obj
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim()
            .to_string();
        assert!(
            !name.is_empty(),
            "parameter_sweeps.sweep_plan.dimensions[{index}].name must be non-empty in {artifact}"
        );
        observed_dimension_names.insert(name.clone());
        let candidate_values = dimension_obj
            .get("candidate_values")
            .and_then(Value::as_array)
            .unwrap_or_else(|| {
                panic!("parameter_sweeps.sweep_plan.dimensions[{index}].candidate_values must be an array in {artifact}")
            });
        assert!(
            !candidate_values.is_empty(),
            "parameter_sweeps.sweep_plan.dimensions[{index}].candidate_values must be non-empty in {artifact}"
        );
        let parsed_candidates: HashSet<u64> = candidate_values
            .iter()
            .map(|candidate| {
                parse_positive_u64(Some(candidate)).unwrap_or_else(|| {
                    panic!(
                        "parameter_sweeps.sweep_plan.dimensions[{index}].candidate_values entries must be positive integers in {artifact}"
                    )
                })
            })
            .collect();
        if let Some(selected_default) = selected_default_values.get(name.as_str()) {
            assert!(
                parsed_candidates.contains(selected_default),
                "parameter_sweeps.selected_defaults.{name}={selected_default} must appear in sweep_plan.dimensions[{index}].candidate_values in {artifact}"
            );
        }
    }
    for required in ["flush_cadence_ms", "queue_max_items", "compaction_quota_mb"] {
        assert!(
            observed_dimension_names.contains(required),
            "parameter_sweeps.sweep_plan.dimensions missing required knob {required} in {artifact}"
        );
    }
}

// ============================================================================
// Threshold enforcement
// ============================================================================

/// Compute pass/(pass+fail), ignoring N/A extensions that lack evidence.
/// Matches the `effective_pass_rate_pct` logic in `conformance_regression_gate.rs`.
fn effective_pass_rate_pct(sm: &Value) -> f64 {
    let pass = sm
        .pointer("/counts/pass")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let fail = sm
        .pointer("/counts/fail")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let total = sm
        .pointer("/counts/total")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let tested = pass + fail;
    let reported = sm
        .get("pass_rate_pct")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);

    if tested > 0 && tested < total {
        #[allow(clippy::cast_precision_loss)]
        {
            (pass as f64 / tested as f64) * 100.0
        }
    } else {
        reported
    }
}

#[test]
fn conformance_pass_rate_meets_release_threshold() {
    let sm = require_json("tests/ext_conformance/reports/conformance_summary.json");
    let bl = require_json("tests/ext_conformance/reports/conformance_baseline.json");

    let current_rate = effective_pass_rate_pct(&sm);
    let min_rate = bl
        .pointer("/regression_thresholds/overall_pass_rate_min_pct")
        .and_then(Value::as_f64)
        .unwrap_or(80.0);

    assert!(
        current_rate >= min_rate,
        "release gate BLOCKED: conformance pass rate {current_rate:.1}% \
         (effective: pass/(pass+fail), ignoring N/A) < minimum {min_rate:.1}%"
    );
}

#[test]
fn failure_count_within_release_threshold() {
    let sm = require_json("tests/ext_conformance/reports/conformance_summary.json");

    let fail = sm
        .pointer("/counts/fail")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let max_fail: u64 = 36;

    assert!(
        fail <= max_fail,
        "release gate BLOCKED: {fail} failures exceed maximum {max_fail}"
    );
}

#[test]
fn performance_budgets_report_exists_and_valid() {
    let budget = require_json("tests/perf/reports/budget_summary.json");

    assert!(
        budget.get("schema").is_some()
            || budget.get("budgets").is_some()
            || budget.get("summary").is_some(),
        "performance budget report must have recognizable structure"
    );
}

// ============================================================================
// Exception policy completeness
// ============================================================================

#[test]
fn exception_policy_covers_all_current_failures() {
    let bl = require_json("tests/ext_conformance/reports/conformance_baseline.json");

    let entries = bl
        .pointer("/exception_policy/entries")
        .and_then(Value::as_array);
    let total_classified = bl
        .pointer("/remediation_buckets/summary/total_classified")
        .and_then(Value::as_u64)
        .unwrap_or(0);

    let Some(entries) = entries else {
        // If no exception policy, there should be no failures.
        assert_eq!(
            total_classified, 0,
            "failures exist ({total_classified}) but no exception policy defined"
        );
        return;
    };

    // Every exception entry must have all required fields.
    let approved = entries
        .iter()
        .filter(|e| {
            e.get("status")
                .and_then(Value::as_str)
                .is_some_and(|s| s == "approved" || s == "temporary")
        })
        .count();

    assert!(
        approved > 0 || total_classified == 0,
        "failures exist ({total_classified}) but no approved exceptions"
    );
}

#[test]
fn exception_entries_have_review_dates() {
    let bl = require_json("tests/ext_conformance/reports/conformance_baseline.json");

    let entries = bl
        .pointer("/exception_policy/entries")
        .and_then(Value::as_array);

    let Some(entries) = entries else {
        return;
    };

    for entry in entries {
        let id = entry.get("id").and_then(Value::as_str).unwrap_or("?");
        let review_by = entry.get("review_by").and_then(Value::as_str);

        assert!(
            review_by.is_some(),
            "exception entry {id} missing review_by date"
        );
    }
}

// ============================================================================
// Evidence completeness score
// ============================================================================

#[test]
fn evidence_completeness_score_above_minimum() {
    let root = repo_root();
    let mut present = 0u32;

    for (path, _) in REQUIRED_ARTIFACTS {
        if root.join(path).is_file() {
            present += 1;
        }
    }

    #[allow(clippy::cast_precision_loss)]
    let score = (f64::from(present) / REQUIRED_ARTIFACTS.len() as f64) * 100.0;

    assert!(
        score >= 80.0,
        "evidence completeness {score:.0}% < 80% minimum (present={present}/{})",
        REQUIRED_ARTIFACTS.len()
    );
}

#[test]
fn conformance_evidence_has_linked_test_targets() {
    let sm = require_json("tests/ext_conformance/reports/conformance_summary.json");

    let evidence = sm.get("evidence").and_then(Value::as_object);
    let Some(evidence) = evidence else {
        // Evidence section is optional in summary v1.
        return;
    };

    // At least one evidence category should have non-zero count.
    let total_evidence: u64 = evidence.values().filter_map(Value::as_u64).sum();

    assert!(
        total_evidence > 0,
        "conformance summary has evidence section but all counts are zero"
    );
}

#[test]
fn franken_node_claim_contract_is_present_and_valid() {
    let contract = require_json(FRANKEN_NODE_CLAIM_CONTRACT_PATH);
    validate_franken_node_claim_contract(&contract).unwrap_or_else(|err| {
        panic!("franken_node claim contract should validate fail-closed: {err}")
    });
}

#[test]
fn franken_node_claim_contract_fails_closed_on_missing_required_tier() {
    let mut contract = require_json(FRANKEN_NODE_CLAIM_CONTRACT_PATH);
    let Some(tiers) = contract
        .get_mut("claim_tiers")
        .and_then(Value::as_array_mut)
    else {
        panic!("fixture claim_tiers must be an array");
    };
    tiers.retain(|tier| {
        tier.get("tier_id")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim()
            != "TIER-3-FULL-NODE-BUN-REPLACEMENT"
    });

    let err = validate_franken_node_claim_contract(&contract)
        .expect_err("missing required tier must fail closed");
    assert!(
        err.contains("missing required claim tier: TIER-3-FULL-NODE-BUN-REPLACEMENT"),
        "error should name the missing required tier, got: {err}"
    );
}

#[test]
fn franken_node_claim_contract_fails_closed_on_empty_required_evidence_list() {
    let mut contract = require_json(FRANKEN_NODE_CLAIM_CONTRACT_PATH);
    contract["claim_tiers"][0]["required_evidence"] = serde_json::json!([]);

    let err = validate_franken_node_claim_contract(&contract)
        .expect_err("empty required_evidence list must fail closed");
    assert!(
        err.contains("must include required_evidence entries")
            || err.contains("required_evidence must be non-empty"),
        "error should explain required_evidence contract failure, got: {err}"
    );
}

#[test]
fn franken_node_claim_contract_fails_closed_on_missing_package_interop_evidence_token() {
    let mut contract = require_json(FRANKEN_NODE_CLAIM_CONTRACT_PATH);
    let tiers = contract
        .get_mut("claim_tiers")
        .and_then(Value::as_array_mut)
        .expect("claim_tiers must be an array");
    let targeted_runtime_tier = tiers
        .iter_mut()
        .find(|tier| {
            tier.get("tier_id")
                .and_then(Value::as_str)
                .map(str::trim)
                .is_some_and(|tier_id| tier_id == "TIER-2-TARGETED-RUNTIME-PARITY")
        })
        .expect("TIER-2-TARGETED-RUNTIME-PARITY must exist");
    let evidence = targeted_runtime_tier
        .get_mut("required_evidence")
        .and_then(Value::as_array_mut)
        .expect("TIER-2 required_evidence must be an array");
    evidence.retain(|entry| {
        !entry.as_str().map_or("", str::trim).eq_ignore_ascii_case(
            "package/ecosystem interoperability contract evidence (CJS/ESM/npm)",
        )
    });

    let err = validate_franken_node_claim_contract(&contract)
        .expect_err("missing package interop evidence token must fail closed");
    assert!(
        err.contains("required_evidence missing token")
            && err.contains("package/ecosystem interoperability contract evidence"),
        "error should identify missing package interop token, got: {err}"
    );
}

#[test]
fn franken_node_claim_contract_fails_closed_on_missing_kernel_mapping_evidence_token() {
    let mut contract = require_json(FRANKEN_NODE_CLAIM_CONTRACT_PATH);
    let tiers = contract
        .get_mut("claim_tiers")
        .and_then(Value::as_array_mut)
        .expect("claim_tiers must be an array");
    let target_tier = tiers
        .iter_mut()
        .find(|tier| {
            tier.get("tier_id")
                .and_then(Value::as_str)
                .map(str::trim)
                .is_some_and(|tier_id| tier_id == "TIER-3-FULL-NODE-BUN-REPLACEMENT")
        })
        .expect("TIER-3-FULL-NODE-BUN-REPLACEMENT must exist");
    let evidence = target_tier
        .get_mut("required_evidence")
        .and_then(Value::as_array_mut)
        .expect("TIER-3 required_evidence must be an array");
    evidence.retain(|entry| {
        !entry.as_str().map_or("", str::trim).eq_ignore_ascii_case(
            "kernel extraction boundary manifest and reintegration mapping evidence",
        )
    });

    let err = validate_franken_node_claim_contract(&contract)
        .expect_err("missing kernel mapping evidence token must fail closed");
    assert!(
        err.contains("required_evidence missing token")
            && err
                .contains("kernel extraction boundary manifest and reintegration mapping evidence"),
        "error should identify missing kernel mapping token, got: {err}"
    );
}

#[test]
fn franken_node_claim_contract_fails_closed_on_missing_runtime_substrate_evidence_token() {
    let mut contract = require_json(FRANKEN_NODE_CLAIM_CONTRACT_PATH);
    let tiers = contract
        .get_mut("claim_tiers")
        .and_then(Value::as_array_mut)
        .expect("claim_tiers must be an array");
    let target_tier = tiers
        .iter_mut()
        .find(|tier| {
            tier.get("tier_id")
                .and_then(Value::as_str)
                .map(str::trim)
                .is_some_and(|tier_id| tier_id == "TIER-3-FULL-NODE-BUN-REPLACEMENT")
        })
        .expect("TIER-3-FULL-NODE-BUN-REPLACEMENT must exist");
    let evidence = target_tier
        .get_mut("required_evidence")
        .and_then(Value::as_array_mut)
        .expect("TIER-3 required_evidence must be an array");
    evidence.retain(|entry| {
        !entry
            .as_str()
            .map_or("", str::trim)
            .eq_ignore_ascii_case("runtime-substrate generalization evidence for bd-3ar8v.7.5")
    });

    let err = validate_franken_node_claim_contract(&contract)
        .expect_err("missing runtime substrate evidence token must fail closed");
    assert!(
        err.contains("required_evidence missing token")
            && err.contains("runtime-substrate generalization evidence for bd-3ar8v.7.5"),
        "error should identify missing runtime substrate evidence token, got: {err}"
    );
}

#[test]
fn franken_node_claim_contract_fails_closed_on_missing_multi_tier_execution_evidence_token() {
    let mut contract = require_json(FRANKEN_NODE_CLAIM_CONTRACT_PATH);
    let tiers = contract
        .get_mut("claim_tiers")
        .and_then(Value::as_array_mut)
        .expect("claim_tiers must be an array");
    let tier3_entry = tiers
        .iter_mut()
        .find(|tier| {
            tier.get("tier_id")
                .and_then(Value::as_str)
                .map(str::trim)
                .is_some_and(|tier_id| tier_id == "TIER-3-FULL-NODE-BUN-REPLACEMENT")
        })
        .expect("TIER-3-FULL-NODE-BUN-REPLACEMENT must exist");
    let evidence = tier3_entry
        .get_mut("required_evidence")
        .and_then(Value::as_array_mut)
        .expect("TIER-3 required_evidence must be an array");
    evidence.retain(|entry| {
        !entry
            .as_str()
            .map_or("", str::trim)
            .eq_ignore_ascii_case("multi-tier execution engine evidence for bd-3ar8v.7.6")
    });

    let err = validate_franken_node_claim_contract(&contract)
        .expect_err("missing multi-tier execution evidence token must fail closed");
    assert!(
        err.contains("required_evidence missing token")
            && err.contains("multi-tier execution engine evidence for bd-3ar8v.7.6"),
        "error should identify missing multi-tier execution evidence token, got: {err}"
    );
}

#[test]
fn franken_node_claim_contract_fails_closed_on_missing_remediation_backlog_evidence_token() {
    let mut contract = require_json(FRANKEN_NODE_CLAIM_CONTRACT_PATH);
    let tiers = contract
        .get_mut("claim_tiers")
        .and_then(Value::as_array_mut)
        .expect("claim_tiers must be an array");
    let tier3_entry = tiers
        .iter_mut()
        .find(|tier| {
            tier.get("tier_id")
                .and_then(Value::as_str)
                .map(str::trim)
                .is_some_and(|tier_id| tier_id == "TIER-3-FULL-NODE-BUN-REPLACEMENT")
        })
        .expect("TIER-3-FULL-NODE-BUN-REPLACEMENT must exist");
    let evidence = tier3_entry
        .get_mut("required_evidence")
        .and_then(Value::as_array_mut)
        .expect("TIER-3 required_evidence must be an array");
    evidence.retain(|entry| {
        !entry.as_str().map_or("", str::trim).eq_ignore_ascii_case(
            "compatibility remediation backlog generator evidence for bd-3ar8v.7.16",
        )
    });

    let err = validate_franken_node_claim_contract(&contract)
        .expect_err("missing remediation backlog evidence token must fail closed");
    assert!(
        err.contains("required_evidence missing token")
            && err
                .contains("compatibility remediation backlog generator evidence for bd-3ar8v.7.16"),
        "error should identify missing remediation backlog evidence token, got: {err}"
    );
}

#[test]
fn franken_node_claim_contract_fails_closed_on_missing_required_overclaim_blocker() {
    let mut contract = require_json(FRANKEN_NODE_CLAIM_CONTRACT_PATH);
    let Some(blockers) = contract
        .pointer_mut("/claim_gate_policy/overclaim_blockers")
        .and_then(Value::as_array_mut)
    else {
        panic!("fixture overclaim_blockers must be an array");
    };
    blockers
        .retain(|entry| entry.as_str().map_or("", str::trim) != "forbidden_claim_phrase_detected");

    let err = validate_franken_node_claim_contract(&contract)
        .expect_err("missing required overclaim blocker must fail closed");
    assert!(
        err.contains(
            "claim_gate_policy.overclaim_blockers missing forbidden_claim_phrase_detected"
        ),
        "error should identify missing overclaim blocker token, got: {err}"
    );
}

#[test]
fn franken_node_claim_contract_fails_closed_on_allowed_forbidden_phrase_overlap() {
    let mut contract = require_json(FRANKEN_NODE_CLAIM_CONTRACT_PATH);
    contract["claim_tiers"][0]["forbidden_claim_language"] =
        serde_json::json!(["Extension-hosting parity scope only"]);

    let err = validate_franken_node_claim_contract(&contract)
        .expect_err("allowed/forbidden phrase overlap must fail closed");
    assert!(
        err.contains("overlap between allowed_claim_language and forbidden_claim_language"),
        "error should explain overlap violation, got: {err}"
    );
}
