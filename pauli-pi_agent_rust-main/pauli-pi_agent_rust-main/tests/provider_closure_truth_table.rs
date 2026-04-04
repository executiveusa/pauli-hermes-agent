//! Validation tests for docs/provider-closure-truth-table.json
//!
//! Ensures the closure truth table is structurally sound, internally consistent,
//! enumerates all 5 focus providers with complete evidence chains, and
//! cross-references discrepancy IDs correctly.
//!
//! Bead: bd-3uqg.12.2.2

mod common;

use common::TestHarness;
use serde_json::Value;
use std::collections::HashSet;
use std::path::Path;

fn load_truth_table() -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/provider-closure-truth-table.json");
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read truth table file: {e}"));
    serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("Failed to parse truth table JSON: {e}"))
}

// ─── Section 1: Schema and file existence ───────────────────────────────────

#[test]
fn truth_table_file_exists_and_parses() {
    let harness = TestHarness::new("truth_table_file_exists_and_parses");
    harness
        .log()
        .info("schema", "Verifying truth table file exists");
    let doc = load_truth_table();
    assert_eq!(
        doc["schema"].as_str().unwrap(),
        "pi.qa.provider_closure_truth_table.v1"
    );
    harness.log().info("schema", "Truth table file valid");
}

#[test]
fn truth_table_has_required_top_level_fields() {
    let doc = load_truth_table();
    let required = [
        "schema",
        "bead_id",
        "generated_at_utc",
        "purpose",
        "methodology",
        "providers",
        "cross_provider_matrix",
        "aggregate_summary",
        "acceptance_criteria",
    ];
    for field in &required {
        assert!(
            doc.get(field).is_some(),
            "Missing required top-level field: {field}"
        );
    }
}

#[test]
fn truth_table_bead_id_matches() {
    let doc = load_truth_table();
    assert_eq!(doc["bead_id"].as_str().unwrap(), "bd-3uqg.12.2.2");
}

// ─── Section 2: All 5 focus providers present ───────────────────────────────

#[test]
fn all_five_focus_providers_present() {
    let doc = load_truth_table();
    let providers = doc["providers"].as_object().unwrap();
    let expected: HashSet<&str> = ["groq", "cerebras", "openrouter", "kimi", "qwen"]
        .iter()
        .copied()
        .collect();
    let actual: HashSet<&str> = providers.keys().map(String::as_str).collect();
    assert_eq!(
        expected, actual,
        "Provider set mismatch. Expected: {expected:?}, got: {actual:?}"
    );
}

#[test]
fn no_extra_providers_beyond_five() {
    let doc = load_truth_table();
    let providers = doc["providers"].as_object().unwrap();
    assert_eq!(
        providers.len(),
        5,
        "Expected exactly 5 providers, got {}",
        providers.len()
    );
}

// ─── Section 3: Each provider has required fields ───────────────────────────

#[test]
fn each_provider_has_display_name() {
    let doc = load_truth_table();
    let providers = doc["providers"].as_object().unwrap();
    for (key, provider) in providers {
        assert!(
            provider.get("display_name").is_some(),
            "Provider {key} missing display_name"
        );
    }
}

#[test]
fn each_provider_has_disposition() {
    let doc = load_truth_table();
    let providers = doc["providers"].as_object().unwrap();
    for (key, provider) in providers {
        assert!(
            provider.get("disposition").is_some(),
            "Provider {key} missing disposition"
        );
        let disposition = &provider["disposition"];
        assert!(
            disposition.get("status").is_some(),
            "Provider {key} disposition missing status"
        );
        assert!(
            disposition.get("label").is_some(),
            "Provider {key} disposition missing label"
        );
        assert!(
            disposition.get("summary").is_some(),
            "Provider {key} disposition missing summary"
        );
    }
}

#[test]
fn each_provider_has_test_evidence() {
    let doc = load_truth_table();
    let providers = doc["providers"].as_object().unwrap();
    for (key, provider) in providers {
        assert!(
            provider.get("test_evidence").is_some(),
            "Provider {key} missing test_evidence"
        );
        let evidence = &provider["test_evidence"];
        assert!(
            evidence.get("vcr_cassettes").is_some(),
            "Provider {key} test_evidence missing vcr_cassettes"
        );
        assert!(
            evidence.get("unit_tests").is_some(),
            "Provider {key} test_evidence missing unit_tests"
        );
        assert!(
            evidence.get("conformance_tests").is_some(),
            "Provider {key} test_evidence missing conformance_tests"
        );
        assert!(
            evidence.get("e2e_tests").is_some(),
            "Provider {key} test_evidence missing e2e_tests"
        );
    }
}

