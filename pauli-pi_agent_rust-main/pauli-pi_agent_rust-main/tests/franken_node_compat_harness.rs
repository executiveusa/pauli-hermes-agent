//! `FrankenNode` Semantic Compatibility Harness (bd-3ar8v.7.3)
//!
//! Executes JS fixture scripts against Node.js and Bun to capture baseline
//! compatibility data, then produces a machine-readable compatibility matrix
//! artifact. When `FrankenNode` runtime is available, it will be tested against
//! these same fixtures for parity verification.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixture_dir() -> PathBuf {
    repo_root().join("tests/franken_node_compat/fixtures")
}

fn reports_dir() -> PathBuf {
    repo_root().join("tests/franken_node_compat/reports")
}

fn is_real_node(path: &str) -> bool {
    // Guard against Bun's `node` shim being mistaken for real Node in CI/worker images.
    let version_ok = Command::new(path).arg("--version").output().is_ok_and(|o| {
        if !o.status.success() {
            return false;
        }
        let version = String::from_utf8_lossy(&o.stdout);
        let mut chars = version.trim().chars();
        matches!(chars.next(), Some('v')) && chars.next().is_some_and(|c| c.is_ascii_digit())
    });
    if !version_ok {
        return false;
    }

    Command::new(path)
        .args([
            "-p",
            "(process.release && process.release.name) + ':' + !!(process.versions && process.versions.node) + ':' + !!(process.versions && process.versions.bun)",
        ])
        .output()
        .is_ok_and(|o| o.status.success() && String::from_utf8_lossy(&o.stdout).trim() == "node:true:false")
}

fn find_node() -> Option<String> {
    let candidates = [
        "/usr/bin/node",
        "/usr/local/bin/node",
        "/home/ubuntu/.nvm/versions/node/current/bin/node",
    ];
    for c in candidates {
        if Path::new(c).exists() && is_real_node(c) {
            return Some(c.to_string());
        }
    }
    // Fallback: try `which node` and verify it's real
    if let Ok(out) = Command::new("which").arg("node").output() {
        let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !path.is_empty() && is_real_node(&path) {
            return Some(path);
        }
    }
    None
}

