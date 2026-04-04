//! SEC-6.5 (bd-2jkio): Unit-Test Traceability Matrix Across Security Workstreams
//!
//! Validates that every SEC implementation bead maps to concrete test targets,
//! that all referenced test files exist, and that minimum test counts are met.
//! This is the machine-readable governance gate for security test coverage.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// One row in the SEC traceability matrix.
struct SecBeadEntry {
    bead_id: &'static str,
    sec_id: &'static str,
    title: &'static str,
    /// Primary test files that directly verify this bead's functionality.
    primary_test_files: &'static [&'static str],
    /// Supplementary test files with partial or indirect coverage.
    supplementary_test_files: &'static [&'static str],
    /// Minimum number of `#[test]` functions expected across primary files.
    min_primary_test_count: usize,
    /// Whether this bead requires deterministic/golden-fixture tests.
    requires_deterministic_fixtures: bool,
}

/// Complete SEC traceability matrix: every SEC implementation bead mapped to tests.
const SEC_MATRIX: &[SecBeadEntry] = &[
    // ── WS2: Supply-Chain Integrity ──
    SecBeadEntry {
        bead_id: "bd-f0huc",
        sec_id: "SEC-2.1",
        title: "Extension manifest v2: capabilities, intents, trust metadata",
        primary_test_files: &["extensions_manifest", "capability_policy_model"],
        supplementary_test_files: &["e2e_workflow_preflight"],
        min_primary_test_count: 50,
        requires_deterministic_fixtures: false,
    },
    SecBeadEntry {
        bead_id: "bd-3br2a",
        sec_id: "SEC-2.2",
        title: "Deterministic extension lockfile and provenance verification",
        primary_test_files: &["extension_lockfile_provenance"],
        supplementary_test_files: &["ext_provenance_verification", "e2e_workflow_preflight"],
        min_primary_test_count: 25,
        requires_deterministic_fixtures: true,
    },
    SecBeadEntry {
        bead_id: "bd-21vng",
        sec_id: "SEC-2.3",
        title: "Install-time static scanner with deterministic risk classifier",
        primary_test_files: &["install_time_security_scanner"],
        supplementary_test_files: &["ext_preflight_analyzer"],
        min_primary_test_count: 40,
        requires_deterministic_fixtures: true,
    },
    SecBeadEntry {
        bead_id: "bd-21nj4",
        sec_id: "SEC-2.4",
        title: "Quarantine-to-trust promotion workflow",
        primary_test_files: &["extension_trust_promotion"],
        supplementary_test_files: &[],
        min_primary_test_count: 30,
        requires_deterministic_fixtures: false,
    },
    // ── WS3: Runtime Anomaly Detection ──
    SecBeadEntry {
        bead_id: "bd-2a9ll",
        sec_id: "SEC-3.1",
        title: "Runtime hostcall telemetry schema and feature extraction",
        primary_test_files: &["baseline_modeling"],
        supplementary_test_files: &["e2e_runtime_risk_telemetry"],
        min_primary_test_count: 25,
        requires_deterministic_fixtures: false,
    },
    SecBeadEntry {
        bead_id: "bd-153pv",
        sec_id: "SEC-3.2",
        title: "Baseline modeling with robust statistics and Markov profiles",
        primary_test_files: &["baseline_modeling", "baseline_modeling_evidence"],
        supplementary_test_files: &["runtime_risk_quantile_validation"],
        min_primary_test_count: 30,
        requires_deterministic_fixtures: true,
    },
    SecBeadEntry {
        bead_id: "bd-3f1ab",
        sec_id: "SEC-3.3",
        title: "Online deterministic risk scorer with explainable reason codes",
        primary_test_files: &[
            "risk_scorer_golden_fixtures",
            "bayesian_explanation_evidence",
            "explanation_calibration_replay",
        ],
        supplementary_test_files: &["runtime_risk_quantile_evidence"],
        min_primary_test_count: 20,
        requires_deterministic_fixtures: true,
    },
    SecBeadEntry {
        bead_id: "bd-3tb30",
        sec_id: "SEC-3.4",
        title: "Enforcement state machine: allow/harden/prompt/deny/terminate",
        primary_test_files: &["enforcement_state_machine_sec34"],
        supplementary_test_files: &[],
        min_primary_test_count: 12,
        requires_deterministic_fixtures: true,
    },
    SecBeadEntry {
        bead_id: "bd-3i9da",
        sec_id: "SEC-3.5",
        title: "Hash-chained decision ledger and offline threshold calibration",
        primary_test_files: &["ledger_calibration_sec35"],
        supplementary_test_files: &[],
        min_primary_test_count: 12,
        requires_deterministic_fixtures: true,
    },
    // ── WS4: Runtime Policy Enforcement ──
    SecBeadEntry {
        bead_id: "bd-b1d7o",
        sec_id: "SEC-4.1",
        title: "Per-extension resource quotas (CPU, memory, hostcalls, subprocess)",
        primary_test_files: &["capability_policy_scoped", "security_budgets"],
        supplementary_test_files: &[],
        min_primary_test_count: 80,
        requires_deterministic_fixtures: false,
    },
    SecBeadEntry {
        bead_id: "bd-wzzp4",
        sec_id: "SEC-4.2",
        title: "Capability-scoped filesystem/network allowlists",
        primary_test_files: &[
            "capability_policy_scoped",
            "security_fs_escape",
            "security_http_policy",
            "capability_denial_matrix",
        ],
        supplementary_test_files: &[],
        min_primary_test_count: 100,
        requires_deterministic_fixtures: false,
    },
    SecBeadEntry {
        bead_id: "bd-zh0hj",
        sec_id: "SEC-4.3",
        title: "Exec and secret mediation hardening (command policy + secret broker)",
        primary_test_files: &["exec_mediation_integration"],
        supplementary_test_files: &["adversarial_extensions"],
        min_primary_test_count: 50,
        requires_deterministic_fixtures: false,
    },
    SecBeadEntry {
        bead_id: "bd-2vbax",
        sec_id: "SEC-4.4",
        title: "Policy profile hardening and dangerous-capability opt-in",
        primary_test_files: &["policy_profile_hardening", "capability_policy_model"],
        supplementary_test_files: &[],
        min_primary_test_count: 60,
        requires_deterministic_fixtures: false,
    },
    // ── WS5: Operator UX and Incident Response ──
    SecBeadEntry {
        bead_id: "bd-qudx1",
        sec_id: "SEC-5.1",
        title: "Security control center and real-time runtime alerts",
        primary_test_files: &["security_alert_integration"],
        supplementary_test_files: &[],
        min_primary_test_count: 25,
        requires_deterministic_fixtures: false,
    },
    SecBeadEntry {
        bead_id: "bd-ww5br",
        sec_id: "SEC-5.2",
        title: "Trust onboarding wizard and emergency kill-switch",
        primary_test_files: &["trust_onboarding_killswitch_sec52"],
        supplementary_test_files: &["extension_trust_promotion"],
        min_primary_test_count: 10,
        requires_deterministic_fixtures: false,
    },
    SecBeadEntry {
        bead_id: "bd-11mqo",
        sec_id: "SEC-5.3",
        title: "Incident evidence bundle export and forensic replay",
        primary_test_files: &["incident_evidence_bundle", "incident_evidence_bundle_sec53"],
        supplementary_test_files: &[],
        min_primary_test_count: 35,
        requires_deterministic_fixtures: true,
    },
    // ── WS7: Rollout and Operations ──
    SecBeadEntry {
        bead_id: "bd-8lppo",
        sec_id: "SEC-7.2",
        title: "Graduated enforcement rollout with rollback guards",
        primary_test_files: &["graduated_enforcement_rollout"],
        supplementary_test_files: &["enforcement_state_machine_sec34"],
        min_primary_test_count: 40,
        requires_deterministic_fixtures: false,
    },
];

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn count_tests_in_file(path: &Path) -> usize {
    let content = std::fs::read_to_string(path).unwrap_or_default();
    content.matches("#[test]").count()
}