#[test]
fn each_provider_has_docs_section() {
    let doc = load_truth_table();
    let providers = doc["providers"].as_object().unwrap();
    for (key, provider) in providers {
        assert!(
            provider.get("docs").is_some(),
            "Provider {key} missing docs"
        );
        let docs = &provider["docs"];
        assert!(
            docs.get("setup_guide").is_some(),
            "Provider {key} docs missing setup_guide"
        );
        assert!(
            docs.get("capability_profile").is_some(),
            "Provider {key} docs missing capability_profile"
        );
        assert!(
            docs.get("status").is_some(),
            "Provider {key} docs missing status"
        );
    }
}

#[test]
fn each_provider_has_known_caveats() {
    let doc = load_truth_table();
    let providers = doc["providers"].as_object().unwrap();
    for (key, provider) in providers {
        assert!(
            provider.get("known_caveats").is_some(),
            "Provider {key} missing known_caveats"
        );
        let caveats = provider["known_caveats"].as_array().unwrap();
        assert!(
            !caveats.is_empty(),
            "Provider {key} has empty known_caveats (every focus provider has at least one)"
        );
    }
}

#[test]
fn each_provider_has_related_beads() {
    let doc = load_truth_table();
    let providers = doc["providers"].as_object().unwrap();
    for (key, provider) in providers {
        assert!(
            provider.get("related_beads").is_some(),
            "Provider {key} missing related_beads"
        );
        let beads = &provider["related_beads"];
        assert!(
            beads.get("onboarding").is_some(),
            "Provider {key} related_beads missing onboarding"
        );
        assert!(
            beads.get("audit_verification").is_some(),
            "Provider {key} related_beads missing audit_verification"
        );
    }
}

// ─── Section 4: VCR cassette counts are accurate ────────────────────────────

#[test]
fn simple_providers_have_seven_cassettes() {
    let doc = load_truth_table();
    for provider_key in &["groq", "cerebras", "openrouter"] {
        let cassettes = &doc["providers"][provider_key]["test_evidence"]["vcr_cassettes"];
        let count = cassettes["count"].as_u64().unwrap();
        assert_eq!(
            count, 7,
            "Provider {provider_key} should have 7 VCR cassettes, got {count}"
        );
        let scenarios = cassettes["scenarios"].as_array().unwrap();
        assert_eq!(
            scenarios.len(),
            7,
            "Provider {provider_key} should enumerate 7 scenarios"
        );
    }
}

#[test]
fn kimi_has_fifteen_total_cassettes() {
    let doc = load_truth_table();
    let cassettes = &doc["providers"]["kimi"]["test_evidence"]["vcr_cassettes"];
    let total = cassettes["total_count"].as_u64().unwrap();
    assert_eq!(total, 15, "Kimi should have 15 total VCR cassettes");

    let by_entry = cassettes["by_entry"].as_object().unwrap();
    assert_eq!(
        by_entry["moonshotai"]["count"].as_u64().unwrap(),
        7,
        "moonshotai should have 7 cassettes"
    );
    assert_eq!(
        by_entry["moonshotai-cn"]["count"].as_u64().unwrap(),
        4,
        "moonshotai-cn should have 4 cassettes"
    );
    assert_eq!(
        by_entry["kimi-for-coding"]["count"].as_u64().unwrap(),
        4,
        "kimi-for-coding should have 4 cassettes"
    );
}

#[test]
fn qwen_has_eleven_total_cassettes() {
    let doc = load_truth_table();
    let cassettes = &doc["providers"]["qwen"]["test_evidence"]["vcr_cassettes"];
    let total = cassettes["total_count"].as_u64().unwrap();
    assert_eq!(total, 11, "Qwen should have 11 total VCR cassettes");

    let by_entry = cassettes["by_entry"].as_object().unwrap();
    assert_eq!(
        by_entry["alibaba"]["count"].as_u64().unwrap(),
        7,
        "alibaba should have 7 cassettes"
    );
    assert_eq!(
        by_entry["alibaba-cn"]["count"].as_u64().unwrap(),
        4,
        "alibaba-cn should have 4 cassettes"
    );
}

