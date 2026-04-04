//! Validation tests for docs/provider-discrepancy-classification.json
//!
//! Ensures the discrepancy ledger is structurally sound, internally consistent,
//! and aligned with the bead-specified root-cause taxonomy.
//!
//! Bead: bd-3uqg.12.1.1

mod common;

use common::TestHarness;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::path::Path;

fn load_classification() -> Value {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/provider-discrepancy-classification.json");
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read classification file: {e}"));
    serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("Failed to parse classification JSON: {e}"))
}

// ─── Section 1: Schema and file existence ───────────────────────────────────

#[test]
fn classification_file_exists_and_parses() {
    let harness = TestHarness::new("classification_file_exists_and_parses");
    harness
        .log()
        .info("schema", "Verifying classification file exists");
    let doc = load_classification();
    assert_eq!(
        doc["schema"].as_str().unwrap(),
        "pi.qa.provider_discrepancy_classification.v1"
    );
    harness.log().info("schema", "Classification file valid");
}

#[test]
fn classification_has_required_top_level_fields() {
    let doc = load_classification();
    let required = [
        "schema",
        "bead_id",
        "generated_at_utc",
        "description",
        "methodology",
        "summary",
        "discrepancies",
        "cross_reference_matrix",
        "remediation_priority",
        "acceptance_criteria_compliance",
    ];
    for field in &required {
        assert!(
            doc.get(field).is_some(),
            "Missing required top-level field: {field}"
        );
    }
}

// ─── Section 2: Discrepancy entry validation ────────────────────────────────

#[test]
fn every_discrepancy_has_required_fields() {
    let harness = TestHarness::new("every_discrepancy_has_required_fields");
    let doc = load_classification();
    let entries = doc["discrepancies"].as_array().unwrap();
    let required_fields = [
        "id",
        "provider_scope",
        "discrepancy_type",
        "title",
        "severity",
        "user_impact",
        "evidence_refs",
        "proposed_action",
    ];
    for entry in entries {
        let id = entry["id"].as_str().unwrap_or("UNKNOWN");
        for field in &required_fields {
            assert!(
                entry.get(field).is_some(),
                "Discrepancy {id} missing required field: {field}"
            );
        }
        harness
            .log()
            .info("validate", format!("{id}: all required fields present"));
    }
}

#[test]
fn no_duplicate_discrepancy_ids() {
    let doc = load_classification();
    let entries = doc["discrepancies"].as_array().unwrap();
    let mut seen = HashSet::new();
    for entry in entries {
        let id = entry["id"].as_str().unwrap();
        assert!(
            seen.insert(id.to_string()),
            "Duplicate discrepancy ID: {id}"
        );
    }
}

#[test]
fn discrepancy_ids_are_sequential() {
    let doc = load_classification();
    let entries = doc["discrepancies"].as_array().unwrap();
    for (i, entry) in entries.iter().enumerate() {
        let expected = format!("DISC-{:03}", i + 1);
        let actual = entry["id"].as_str().unwrap();
        assert_eq!(
            actual, expected,
            "Expected {expected} at index {i}, got {actual}"
        );
    }
}

// ─── Section 3: Taxonomy compliance ─────────────────────────────────────────

#[test]
fn all_discrepancy_types_are_from_taxonomy() {
    let doc = load_classification();
    let rules = doc["methodology"]["classification_rules"]
        .as_object()
        .unwrap();
    let allowed_types: HashSet<&str> = rules.keys().map(String::as_str).collect();

    let entries = doc["discrepancies"].as_array().unwrap();
    for entry in entries {
        let id = entry["id"].as_str().unwrap();
        let dtype = entry["discrepancy_type"].as_str().unwrap();
        assert!(
            allowed_types.contains(dtype),
            "Discrepancy {id} has type '{dtype}' not in taxonomy: {allowed_types:?}"
        );
    }
}

#[test]
fn taxonomy_covers_bead_required_classes() {
    let doc = load_classification();
    let rules = doc["methodology"]["classification_rules"]
        .as_object()
        .unwrap();
    let types: HashSet<&str> = rules.keys().map(String::as_str).collect();

    // Bead bd-3uqg.12.1.1 requires these root-cause classes
    let required_classes = [
        "implementation_gap",
        "alias_mismatch",
        "test_evidence_gap",
        "docs_mismatch",
        "logging_schema_drift",
    ];
    for class in &required_classes {
        assert!(
            types.contains(class),
            "Taxonomy missing bead-required class: {class}"
        );
    }
}