// ── Matrix completeness ──

#[test]
fn every_sec_bead_has_at_least_one_primary_test_file() {
    let missing: Vec<_> = SEC_MATRIX
        .iter()
        .filter(|e| e.primary_test_files.is_empty())
        .map(|e| format!("{} ({})", e.sec_id, e.bead_id))
        .collect();

    assert!(
        missing.is_empty(),
        "SEC beads without any primary test file mapping:\n{}",
        missing.join("\n")
    );
}

#[test]
fn no_duplicate_bead_ids_in_matrix() {
    let mut seen = BTreeSet::new();
    let duplicates: Vec<_> = SEC_MATRIX
        .iter()
        .filter(|e| !seen.insert(e.bead_id))
        .map(|e| format!("{} ({})", e.sec_id, e.bead_id))
        .collect();

    assert!(
        duplicates.is_empty(),
        "Duplicate bead IDs in SEC matrix:\n{}",
        duplicates.join("\n")
    );
}

#[test]
fn no_duplicate_sec_ids_in_matrix() {
    let mut seen = BTreeSet::new();
    let duplicates: Vec<_> = SEC_MATRIX
        .iter()
        .filter(|e| !seen.insert(e.sec_id))
        .map(|e| format!("{} ({})", e.sec_id, e.bead_id))
        .collect();

    assert!(
        duplicates.is_empty(),
        "Duplicate SEC IDs in matrix:\n{}",
        duplicates.join("\n")
    );
}