// ─── Section 5: Aggregate summary consistency ───────────────────────────────

#[test]
fn aggregate_total_providers_matches() {
    let doc = load_truth_table();
    let summary = &doc["aggregate_summary"];
    assert_eq!(
        summary["total_providers_audited"].as_u64().unwrap(),
        5,
        "Should audit exactly 5 providers"
    );
}

#[test]
fn aggregate_total_pi_entries_matches() {
    let doc = load_truth_table();
    let summary = &doc["aggregate_summary"];
    // groq(1) + cerebras(1) + openrouter(1) + kimi(3) + qwen(2) = 8
    assert_eq!(
        summary["total_pi_entries"].as_u64().unwrap(),
        8,
        "Should have 8 total Pi provider entries"
    );
}

#[test]
fn aggregate_total_vcr_cassettes_matches() {
    let doc = load_truth_table();
    let summary = &doc["aggregate_summary"];
    // groq(7) + cerebras(7) + openrouter(7) + kimi(15) + qwen(11) = 47
    assert_eq!(
        summary["total_vcr_cassettes"].as_u64().unwrap(),
        47,
        "Should have 47 total VCR cassettes"
    );
}

#[test]
fn aggregate_all_dispositions_closed() {
    let doc = load_truth_table();
    assert!(
        doc["aggregate_summary"]["all_dispositions_closed"]
            .as_bool()
            .unwrap()
    );
    assert!(
        doc["aggregate_summary"]["all_production_ready"]
            .as_bool()
            .unwrap()
    );
    assert_eq!(
        doc["aggregate_summary"]["blocking_issues"]
            .as_u64()
            .unwrap(),
        0
    );
}

// ─── Section 6: Cross-provider matrix structure ─────────────────────────────

#[test]
fn cross_provider_matrix_has_correct_headers() {
    let doc = load_truth_table();
    let matrix = &doc["cross_provider_matrix"];
    let headers = matrix["headers"].as_array().unwrap();
    assert_eq!(headers.len(), 6, "Matrix should have 6 columns");
    assert_eq!(headers[0].as_str().unwrap(), "capability");
    assert_eq!(headers[1].as_str().unwrap(), "groq");
    assert_eq!(headers[2].as_str().unwrap(), "cerebras");
    assert_eq!(headers[3].as_str().unwrap(), "openrouter");
    assert_eq!(headers[4].as_str().unwrap(), "kimi");
    assert_eq!(headers[5].as_str().unwrap(), "qwen");
}

#[test]
fn cross_provider_matrix_rows_match_header_width() {
    let doc = load_truth_table();
    let matrix = &doc["cross_provider_matrix"];
    let headers = matrix["headers"].as_array().unwrap();
    let rows = matrix["rows"].as_array().unwrap();
    assert!(!rows.is_empty(), "Matrix should have at least one row");
    for (i, row) in rows.iter().enumerate() {
        let row_arr = row.as_array().unwrap();
        assert_eq!(
            row_arr.len(),
            headers.len(),
            "Row {i} width ({}) != header width ({})",
            row_arr.len(),
            headers.len()
        );
    }
}

#[test]
fn cross_provider_matrix_has_production_ready_row() {
    let doc = load_truth_table();
    let rows = doc["cross_provider_matrix"]["rows"].as_array().unwrap();
    let has_production_ready = rows
        .iter()
        .any(|row| row[0].as_str() == Some("production_ready"));
    assert!(
        has_production_ready,
        "Matrix should have a production_ready row"
    );
}

// ─── Section 7: Acceptance criteria all met ─────────────────────────────────

#[test]
fn all_acceptance_criteria_met() {
    let doc = load_truth_table();
    let criteria = doc["acceptance_criteria"].as_object().unwrap();
    assert!(
        !criteria.is_empty(),
        "Should have at least one acceptance criterion"
    );
    for (key, criterion) in criteria {
        assert!(
            criterion["met"].as_bool().unwrap(),
            "Acceptance criterion {key} is not met"
        );
        assert!(
            criterion.get("evidence").is_some(),
            "Acceptance criterion {key} missing evidence"
        );
    }
}