#[test]
fn all_severity_values_are_from_grading() {
    let doc = load_classification();
    let grading = doc["methodology"]["severity_grading"].as_object().unwrap();
    let allowed_severities: HashSet<&str> = grading.keys().map(String::as_str).collect();

    let entries = doc["discrepancies"].as_array().unwrap();
    for entry in entries {
        let id = entry["id"].as_str().unwrap();
        let severity = entry["severity"].as_str().unwrap();
        assert!(
            allowed_severities.contains(severity),
            "Discrepancy {id} has severity '{severity}' not in grading: {allowed_severities:?}"
        );
    }
}

// ─── Section 4: Summary consistency ─────────────────────────────────────────

#[test]
fn summary_total_matches_discrepancy_count() {
    let doc = load_classification();
    let total = doc["summary"]["total_discrepancies"].as_u64().unwrap();
    let entries = doc["discrepancies"].as_array().unwrap();
    assert_eq!(
        usize::try_from(total).unwrap(),
        entries.len(),
        "Summary total ({total}) does not match actual discrepancy count ({})",
        entries.len()
    );
}

#[test]
fn summary_by_type_matches_actual_distribution() {
    let doc = load_classification();
    let by_type = doc["summary"]["by_type"].as_object().unwrap();
    let entries = doc["discrepancies"].as_array().unwrap();

    // Count actual types
    let mut actual_counts: HashMap<String, u64> = HashMap::new();
    for entry in entries {
        let dtype = entry["discrepancy_type"].as_str().unwrap().to_string();
        *actual_counts.entry(dtype).or_insert(0) += 1;
    }

    for (dtype, claimed_count) in by_type {
        let claimed = claimed_count.as_u64().unwrap();
        let actual = actual_counts.get(dtype.as_str()).copied().unwrap_or(0);
        assert_eq!(
            claimed, actual,
            "Summary by_type[{dtype}] claims {claimed} but actual is {actual}"
        );
    }

    // Check no types exist in actual but not in summary
    for (dtype, count) in &actual_counts {
        assert!(
            by_type.contains_key(dtype),
            "Type '{dtype}' with {count} entries not listed in summary.by_type"
        );
    }
}

#[test]
fn summary_by_severity_matches_actual_distribution() {
    let doc = load_classification();
    let by_severity = doc["summary"]["by_severity"].as_object().unwrap();
    let entries = doc["discrepancies"].as_array().unwrap();

    let mut actual_counts: HashMap<String, u64> = HashMap::new();
    for entry in entries {
        let severity = entry["severity"].as_str().unwrap().to_string();
        *actual_counts.entry(severity).or_insert(0) += 1;
    }

    for (severity, claimed_count) in by_severity {
        let claimed = claimed_count.as_u64().unwrap();
        let actual = actual_counts.get(severity.as_str()).copied().unwrap_or(0);
        assert_eq!(
            claimed, actual,
            "Summary by_severity[{severity}] claims {claimed} but actual is {actual}"
        );
    }
}

// ─── Section 5: Cross-reference matrix validation ───────────────────────────

#[test]
fn cross_reference_by_root_cause_ids_exist_in_discrepancies() {
    let doc = load_classification();
    let entries = doc["discrepancies"].as_array().unwrap();
    let all_ids: HashSet<&str> = entries.iter().map(|e| e["id"].as_str().unwrap()).collect();

    let by_root_cause = doc["cross_reference_matrix"]["by_root_cause"]
        .as_object()
        .unwrap();
    for (cause, data) in by_root_cause {
        let ids = data["ids"].as_array().unwrap();
        for id_val in ids {
            let id = id_val.as_str().unwrap();
            assert!(
                all_ids.contains(id),
                "Cross-ref by_root_cause[{cause}] references {id} which does not exist"
            );
        }
    }
}

#[test]
fn cross_reference_by_root_cause_counts_match() {
    let doc = load_classification();
    let by_root_cause = doc["cross_reference_matrix"]["by_root_cause"]
        .as_object()
        .unwrap();
    for (cause, data) in by_root_cause {
        let claimed_count = data["count"].as_u64().unwrap();
        let ids = data["ids"].as_array().unwrap();
        assert_eq!(
            usize::try_from(claimed_count).unwrap(),
            ids.len(),
            "Cross-ref by_root_cause[{cause}] count ({claimed_count}) != ids length ({})",
            ids.len()
        );
    }
}