// ── File existence ──

#[test]
fn all_primary_test_files_exist_on_disk() {
    let root = repo_root();
    let mut missing = Vec::new();
    for entry in SEC_MATRIX {
        for f in entry.primary_test_files {
            let path = root.join(format!("tests/{f}.rs"));
            if !path.exists() {
                missing.push(format!(
                    "  {} ({}): tests/{f}.rs",
                    entry.sec_id, entry.bead_id
                ));
            }
        }
    }

    assert!(
        missing.is_empty(),
        "Primary test files referenced by matrix do not exist:\n{}",
        missing.join("\n")
    );
}

#[test]
fn all_supplementary_test_files_exist_on_disk() {
    let root = repo_root();
    let mut missing = Vec::new();
    for entry in SEC_MATRIX {
        for f in entry.supplementary_test_files {
            let path = root.join(format!("tests/{f}.rs"));
            if !path.exists() {
                missing.push(format!(
                    "  {} ({}): tests/{f}.rs",
                    entry.sec_id, entry.bead_id
                ));
            }
        }
    }

    assert!(
        missing.is_empty(),
        "Supplementary test files referenced by matrix do not exist:\n{}",
        missing.join("\n")
    );
}

// ── Test count thresholds ──

#[test]
fn primary_test_count_meets_minimum_threshold() {
    let root = repo_root();
    let mut failures = Vec::new();

    for entry in SEC_MATRIX {
        let total: usize = entry
            .primary_test_files
            .iter()
            .map(|f| count_tests_in_file(&root.join(format!("tests/{f}.rs"))))
            .sum();

        if total < entry.min_primary_test_count {
            failures.push(format!(
                "  {} ({}): found {total} tests, expected >= {} across {:?}",
                entry.sec_id, entry.bead_id, entry.min_primary_test_count, entry.primary_test_files
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "SEC beads below minimum primary test count:\n{}",
        failures.join("\n")
    );
}

// ── Deterministic fixture requirement ──

#[test]
fn deterministic_beads_have_fixture_or_golden_tests() {
    let root = repo_root();
    let fixture_keywords = [
        "golden",
        "fixture",
        "deterministic",
        "replay",
        "reproducible",
        "calibration",
        "hash_chain",
        "chain_intact",
    ];

    let mut missing = Vec::new();
    for entry in SEC_MATRIX {
        if !entry.requires_deterministic_fixtures {
            continue;
        }
        let mut has_fixture_test = false;
        for f in entry.primary_test_files {
            let path = root.join(format!("tests/{f}.rs"));
            let content = std::fs::read_to_string(&path).unwrap_or_default();
            let lower = content.to_lowercase();
            if fixture_keywords.iter().any(|kw| lower.contains(kw)) {
                has_fixture_test = true;
                break;
            }
        }
        if !has_fixture_test {
            missing.push(format!(
                "  {} ({}): requires deterministic fixtures but none found in {:?}",
                entry.sec_id, entry.bead_id, entry.primary_test_files
            ));
        }
    }

    assert!(
        missing.is_empty(),
        "SEC beads requiring deterministic fixtures without matching test content:\n{}",
        missing.join("\n")
    );
}

// ── Workstream coverage ──

#[test]
fn all_security_workstreams_represented() {
    let workstreams: BTreeSet<&str> = SEC_MATRIX
        .iter()
        .map(|e| {
            let parts: Vec<&str> = e.sec_id.split('-').collect();
            let num = parts[1].split('.').next().unwrap_or("0");
            match num {
                "2" => "WS2-SupplyChain",
                "3" => "WS3-AnomalyDetection",
                "4" => "WS4-PolicyEnforcement",
                "5" => "WS5-OperatorUX",
                "7" => "WS7-RolloutOps",
                _ => "Unknown",
            }
        })
        .collect();

    let expected = BTreeSet::from([
        "WS2-SupplyChain",
        "WS3-AnomalyDetection",
        "WS4-PolicyEnforcement",
        "WS5-OperatorUX",
        "WS7-RolloutOps",
    ]);

    let missing: Vec<_> = expected.difference(&workstreams).collect();
    assert!(
        missing.is_empty(),
        "Security workstreams without any mapped bead: {missing:?}"
    );
}

// ── Cross-reference with traceability_matrix.json ──

#[test]
fn sec_beads_present_in_traceability_matrix_json() {
    let root = repo_root();
    let path = root.join("docs/traceability_matrix.json");
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    let matrix: serde_json::Value =
        serde_json::from_str(&content).unwrap_or_else(|e| panic!("invalid JSON: {e}"));

    let json_ids: BTreeSet<String> = matrix
        .get("requirements")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|r| r.get("id").and_then(|v| v.as_str()).map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let missing: Vec<_> = SEC_MATRIX
        .iter()
        .filter(|e| !json_ids.contains(e.bead_id))
        .map(|e| format!("{} ({})", e.sec_id, e.bead_id))
        .collect();

    assert!(
        missing.is_empty(),
        "SEC beads in code matrix but missing from docs/traceability_matrix.json:\n{}",
        missing.join("\n")
    );
}

// ── Coverage summary report ──

#[test]
fn sec_traceability_coverage_report() {
    let root = repo_root();
    let mut total_beads = 0;
    let mut total_primary_tests = 0;
    let mut total_supplementary_tests = 0;
    let mut per_ws: BTreeMap<&str, (usize, usize)> = BTreeMap::new();

    eprintln!("\n=== SEC-6.5 Traceability Matrix Coverage Report ===\n");
    eprintln!(
        "{:<10} {:<8} {:<50} {:>6} {:>6} {:>6}",
        "SEC-ID", "Bead", "Title", "Prim", "Supp", "Total"
    );
    eprintln!("{}", "-".repeat(96));

    for entry in SEC_MATRIX {
        total_beads += 1;
        let primary: usize = entry
            .primary_test_files
            .iter()
            .map(|f| count_tests_in_file(&root.join(format!("tests/{f}.rs"))))
            .sum();
        let supplementary: usize = entry
            .supplementary_test_files
            .iter()
            .map(|f| count_tests_in_file(&root.join(format!("tests/{f}.rs"))))
            .sum();

        total_primary_tests += primary;
        total_supplementary_tests += supplementary;

        let ws = match entry.sec_id.chars().nth(4) {
            Some('2') => "WS2",
            Some('3') => "WS3",
            Some('4') => "WS4",
            Some('5') => "WS5",
            Some('7') => "WS7",
            _ => "WS?",
        };
        let ws_entry = per_ws.entry(ws).or_insert((0, 0));
        ws_entry.0 += 1;
        ws_entry.1 += primary + supplementary;

        let title_trunc = if entry.title.len() > 48 {
            format!("{}...", &entry.title[..45])
        } else {
            entry.title.to_string()
        };

        eprintln!(
            "{:<10} {:<8} {:<50} {:>6} {:>6} {:>6}",
            entry.sec_id,
            entry.bead_id,
            title_trunc,
            primary,
            supplementary,
            primary + supplementary
        );
    }

    eprintln!("{}", "-".repeat(96));
    eprintln!(
        "{:<10} {:<8} {:<50} {:>6} {:>6} {:>6}",
        "TOTAL",
        "",
        format!("{total_beads} beads"),
        total_primary_tests,
        total_supplementary_tests,
        total_primary_tests + total_supplementary_tests
    );
    eprintln!();
    eprintln!("Per-workstream breakdown:");
    for (ws, (beads, tests)) in &per_ws {
        eprintln!("  {ws}: {beads} beads, {tests} tests");
    }
    eprintln!();

    // Sanity: we should have at least 16 SEC beads mapped
    assert!(
        total_beads >= 16,
        "Expected at least 16 SEC beads in matrix, found {total_beads}"
    );
    // And at least 500 tests across all primary+supplementary
    let grand_total = total_primary_tests + total_supplementary_tests;
    assert!(
        grand_total >= 500,
        "Expected at least 500 total SEC tests, found {grand_total}"
    );
}

// ── Suite classification cross-check ──

#[test]
fn all_matrix_files_are_classified_in_suite_toml() {
    let root = repo_root();
    let toml_path = root.join("tests/suite_classification.toml");
    let content = std::fs::read_to_string(&toml_path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", toml_path.display()));

    let all_files: BTreeSet<&str> = SEC_MATRIX
        .iter()
        .flat_map(|e| {
            e.primary_test_files
                .iter()
                .chain(e.supplementary_test_files.iter())
                .copied()
        })
        .collect();

    let unclassified: Vec<_> = all_files
        .iter()
        .filter(|f| !content.contains(**f))
        .collect();

    assert!(
        unclassified.is_empty(),
        "SEC matrix files not in suite_classification.toml:\n{}",
        unclassified
            .iter()
            .map(|f| format!("  tests/{f}.rs"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

// ── Inline test coverage in extensions.rs ──

#[test]
fn extensions_rs_has_security_inline_tests() {
    let root = repo_root();
    let path = root.join("src/extensions.rs");
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));

    let test_count = content.matches("#[test]").count();

    // extensions.rs should have a substantial number of inline tests
    assert!(
        test_count >= 400,
        "Expected at least 400 inline #[test] in extensions.rs, found {test_count}"
    );
    eprintln!("extensions.rs inline test count: {test_count}");
}