// ─── Section 8: Disposition values are valid ────────────────────────────────

#[test]
fn all_dispositions_are_valid_values() {
    let doc = load_truth_table();
    let valid_statuses: HashSet<&str> = ["CLOSED", "OPEN"].iter().copied().collect();
    let valid_labels: HashSet<&str> = [
        "PRODUCTION_READY",
        "BLOCKED",
        "PARTIALLY_COMPLETE",
        "NOT_STARTED",
    ]
    .iter()
    .copied()
    .collect();

    let providers = doc["providers"].as_object().unwrap();
    for (key, provider) in providers {
        let status = provider["disposition"]["status"].as_str().unwrap();
        assert!(
            valid_statuses.contains(status),
            "Provider {key} has invalid disposition status: {status}"
        );
        let label = provider["disposition"]["label"].as_str().unwrap();
        assert!(
            valid_labels.contains(label),
            "Provider {key} has invalid disposition label: {label}"
        );
    }
}

// ─── Section 9: VCR cassette files actually exist ───────────────────────────

#[test]
fn vcr_cassette_files_exist_for_simple_providers() {
    let fixtures_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/vcr");
    let doc = load_truth_table();

    for provider_key in &["groq", "cerebras", "openrouter"] {
        let scenarios =
            doc["providers"][provider_key]["test_evidence"]["vcr_cassettes"]["scenarios"]
                .as_array()
                .unwrap();
        for scenario in scenarios {
            let name = scenario.as_str().unwrap();
            let cassette_path = fixtures_dir.join(format!("{name}.json"));
            assert!(
                cassette_path.exists(),
                "Missing VCR cassette: {name}.json for provider {provider_key}"
            );
        }
    }
}

#[test]
fn vcr_cassette_files_exist_for_kimi_variants() {
    let fixtures_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/vcr");
    let doc = load_truth_table();

    let by_entry = doc["providers"]["kimi"]["test_evidence"]["vcr_cassettes"]["by_entry"]
        .as_object()
        .unwrap();
    for (entry_name, entry_data) in by_entry {
        let scenarios = entry_data["scenarios"].as_array().unwrap();
        for scenario in scenarios {
            let name = scenario.as_str().unwrap();
            let cassette_path = fixtures_dir.join(format!("{name}.json"));
            assert!(
                cassette_path.exists(),
                "Missing VCR cassette: {name}.json for kimi entry {entry_name}"
            );
        }
    }
}

#[test]
fn vcr_cassette_files_exist_for_qwen_variants() {
    let fixtures_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/vcr");
    let doc = load_truth_table();

    let by_entry = doc["providers"]["qwen"]["test_evidence"]["vcr_cassettes"]["by_entry"]
        .as_object()
        .unwrap();
    for (entry_name, entry_data) in by_entry {
        let scenarios = entry_data["scenarios"].as_array().unwrap();
        for scenario in scenarios {
            let name = scenario.as_str().unwrap();
            let cassette_path = fixtures_dir.join(format!("{name}.json"));
            assert!(
                cassette_path.exists(),
                "Missing VCR cassette: {name}.json for qwen entry {entry_name}"
            );
        }
    }
}

// ─── Section 10: Doc files referenced actually exist ────────────────────────

#[test]
fn referenced_doc_files_exist() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let doc = load_truth_table();
    let providers = doc["providers"].as_object().unwrap();

    for (key, provider) in providers {
        let docs = &provider["docs"];

        let setup = docs["setup_guide"].as_str().unwrap();
        assert!(
            root.join(setup).exists(),
            "Provider {key}: setup_guide not found: {setup}"
        );

        let profile = docs["capability_profile"].as_str().unwrap();
        assert!(
            root.join(profile).exists(),
            "Provider {key}: capability_profile not found: {profile}"
        );
    }
}

// ─── Section 11: Methodology inputs are documented ──────────────────────────

#[test]
fn methodology_has_inputs_and_dimensions() {
    let doc = load_truth_table();
    let methodology = &doc["methodology"];
    let inputs = methodology["inputs"].as_array().unwrap();
    assert!(
        inputs.len() >= 10,
        "Methodology should reference at least 10 input sources"
    );
    let dimensions = methodology["dimensions"].as_array().unwrap();
    assert!(
        dimensions.len() >= 5,
        "Methodology should document at least 5 analysis dimensions"
    );
}