#[test]
fn cross_reference_by_user_impact_ids_exist_in_discrepancies() {
    let doc = load_classification();
    let entries = doc["discrepancies"].as_array().unwrap();
    let all_ids: HashSet<&str> = entries.iter().map(|e| e["id"].as_str().unwrap()).collect();

    let by_impact = doc["cross_reference_matrix"]["by_user_impact"]
        .as_object()
        .unwrap();
    for (impact, data) in by_impact {
        let ids = data["ids"].as_array().unwrap();
        for id_val in ids {
            let id = id_val.as_str().unwrap();
            assert!(
                all_ids.contains(id),
                "Cross-ref by_user_impact[{impact}] references {id} which does not exist"
            );
        }
    }
}

#[test]
fn cross_reference_by_user_impact_covers_all_discrepancies() {
    let doc = load_classification();
    let entries = doc["discrepancies"].as_array().unwrap();
    let all_ids: HashSet<&str> = entries.iter().map(|e| e["id"].as_str().unwrap()).collect();

    let by_impact = doc["cross_reference_matrix"]["by_user_impact"]
        .as_object()
        .unwrap();
    let mut covered_ids: HashSet<String> = HashSet::new();
    for (_impact, data) in by_impact {
        let ids = data["ids"].as_array().unwrap();
        for id_val in ids {
            covered_ids.insert(id_val.as_str().unwrap().to_string());
        }
    }

    for id in &all_ids {
        assert!(
            covered_ids.contains(*id),
            "Discrepancy {id} not covered by any by_user_impact category"
        );
    }
}

#[test]
fn cross_reference_by_user_impact_counts_match() {
    let doc = load_classification();
    let by_impact = doc["cross_reference_matrix"]["by_user_impact"]
        .as_object()
        .unwrap();
    for (impact, data) in by_impact {
        let claimed_count = data["count"].as_u64().unwrap();
        let ids = data["ids"].as_array().unwrap();
        assert_eq!(
            usize::try_from(claimed_count).unwrap(),
            ids.len(),
            "Cross-ref by_user_impact[{impact}] count ({claimed_count}) != ids length ({})",
            ids.len()
        );
    }
}

// ─── Section 6: Evidence references validation ──────────────────────────────

#[test]
fn evidence_refs_are_non_empty_arrays() {
    let doc = load_classification();
    let entries = doc["discrepancies"].as_array().unwrap();
    for entry in entries {
        let id = entry["id"].as_str().unwrap();
        let refs = entry["evidence_refs"].as_array().unwrap();
        assert!(!refs.is_empty(), "Discrepancy {id} has empty evidence_refs");
        for r in refs {
            assert!(
                r.as_str().is_some() && !r.as_str().unwrap().is_empty(),
                "Discrepancy {id} has non-string or empty evidence_ref"
            );
        }
    }
}

#[test]
fn evidence_refs_point_to_known_files_or_beads() {
    let doc = load_classification();
    let entries = doc["discrepancies"].as_array().unwrap();
    let known_prefixes = ["docs/", "src/", "tests/", "bd-"];
    for entry in entries {
        let id = entry["id"].as_str().unwrap();
        let refs = entry["evidence_refs"].as_array().unwrap();
        for r in refs {
            let ref_str = r.as_str().unwrap();
            let has_known_prefix = known_prefixes.iter().any(|p| ref_str.starts_with(p));
            assert!(
                has_known_prefix,
                "Discrepancy {id} evidence_ref '{ref_str}' does not start with a known prefix ({known_prefixes:?})"
            );
        }
    }
}

// ─── Section 7: Remediation priority validation ─────────────────────────────

#[test]
fn remediation_ranks_are_sequential() {
    let doc = load_classification();
    let priorities = doc["remediation_priority"].as_array().unwrap();
    for (i, entry) in priorities.iter().enumerate() {
        let rank = entry["rank"].as_u64().unwrap();
        assert_eq!(
            usize::try_from(rank).unwrap(),
            i + 1,
            "Remediation priority rank at index {i} is {rank}, expected {}",
            i + 1
        );
    }
}

#[test]
fn remediation_entries_reference_existing_discrepancies() {
    let doc = load_classification();
    let entries = doc["discrepancies"].as_array().unwrap();
    let all_ids: HashSet<&str> = entries.iter().map(|e| e["id"].as_str().unwrap()).collect();

    let priorities = doc["remediation_priority"].as_array().unwrap();
    for entry in priorities {
        // Entry may have "id" (single) or "ids" (multiple)
        if let Some(id) = entry.get("id").and_then(Value::as_str) {
            assert!(
                all_ids.contains(id),
                "Remediation references non-existent discrepancy: {id}"
            );
        }
        if let Some(ids) = entry.get("ids").and_then(Value::as_array) {
            for id_val in ids {
                let id = id_val.as_str().unwrap();
                assert!(
                    all_ids.contains(id),
                    "Remediation references non-existent discrepancy: {id}"
                );
            }
        }
    }
}