fn find_bun() -> Option<String> {
    let candidates = [
        "/home/ubuntu/.bun/bin/bun",
        "/usr/local/bin/bun",
        "/usr/bin/bun",
    ];
    for c in candidates {
        if Path::new(c).exists() {
            return Some(c.to_string());
        }
    }
    None
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FixtureCheck {
    name: String,
    pass: bool,
    detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FixtureResult {
    fixture_id: String,
    scenario_id: String,
    #[serde(default)]
    surface: Option<String>,
    checks: Vec<FixtureCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RuntimeResult {
    runtime: String,
    version: String,
    fixture_id: String,
    scenario_id: String,
    exit_code: i32,
    all_pass: bool,
    check_count: usize,
    pass_count: usize,
    fail_count: usize,
    checks: Vec<FixtureCheck>,
    error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ScenarioVerdict {
    scenario_id: String,
    domain: String,
    criticality: String,
    node_pass_rate: f64,
    bun_pass_rate: f64,
    node_bun_parity: String,
    fixture_count: usize,
    fixtures: Vec<FixtureVerdict>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FixtureVerdict {
    fixture_id: String,
    node_all_pass: bool,
    bun_all_pass: bool,
    divergences: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CompatibilityMatrix {
    schema: String,
    bead_id: String,
    generated_at: String,
    node_version: String,
    bun_version: String,
    scenarios: Vec<ScenarioVerdict>,
    summary: MatrixSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MatrixSummary {
    total_scenarios: usize,
    total_fixtures: usize,
    total_checks: usize,
    node_pass_rate: f64,
    bun_pass_rate: f64,
    node_bun_divergence_count: usize,
    overall_parity: String,
}

fn runtime_version(runtime_path: &str) -> String {
    Command::new(runtime_path)
        .arg("--version")
        .output()
        .map_or_else(
            |_| "unknown".to_string(),
            |output| String::from_utf8_lossy(&output.stdout).trim().to_string(),
        )
}

fn ratio(pass: usize, total: usize) -> f64 {
    if total == 0 {
        return 0.0;
    }
    let pass = u32::try_from(pass).expect("pass counts should fit in u32");
    let total = u32::try_from(total).expect("total counts should fit in u32");
    f64::from(pass) / f64::from(total)
}

/// Run a JS fixture with the given runtime binary and return parsed result.
fn run_fixture(runtime_path: &str, fixture_path: &Path) -> RuntimeResult {
    let runtime_name = if runtime_path.contains("bun") {
        "bun"
    } else {
        "node"
    };

    let version = runtime_version(runtime_path);

    let output = Command::new(runtime_path).arg(fixture_path).output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let exit_code = out.status.code().unwrap_or(-1);

            match serde_json::from_str::<FixtureResult>(stdout.trim()) {
                Ok(result) => {
                    let pass_count = result.checks.iter().filter(|c| c.pass).count();
                    let fail_count = result.checks.len() - pass_count;
                    RuntimeResult {
                        runtime: runtime_name.to_string(),
                        version,
                        fixture_id: result.fixture_id,
                        scenario_id: result.scenario_id,
                        exit_code,
                        all_pass: fail_count == 0,
                        check_count: result.checks.len(),
                        pass_count,
                        fail_count,
                        checks: result.checks,
                        error: None,
                    }
                }
                Err(err) => RuntimeResult {
                    runtime: runtime_name.to_string(),
                    version,
                    fixture_id: fixture_path
                        .file_stem()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_default(),
                    scenario_id: "unknown".to_string(),
                    exit_code,
                    all_pass: false,
                    check_count: 0,
                    pass_count: 0,
                    fail_count: 0,
                    checks: Vec::new(),
                    error: Some(format!(
                        "parse error: {err}; stdout: {}",
                        &stdout[..stdout.len().min(200)]
                    )),
                },
            }
        }
        Err(err) => RuntimeResult {
            runtime: runtime_name.to_string(),
            version,
            fixture_id: fixture_path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default(),
            scenario_id: "unknown".to_string(),
            exit_code: -1,
            all_pass: false,
            check_count: 0,
            pass_count: 0,
            fail_count: 0,
            checks: Vec::new(),
            error: Some(format!("execution error: {err}")),
        },
    }
}

/// Scenario metadata from the contract.
struct ScenarioMeta {
    scenario_id: &'static str,
    domain: &'static str,
    criticality: &'static str,
    fixtures: &'static [&'static str],
}

const SCENARIOS: &[ScenarioMeta] = &[
    ScenarioMeta {
        scenario_id: "SCN-module-resolution-esm-cjs",
        domain: "module-resolution",
        criticality: "high",
        fixtures: &["esm_import.mjs", "cjs_require.cjs"],
    },
    ScenarioMeta {
        scenario_id: "SCN-node-builtin-apis",
        domain: "builtin-apis",
        criticality: "high",
        fixtures: &["builtin_apis.mjs"],
    },
    ScenarioMeta {
        scenario_id: "SCN-event-loop-io-ordering",
        domain: "event-loop-io",
        criticality: "high",
        fixtures: &["event_loop.mjs"],
    },
    ScenarioMeta {
        scenario_id: "SCN-error-and-diagnostics-parity",
        domain: "errors-diagnostics",
        criticality: "medium",
        fixtures: &["error_diagnostics.mjs"],
    },
];

fn compute_parity(node_rate: f64, bun_rate: f64) -> &'static str {
    if node_rate >= 1.0 && bun_rate >= 1.0 {
        "EXACT_PARITY"
    } else if node_rate >= 1.0 || bun_rate >= 1.0 {
        "ACCEPTABLE_SUPERSET"
    } else if node_rate >= 0.8 && bun_rate >= 0.8 {
        "PARTIAL_PARITY"
    } else {
        "INCOMPATIBLE"
    }
}

struct ScenarioRun {
    verdict: ScenarioVerdict,
    fixture_count: usize,
    check_count: usize,
    node_pass: usize,
    node_checks: usize,
    bun_pass: usize,
    bun_checks: usize,
    divergence_count: usize,
}

fn run_scenario(
    meta: &ScenarioMeta,
    node_path: &str,
    bun_path: &str,
    fixture_base: &Path,
) -> ScenarioRun {
    let mut fixture_verdicts = Vec::new();
    let mut scenario_node_pass = 0;
    let mut scenario_node_total = 0;
    let mut scenario_bun_pass = 0;
    let mut scenario_bun_total = 0;
    let mut scenario_check_count = 0;
    let mut scenario_divergences = 0;

    for fixture_file in meta.fixtures {
        let fixture_path = fixture_base.join(fixture_file);
        if !fixture_path.exists() {
            continue;
        }

        let node_result = run_fixture(node_path, &fixture_path);
        let bun_result = run_fixture(bun_path, &fixture_path);

        scenario_node_pass += node_result.pass_count;
        scenario_node_total += node_result.check_count;
        scenario_bun_pass += bun_result.pass_count;
        scenario_bun_total += bun_result.check_count;
        scenario_check_count += node_result.check_count.max(bun_result.check_count);

        // Find divergences (where node and bun disagree)
        let node_checks: HashMap<&str, bool> = node_result
            .checks
            .iter()
            .map(|c| (c.name.as_str(), c.pass))
            .collect();
        let mut divergences = Vec::new();
        for check in &bun_result.checks {
            if let Some(&node_pass) = node_checks.get(check.name.as_str()) {
                if node_pass != check.pass {
                    divergences.push(format!(
                        "{}: node={}, bun={}",
                        check.name, node_pass, check.pass
                    ));
                }
            }
        }
        scenario_divergences += divergences.len();

        fixture_verdicts.push(FixtureVerdict {
            fixture_id: node_result.fixture_id.clone(),
            node_all_pass: node_result.all_pass,
            bun_all_pass: bun_result.all_pass,
            divergences,
        });
    }

    let node_rate = ratio(scenario_node_pass, scenario_node_total);
    let bun_rate = ratio(scenario_bun_pass, scenario_bun_total);
    let fixture_count = fixture_verdicts.len();

    ScenarioRun {
        verdict: ScenarioVerdict {
            scenario_id: meta.scenario_id.to_string(),
            domain: meta.domain.to_string(),
            criticality: meta.criticality.to_string(),
            node_pass_rate: node_rate,
            bun_pass_rate: bun_rate,
            node_bun_parity: compute_parity(node_rate, bun_rate).to_string(),
            fixture_count,
            fixtures: fixture_verdicts,
        },
        fixture_count,
        check_count: scenario_check_count,
        node_pass: scenario_node_pass,
        node_checks: scenario_node_total,
        bun_pass: scenario_bun_pass,
        bun_checks: scenario_bun_total,
        divergence_count: scenario_divergences,
    }
}

/// Run all fixtures and produce the compatibility matrix.
fn run_compatibility_matrix() -> CompatibilityMatrix {
    let node_path = find_node().expect("Node.js not found");
    let bun_path = find_bun().expect("Bun not found");
    let fixture_base = fixture_dir();

    let node_version = runtime_version(&node_path);
    let bun_version = runtime_version(&bun_path);

    let mut scenarios = Vec::new();
    let mut total_fixtures = 0;
    let mut total_checks = 0;
    let mut total_node_pass = 0;
    let mut total_bun_pass = 0;
    let mut total_node_checks = 0;
    let mut total_bun_checks = 0;
    let mut total_divergences = 0;

    for meta in SCENARIOS {
        let run = run_scenario(meta, &node_path, &bun_path, &fixture_base);
        total_fixtures += run.fixture_count;
        total_checks += run.check_count;
        total_node_pass += run.node_pass;
        total_node_checks += run.node_checks;
        total_bun_pass += run.bun_pass;
        total_bun_checks += run.bun_checks;
        total_divergences += run.divergence_count;
        scenarios.push(run.verdict);
    }

    let overall_node_rate = ratio(total_node_pass, total_node_checks);
    let overall_bun_rate = ratio(total_bun_pass, total_bun_checks);

    let overall_parity = if total_divergences == 0 {
        "EXACT_PARITY"
    } else if overall_node_rate >= 0.95 && overall_bun_rate >= 0.95 {
        "ACCEPTABLE_SUPERSET"
    } else {
        "PARTIAL_PARITY"
    };

    CompatibilityMatrix {
        schema: "pi.frankennode.compatibility_matrix.v1".to_string(),
        bead_id: "bd-3ar8v.7.3".to_string(),
        generated_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        node_version,
        bun_version,
        scenarios,
        summary: MatrixSummary {
            total_scenarios: SCENARIOS.len(),
            total_fixtures,
            total_checks,
            node_pass_rate: overall_node_rate,
            bun_pass_rate: overall_bun_rate,
            node_bun_divergence_count: total_divergences,
            overall_parity: overall_parity.to_string(),
        },
    }
}

// ─── Tests ───

/// Helper: skip test if runtime not available.
macro_rules! require_node {
    () => {
        match find_node() {
            Some(p) => p,
            None => {
                eprintln!("SKIP: Node.js not found on this machine");
                return;
            }
        }
    };
}

macro_rules! require_bun {
    () => {
        match find_bun() {
            Some(p) => p,
            None => {
                eprintln!("SKIP: Bun not found on this machine");
                return;
            }
        }
    };
}

#[test]
fn node_detection_rejects_bun_node_shim_when_present() {
    let bun_node_shim = "/home/ubuntu/.bun/bin/node";
    if !Path::new(bun_node_shim).exists() {
        eprintln!("SKIP: Bun node shim not present on this machine");
        return;
    }
    assert!(
        !is_real_node(bun_node_shim),
        "Bun's node shim must never be accepted as a real Node runtime: {bun_node_shim}"
    );
}

fn assert_fixture_all_pass(result: &RuntimeResult, label: &str) {
    assert!(
        result.error.is_none(),
        "{label} fixture error: {:?}",
        result.error
    );
    assert!(
        result.all_pass,
        "{label}: {}/{} checks passed. Failures: {:?}",
        result.pass_count,
        result.check_count,
        result.checks.iter().filter(|c| !c.pass).collect::<Vec<_>>()
    );
}

#[test]
fn compat_harness_node_esm_import_all_pass() {
    let node = require_node!();
    let result = run_fixture(&node, &fixture_dir().join("esm_import.mjs"));
    assert_fixture_all_pass(&result, "Node ESM import");
}

#[test]
fn compat_harness_node_cjs_require_all_pass() {
    let node = require_node!();
    let result = run_fixture(&node, &fixture_dir().join("cjs_require.cjs"));
    assert_fixture_all_pass(&result, "Node CJS require");
}

#[test]
fn compat_harness_node_builtin_apis_all_pass() {
    let node = require_node!();
    let result = run_fixture(&node, &fixture_dir().join("builtin_apis.mjs"));
    assert_fixture_all_pass(&result, "Node builtin APIs");
}

#[test]
fn compat_harness_node_event_loop_ordering() {
    let node = require_node!();
    let result = run_fixture(&node, &fixture_dir().join("event_loop.mjs"));
    assert_fixture_all_pass(&result, "Node event loop ordering");
}

#[test]
fn compat_harness_node_error_diagnostics() {
    let node = require_node!();
    let result = run_fixture(&node, &fixture_dir().join("error_diagnostics.mjs"));
    assert_fixture_all_pass(&result, "Node error diagnostics");
}

#[test]
fn compat_harness_bun_esm_import_all_pass() {
    let bun = require_bun!();
    let result = run_fixture(&bun, &fixture_dir().join("esm_import.mjs"));
    assert_fixture_all_pass(&result, "Bun ESM import");
}

#[test]
fn compat_harness_bun_cjs_require_all_pass() {
    let bun = require_bun!();
    let result = run_fixture(&bun, &fixture_dir().join("cjs_require.cjs"));
    assert_fixture_all_pass(&result, "Bun CJS require");
}

#[test]
fn compat_harness_bun_builtin_apis_all_pass() {
    let bun = require_bun!();
    let result = run_fixture(&bun, &fixture_dir().join("builtin_apis.mjs"));
    assert_fixture_all_pass(&result, "Bun builtin APIs");
}

#[test]
fn compat_harness_bun_event_loop_ordering() {
    let bun = require_bun!();
    let result = run_fixture(&bun, &fixture_dir().join("event_loop.mjs"));
    assert_fixture_all_pass(&result, "Bun event loop ordering");
}

#[test]
fn compat_harness_captures_node_bun_divergences() {
    let node = require_node!();
    let bun = require_bun!();
    let fixture = fixture_dir().join("error_diagnostics.mjs");

    let node_result = run_fixture(&node, &fixture);
    let bun_result = run_fixture(&bun, &fixture);

    // Bun is known to diverge on stack_has_function_names
    let node_checks: HashMap<&str, bool> = node_result
        .checks
        .iter()
        .map(|c| (c.name.as_str(), c.pass))
        .collect();
    let mut divergences = Vec::new();
    for check in &bun_result.checks {
        if let Some(&node_pass) = node_checks.get(check.name.as_str()) {
            if node_pass != check.pass {
                divergences.push(check.name.clone());
            }
        }
    }

    // We expect at least the stack_has_function_names divergence
    assert!(
        !divergences.is_empty(),
        "expected at least one Node/Bun divergence in error_diagnostics"
    );
    println!(
        "Captured {} divergence(s): {:?}",
        divergences.len(),
        divergences
    );
}

#[test]
fn generate_compatibility_matrix() {
    if find_node().is_none() || find_bun().is_none() {
        eprintln!("SKIP: generate_compatibility_matrix requires both Node.js and Bun");
        return;
    }
    let matrix = run_compatibility_matrix();

    // Validate structure
    assert_eq!(matrix.schema, "pi.frankennode.compatibility_matrix.v1");
    assert_eq!(matrix.bead_id, "bd-3ar8v.7.3");
    assert_eq!(matrix.summary.total_scenarios, 4);
    assert!(
        matrix.summary.total_fixtures >= 5,
        "expected at least 5 fixtures, got {}",
        matrix.summary.total_fixtures
    );
    assert!(
        matrix.summary.total_checks >= 20,
        "expected at least 20 checks, got {}",
        matrix.summary.total_checks
    );

    // Node should pass all checks
    assert!(
        matrix.summary.node_pass_rate >= 1.0,
        "Node pass rate should be 100%, got {:.1}%",
        matrix.summary.node_pass_rate * 100.0
    );

    // Bun has known divergences
    assert!(
        matrix.summary.bun_pass_rate >= 0.9,
        "Bun pass rate should be >= 90%, got {:.1}%",
        matrix.summary.bun_pass_rate * 100.0
    );

    // Should capture divergences
    assert!(
        matrix.summary.node_bun_divergence_count >= 1,
        "should capture at least 1 Node/Bun divergence"
    );
    assert_eq!(matrix.summary.overall_parity, "ACCEPTABLE_SUPERSET");

    // High-criticality scenarios should have good rates
    for scenario in &matrix.scenarios {
        if scenario.criticality == "high" {
            assert!(
                scenario.node_pass_rate >= 1.0,
                "high-criticality scenario {} should have 100% Node pass rate",
                scenario.scenario_id
            );
        }
    }

    // Write artifact
    let reports = reports_dir();
    std::fs::create_dir_all(&reports).expect("create reports dir");
    let artifact_path = reports.join("compatibility_matrix.json");
    let json = serde_json::to_string_pretty(&matrix).expect("serialize matrix");
    std::fs::write(&artifact_path, &json).expect("write matrix artifact");

    println!("\n=== FrankenNode Compatibility Matrix ===");
    println!("  Node version: {}", matrix.node_version);
    println!("  Bun version:  {}", matrix.bun_version);
    println!("  Scenarios:    {}", matrix.summary.total_scenarios);
    println!("  Fixtures:     {}", matrix.summary.total_fixtures);
    println!("  Checks:       {}", matrix.summary.total_checks);
    println!(
        "  Node rate:    {:.1}%",
        matrix.summary.node_pass_rate * 100.0
    );
    println!(
        "  Bun rate:     {:.1}%",
        matrix.summary.bun_pass_rate * 100.0
    );
    println!(
        "  Divergences:  {}",
        matrix.summary.node_bun_divergence_count
    );
    println!("  Parity:       {}", matrix.summary.overall_parity);
    for scenario in &matrix.scenarios {
        println!(
            "  [{:6}] {}: node={:.0}% bun={:.0}% → {}",
            scenario.criticality,
            scenario.scenario_id,
            scenario.node_pass_rate * 100.0,
            scenario.bun_pass_rate * 100.0,
            scenario.node_bun_parity,
        );
    }
    println!("  Artifact: {}", artifact_path.display());
}