// ─── Section 12: Discrepancy cross-references are valid DISC-NNN format ─────

#[test]
fn discrepancy_refs_use_valid_format() {
    let doc = load_truth_table();
    let refs = doc["aggregate_summary"]["discrepancy_refs"]
        .as_array()
        .unwrap();
    for disc_ref in refs {
        let s = disc_ref.as_str().unwrap();
        assert!(
            s.starts_with("DISC-"),
            "Discrepancy ref should start with DISC-: {s}"
        );
    }
}

#[test]
fn aggregate_discrepancy_count_matches_refs() {
    let doc = load_truth_table();
    let summary = &doc["aggregate_summary"];
    let total = summary["open_discrepancies_total"].as_u64().unwrap();
    let refs = summary["discrepancy_refs"].as_array().unwrap();
    assert_eq!(
        total,
        refs.len() as u64,
        "open_discrepancies_total ({total}) should match discrepancy_refs count ({})",
        refs.len()
    );
}

// ─── Section 13: Kimi has exactly 3 variants, Qwen has exactly 2 ───────────

#[test]
fn kimi_has_three_variants() {
    let doc = load_truth_table();
    let variants = doc["providers"]["kimi"]["variants"].as_object().unwrap();
    assert_eq!(
        variants.len(),
        3,
        "Kimi should have exactly 3 variants (moonshotai, moonshotai-cn, kimi-for-coding)"
    );
    assert!(variants.contains_key("moonshotai"));
    assert!(variants.contains_key("moonshotai-cn"));
    assert!(variants.contains_key("kimi-for-coding"));
}

#[test]
fn qwen_has_two_variants() {
    let doc = load_truth_table();
    let variants = doc["providers"]["qwen"]["variants"].as_object().unwrap();
    assert_eq!(
        variants.len(),
        2,
        "Qwen should have exactly 2 variants (alibaba, alibaba-cn)"
    );
    assert!(variants.contains_key("alibaba"));
    assert!(variants.contains_key("alibaba-cn"));
}

// ─── Section 14: kimi-for-coding uses Anthropic API ─────────────────────────

#[test]
fn kimi_for_coding_uses_anthropic_api() {
    let doc = load_truth_table();
    let kfc = &doc["providers"]["kimi"]["variants"]["kimi-for-coding"];
    assert_eq!(
        kfc["api_family"].as_str().unwrap(),
        "anthropic-messages",
        "kimi-for-coding should use anthropic-messages API"
    );
    assert_eq!(
        kfc["route_kind"].as_str().unwrap(),
        "ApiAnthropicMessages",
        "kimi-for-coding should route through ApiAnthropicMessages"
    );
}

// ─── Section 15: Comprehensive report ───────────────────────────────────────

#[test]
fn comprehensive_truth_table_report() {
    let harness = TestHarness::new("comprehensive_truth_table_report");
    let doc = load_truth_table();

    // Verify provider count
    let providers = doc["providers"].as_object().unwrap();
    harness
        .log()
        .info("count", format!("Providers audited: {}", providers.len()));
    assert_eq!(providers.len(), 5);

    // Verify all are CLOSED
    for (key, provider) in providers {
        let status = provider["disposition"]["status"].as_str().unwrap();
        harness
            .log()
            .info("disposition", format!("{key}: {status}"));
        assert_eq!(status, "CLOSED", "Provider {key} should be CLOSED");
    }

    // Verify total cassette count
    let summary = &doc["aggregate_summary"];
    let total_cassettes = summary["total_vcr_cassettes"].as_u64().unwrap();
    harness.log().info(
        "cassettes",
        format!("Total VCR cassettes: {total_cassettes}"),
    );
    assert_eq!(total_cassettes, 47);

    // Verify no blockers
    let blocking = summary["blocking_issues"].as_u64().unwrap();
    harness
        .log()
        .info("blockers", format!("Blocking issues: {blocking}"));
    assert_eq!(blocking, 0);

    harness
        .log()
        .info("verdict", "All 5 focus providers CLOSED/PRODUCTION_READY");
}