#[test]
fn remediation_entries_have_required_fields() {
    let doc = load_classification();
    let priorities = doc["remediation_priority"].as_array().unwrap();
    for entry in priorities {
        let rank = entry["rank"].as_u64().unwrap();
        assert!(
            entry.get("action").is_some(),
            "Remediation rank {rank} missing 'action' field"
        );
        assert!(
            entry.get("effort").is_some(),
            "Remediation rank {rank} missing 'effort' field"
        );
        assert!(
            entry.get("impact").is_some(),
            "Remediation rank {rank} missing 'impact' field"
        );
        assert!(
            entry.get("rollout_risk").is_some(),
            "Remediation rank {rank} missing 'rollout_risk' field"
        );
    }
}

// ─── Section 8: Acceptance criteria compliance ──────────────────────────────

#[test]
fn acceptance_criteria_all_pass() {
    let doc = load_classification();
    let criteria = doc["acceptance_criteria_compliance"].as_object().unwrap();
    for (criterion, data) in criteria {
        let status = data["status"].as_str().unwrap();
        assert!(
            status.starts_with("PASS"),
            "Acceptance criterion '{criterion}' is not passing: {status}"
        );
    }
}

// ─── Section 9: Integration with artifact contract ──────────────────────────

#[test]
fn artifact_contract_gaps_are_covered_in_classification() {
    let contract_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/provider_e2e_artifact_contract.json");
    let contract: Value =
        serde_json::from_str(&std::fs::read_to_string(&contract_path).unwrap()).unwrap();

    let gaps = contract["gaps_identified"].as_array().unwrap();
    let classification = load_classification();
    let entries = classification["discrepancies"].as_array().unwrap();

    // Collect all evidence refs that mention the artifact contract
    let contract_refs: Vec<&str> = entries
        .iter()
        .flat_map(|e| {
            e["evidence_refs"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|r| r.as_str())
        })
        .filter(|r| r.contains("provider_e2e_artifact_contract.json"))
        .collect();

    // Each gap should be referenced
    for gap in gaps {
        let gap_id = gap["id"].as_str().unwrap();
        let is_covered = contract_refs.iter().any(|r| r.contains(gap_id));
        assert!(
            is_covered,
            "Artifact contract gap {gap_id} is not referenced in classification evidence_refs"
        );
    }
}

// ─── Section 10: Comprehensive report ───────────────────────────────────────

#[test]
fn comprehensive_discrepancy_classification_report() {
    let harness = TestHarness::new("comprehensive_discrepancy_classification_report");
    let doc = load_classification();

    let entries = doc["discrepancies"].as_array().unwrap();
    let total = entries.len();

    // Count by type
    let mut by_type: HashMap<String, usize> = HashMap::new();
    let mut by_severity: HashMap<String, usize> = HashMap::new();
    for entry in entries {
        let dtype = entry["discrepancy_type"].as_str().unwrap().to_string();
        let severity = entry["severity"].as_str().unwrap().to_string();
        *by_type.entry(dtype).or_insert(0) += 1;
        *by_severity.entry(severity).or_insert(0) += 1;
    }

    harness
        .log()
        .info("report", format!("Total discrepancies: {total}"));
    harness
        .log()
        .info("report", format!("By type: {by_type:?}"));
    harness
        .log()
        .info("report", format!("By severity: {by_severity:?}"));

    // Validate structure
    let checks = [
        ("has_schema", doc.get("schema").is_some()),
        ("has_methodology", doc.get("methodology").is_some()),
        ("has_discrepancies", !entries.is_empty()),
        ("has_cross_ref", doc.get("cross_reference_matrix").is_some()),
        ("has_remediation", doc.get("remediation_priority").is_some()),
        (
            "has_acceptance",
            doc.get("acceptance_criteria_compliance").is_some(),
        ),
        (
            "no_implementation_gaps",
            by_type.get("implementation_gap").copied().unwrap_or(0) == 0,
        ),
        (
            "taxonomy_complete",
            doc["methodology"]["classification_rules"]
                .as_object()
                .unwrap()
                .len()
                >= 5,
        ),
    ];

    let mut pass_count = 0;
    for (name, passed) in &checks {
        let status = if *passed { "PASS" } else { "FAIL" };
        harness.log().info("check", format!("{name}: {status}"));
        if *passed {
            pass_count += 1;
        }
    }

    harness.log().info(
        "summary",
        format!(
            "Classification report: {pass_count}/{} checks passed",
            checks.len()
        ),
    );

    assert_eq!(
        pass_count,
        checks.len(),
        "Not all checks passed: {pass_count}/{}",
        checks.len()
    );
}
