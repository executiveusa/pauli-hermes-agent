//! Unified CI Evidence Bundle — collects all test artifacts into a single
//! structured bundle per CI run (bd-1f42.6.8).
//!
//! Produces:
//! - `tests/evidence_bundle/index.json` — machine-readable index with pointers
//!   to every section.
//! - `tests/evidence_bundle/bundle_report.md` — human-readable summary with
//!   pass/fail verdict for every section.
//! - `tests/evidence_bundle/events.jsonl` — JSONL event log of all collected
//!   artifacts.
//!
//! The bundle unifies:
//! 1. Extension conformance reports (summaries, baselines, gate verdicts)
//! 2. Extension diagnostics (dossiers, health delta, provider compat)
//! 3. E2E test results and transcripts
//! 4. Unit coverage summaries
//! 5. Quarantine audit trails
//! 6. Release gate verdicts
//! 7. Performance budgets
//! 8. Traceability matrices
//!
//! Run:
//!   cargo test --test `ci_evidence_bundle` -- --nocapture

use serde_json::Value;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn load_json(path: &Path) -> Option<Value> {
    let text = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}

/// A section in the evidence bundle.
#[derive(Debug, Clone, serde::Serialize)]
struct BundleSection {
    id: String,
    label: String,
    category: String,
    status: String, // "present", "missing", "invalid"
    #[serde(skip_serializing_if = "Option::is_none")]
    artifact_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    schema: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    summary: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    diagnostics: Option<String>,
    file_count: usize,
    total_bytes: u64,
}

/// The full evidence bundle index.
#[derive(Debug, serde::Serialize)]
struct EvidenceBundle {
    schema: String,
    generated_at: String,
    git_ref: String,
    ci_run_id: String,
    sections: Vec<BundleSection>,
    summary: BundleSummary,
}

/// Summary statistics for the bundle.
#[derive(Debug, serde::Serialize)]
struct BundleSummary {
    total_sections: usize,
    present_sections: usize,
    missing_sections: usize,
    invalid_sections: usize,
    total_artifacts: usize,
    total_bytes: u64,
    verdict: String, // "complete", "partial", "insufficient"
}

/// Artifact source descriptor.
struct ArtifactSource {
    id: &'static str,
    label: &'static str,
    category: &'static str,
    path: &'static str,
    /// Expected schema identifier (if JSON with `schema` field).
    expected_schema: Option<&'static str>,
    /// If true, this is a directory and we count all files inside.
    is_directory: bool,
    /// If true, missing this artifact downgrades verdict.
    required: bool,
}

const PERF3X_LINEAGE_CONTRACT_SCHEMA: &str = "pi.perf3x.lineage_contract.v1";
const PERF3X_LINEAGE_CONTRACT_ARTIFACTS: &str = "tests/ext_conformance/reports/gate/must_pass_gate_verdict.json | \
tests/ext_conformance/reports/conformance_summary.json | \
tests/perf/reports/stress_triage.json";
const PERF3X_LINEAGE_MAX_ARTIFACT_SPAN_DAYS: i64 = 14;
const PARAMETER_SWEEPS_MISSING_DIAGNOSTIC: &str = "parameter_sweeps artifact not found (expected tests/perf/reports, tests/perf/runs/results, or tests/e2e_results/*/results)";

const ARTIFACT_SOURCES: &[ArtifactSource] = &[
    // ── Extension conformance ──
    ArtifactSource {
        id: "conformance_summary",
        label: "Extension conformance summary",
        category: "conformance",
        path: "tests/ext_conformance/reports/conformance_summary.json",
        expected_schema: Some("pi.ext.conformance_summary"),
        is_directory: false,
        required: true,
    },
    ArtifactSource {
        id: "conformance_baseline",
        label: "Conformance baseline",
        category: "conformance",
        path: "tests/ext_conformance/reports/conformance_baseline.json",
        expected_schema: Some("pi.ext.conformance_baseline"),
        is_directory: false,
        required: true,
    },
    ArtifactSource {
        id: "conformance_events",
        label: "Conformance event log",
        category: "conformance",
        path: "tests/ext_conformance/reports/conformance_events.jsonl",
        expected_schema: None,
        is_directory: false,
        required: false,
    },
    ArtifactSource {
        id: "conformance_report_md",
        label: "Conformance report (Markdown)",
        category: "conformance",
        path: "tests/ext_conformance/reports/CONFORMANCE_REPORT.md",
        expected_schema: None,
        is_directory: false,
        required: false,
    },
    ArtifactSource {
        id: "regression_verdict",
        label: "Regression gate verdict",
        category: "conformance",
        path: "tests/ext_conformance/reports/regression_verdict.json",
        expected_schema: Some("pi.conformance.regression_gate"),
        is_directory: false,
        required: false,
    },
    ArtifactSource {
        id: "conformance_trend",
        label: "Conformance trend data",
        category: "conformance",
        path: "tests/ext_conformance/reports/conformance_trend.jsonl",
        expected_schema: None,
        is_directory: false,
        required: false,
    },
    // ── Extension diagnostics ──
    ArtifactSource {
        id: "must_pass_gate",
        label: "Must-pass gate verdict (208 extensions)",
        category: "diagnostics",
        path: "tests/ext_conformance/reports/gate/must_pass_gate_verdict.json",
        expected_schema: Some("pi.ext.must_pass_gate"),
        is_directory: false,
        required: true,
    },
    ArtifactSource {
        id: "must_pass_gate_events",
        label: "Must-pass gate event log",
        category: "diagnostics",
        path: "tests/ext_conformance/reports/gate/must_pass_events.jsonl",
        expected_schema: None,
        is_directory: false,
        required: false,
    },
    ArtifactSource {
        id: "failure_dossiers",
        label: "Per-extension failure dossiers",
        category: "diagnostics",
        path: "tests/ext_conformance/reports/dossiers",
        expected_schema: None,
        is_directory: true,
        required: false,
    },
    ArtifactSource {
        id: "health_delta",
        label: "Health & regression delta report",
        category: "diagnostics",
        path: "tests/ext_conformance/reports/health_delta",
        expected_schema: None,
        is_directory: true,
        required: true,
    },
    ArtifactSource {
        id: "provider_compat",
        label: "Provider compatibility matrix",
        category: "diagnostics",
        path: "tests/ext_conformance/reports/provider_compat",
        expected_schema: None,
        is_directory: true,
        required: false,
    },
    ArtifactSource {
        id: "sharded_reports",
        label: "Sharded extension matrix reports",
        category: "diagnostics",
        path: "tests/ext_conformance/reports/sharded",
        expected_schema: None,
        is_directory: true,
        required: false,
    },
    ArtifactSource {
        id: "journey_reports",
        label: "Extension journey reports",
        category: "diagnostics",
        path: "tests/ext_conformance/reports/journeys",
        expected_schema: None,
        is_directory: true,
        required: true,
    },
    ArtifactSource {
        id: "auto_repair_summary",
        label: "Auto-repair summary",
        category: "diagnostics",
        path: "tests/ext_conformance/reports/auto_repair_summary.json",
        expected_schema: Some("pi.ext.auto_repair_summary"),
        is_directory: false,
        required: false,
    },
    // ── E2E results ──
    ArtifactSource {
        id: "e2e_results",
        label: "E2E test results",
        category: "e2e",
        path: "tests/e2e_results",
        expected_schema: None,
        is_directory: true,
        required: false,
    },
    // ── Quarantine ──
    ArtifactSource {
        id: "quarantine_report",
        label: "Quarantine report",
        category: "quarantine",
        path: "tests/quarantine_report.json",
        expected_schema: Some("pi.test.quarantine_report"),
        is_directory: false,
        required: false,
    },
    ArtifactSource {
        id: "quarantine_audit",
        label: "Quarantine audit trail",
        category: "quarantine",
        path: "tests/quarantine_audit.jsonl",
        expected_schema: None,
        is_directory: false,
        required: false,
    },
    // ── Performance ──
    ArtifactSource {
        id: "perf_budget_summary",
        label: "Performance budget summary",
        category: "performance",
        path: "tests/perf/reports/budget_summary.json",
        expected_schema: None,
        is_directory: false,
        required: false,
    },
    ArtifactSource {
        id: "perf_comparison",
        label: "PERF-3X comparison report",
        category: "performance",
        path: "tests/perf/reports/perf_comparison.json",
        expected_schema: Some("pi.ext.perf_comparison"),
        is_directory: false,
        required: true,
    },
    ArtifactSource {
        id: "parameter_sweeps",
        label: "PERF-3X parameter sweeps report",
        category: "performance",
        path: "tests/perf/reports/parameter_sweeps.json",
        expected_schema: Some("pi.perf.parameter_sweeps"),
        is_directory: false,
        required: true,
    },
    ArtifactSource {
        id: "stress_triage",
        label: "PERF-3X stress triage report",
        category: "performance",
        path: "tests/perf/reports/stress_triage.json",
        expected_schema: Some("pi.ext.stress_triage"),
        is_directory: false,
        required: true,
    },
    ArtifactSource {
        id: "load_time_benchmark",
        label: "Extension load-time benchmark",
        category: "performance",
        path: "tests/ext_conformance/reports/load_time_benchmark.json",
        expected_schema: None,
        is_directory: false,
        required: false,
    },
    // ── Security & provenance ──
    ArtifactSource {
        id: "risk_review",
        label: "Security and licensing risk review",
        category: "security",
        path: "tests/ext_conformance/artifacts/RISK_REVIEW.json",
        expected_schema: None,
        is_directory: false,
        required: true,
    },
    ArtifactSource {
        id: "provenance_verification",
        label: "Extension provenance verification",
        category: "security",
        path: "tests/ext_conformance/artifacts/PROVENANCE_VERIFICATION.json",
        expected_schema: None,
        is_directory: false,
        required: true,
    },
    // ── Traceability ──
    ArtifactSource {
        id: "traceability_matrix",
        label: "Requirement-to-test traceability matrix",
        category: "traceability",
        path: "docs/traceability_matrix.json",
        expected_schema: None,
        is_directory: false,
        required: true,
    },
    // ── Inventory ──
    ArtifactSource {
        id: "extension_inventory",
        label: "Extension inventory",
        category: "inventory",
        path: "tests/ext_conformance/reports/inventory.json",
        expected_schema: Some("pi.ext.inventory"),
        is_directory: false,
        required: false,
    },
    ArtifactSource {
        id: "inclusion_manifest",
        label: "Extension inclusion manifest",
        category: "inventory",
        path: "tests/ext_conformance/reports/inclusion_manifest",
        expected_schema: None,
        is_directory: true,
        required: false,
    },
];

/// Count files and total bytes in a directory recursively.
fn dir_stats(path: &Path) -> (usize, u64) {
    let mut count = 0_usize;
    let mut bytes = 0_u64;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let ft = entry.file_type();
            if ft.as_ref().is_ok_and(std::fs::FileType::is_dir) {
                let (c, b) = dir_stats(&entry.path());
                count += c;
                bytes += b;
            } else if ft.as_ref().is_ok_and(std::fs::FileType::is_file) {
                count += 1;
                bytes += entry.metadata().map_or(0, |m| m.len());
            }
        }
    }
    (count, bytes)
}

fn validate_must_pass_gate_payload(val: &Value) -> Result<Value, String> {
    let status = val
        .get("status")
        .and_then(Value::as_str)
        .ok_or_else(|| "must_pass_gate: missing status".to_string())?;
    if !matches!(status, "pass" | "warn" | "fail") {
        return Err(format!("must_pass_gate: unexpected status '{status}'"));
    }

    let generated_at = val
        .get("generated_at")
        .and_then(Value::as_str)
        .ok_or_else(|| "must_pass_gate: missing generated_at".to_string())?;
    if generated_at.trim().is_empty() {
        return Err("must_pass_gate: generated_at is empty".to_string());
    }

    let run_id = val
        .get("run_id")
        .and_then(Value::as_str)
        .ok_or_else(|| "must_pass_gate: missing run_id".to_string())?;
    if run_id.trim().is_empty() {
        return Err("must_pass_gate: run_id is empty".to_string());
    }

    let correlation_id = val
        .get("correlation_id")
        .and_then(Value::as_str)
        .ok_or_else(|| "must_pass_gate: missing correlation_id".to_string())?;
    if correlation_id.trim().is_empty() {
        return Err("must_pass_gate: correlation_id is empty".to_string());
    }

    let observed = val
        .get("observed")
        .ok_or_else(|| "must_pass_gate: missing observed object".to_string())?;
    let must_pass_total = observed
        .get("must_pass_total")
        .and_then(Value::as_u64)
        .ok_or_else(|| "must_pass_gate: missing observed.must_pass_total".to_string())?;
    let must_pass_passed = observed
        .get("must_pass_passed")
        .and_then(Value::as_u64)
        .ok_or_else(|| "must_pass_gate: missing observed.must_pass_passed".to_string())?;

    if must_pass_total == 0 {
        return Err("must_pass_gate: observed.must_pass_total must be > 0".to_string());
    }
    if must_pass_passed > must_pass_total {
        return Err(format!(
            "must_pass_gate: observed.must_pass_passed ({must_pass_passed}) exceeds total ({must_pass_total})"
        ));
    }

    Ok(serde_json::json!({
        "status": status,
        "must_pass_total": must_pass_total,
        "must_pass_passed": must_pass_passed,
        "must_pass_failed": observed.get("must_pass_failed"),
        "must_pass_skipped": observed.get("must_pass_skipped"),
        "must_pass_pass_rate_pct": observed.get("must_pass_pass_rate_pct"),
        "run_id": run_id,
        "correlation_id": correlation_id,
        "generated_at": generated_at,
    }))
}

fn validate_perf_comparison_payload(val: &Value) -> Result<Value, String> {
    let generated_at = val
        .get("generated_at")
        .and_then(Value::as_str)
        .ok_or_else(|| "perf_comparison: missing generated_at".to_string())?;
    if generated_at.trim().is_empty() {
        return Err("perf_comparison: generated_at is empty".to_string());
    }

    let summary = val
        .get("summary")
        .and_then(Value::as_object)
        .ok_or_else(|| "perf_comparison: missing summary object".to_string())?;
    let overall_verdict = summary
        .get("overall_verdict")
        .and_then(Value::as_str)
        .map_or("", str::trim);
    if overall_verdict.is_empty() {
        return Err("perf_comparison: summary.overall_verdict is missing/empty".to_string());
    }

    let faster_count = summary
        .get("faster_count")
        .and_then(Value::as_u64)
        .ok_or_else(|| "perf_comparison: missing summary.faster_count".to_string())?;
    let slower_count = summary
        .get("slower_count")
        .and_then(Value::as_u64)
        .ok_or_else(|| "perf_comparison: missing summary.slower_count".to_string())?;
    let comparable_count = summary
        .get("comparable_count")
        .and_then(Value::as_u64)
        .ok_or_else(|| "perf_comparison: missing summary.comparable_count".to_string())?;

    Ok(serde_json::json!({
        "generated_at": generated_at,
        "overall_verdict": overall_verdict,
        "faster_count": faster_count,
        "slower_count": slower_count,
        "comparable_count": comparable_count,
    }))
}

fn validate_parameter_sweeps_payload(val: &Value) -> Result<Value, String> {
    let generated_at = val
        .get("generated_at")
        .and_then(Value::as_str)
        .ok_or_else(|| "parameter_sweeps: missing generated_at".to_string())?;
    if generated_at.trim().is_empty() {
        return Err("parameter_sweeps: generated_at is empty".to_string());
    }

    let readiness = val
        .get("readiness")
        .and_then(Value::as_object)
        .ok_or_else(|| "parameter_sweeps: missing readiness object".to_string())?;
    let readiness_status = readiness
        .get("status")
        .and_then(Value::as_str)
        .ok_or_else(|| "parameter_sweeps: missing readiness.status".to_string())?;
    if !matches!(readiness_status, "ready" | "blocked") {
        return Err(format!(
            "parameter_sweeps: readiness.status must be ready|blocked, got '{readiness_status}'"
        ));
    }

    let ready_for_phase5 = readiness
        .get("ready_for_phase5")
        .and_then(Value::as_bool)
        .ok_or_else(|| "parameter_sweeps: missing readiness.ready_for_phase5 bool".to_string())?;
    let blocking_reasons = readiness
        .get("blocking_reasons")
        .and_then(Value::as_array)
        .ok_or_else(|| "parameter_sweeps: missing readiness.blocking_reasons array".to_string())?;

    let source_identity = val
        .get("source_identity")
        .and_then(Value::as_object)
        .ok_or_else(|| "parameter_sweeps: missing source_identity object".to_string())?;
    let source_artifact = source_identity
        .get("source_artifact")
        .and_then(Value::as_str)
        .map_or("", str::trim);
    if source_artifact.is_empty() {
        return Err(
            "parameter_sweeps: source_identity.source_artifact is missing/empty".to_string(),
        );
    }

    Ok(serde_json::json!({
        "generated_at": generated_at,
        "readiness_status": readiness_status,
        "ready_for_phase5": ready_for_phase5,
        "blocking_reasons_count": blocking_reasons.len(),
        "source_artifact": source_artifact,
    }))
}

fn missing_section(source: &ArtifactSource, diagnostics: &str) -> BundleSection {
    BundleSection {
        id: source.id.to_string(),
        label: source.label.to_string(),
        category: source.category.to_string(),
        status: "missing".to_string(),
        artifact_path: Some(source.path.to_string()),
        schema: None,
        summary: None,
        diagnostics: Some(diagnostics.to_string()),
        file_count: 0,
        total_bytes: 0,
    }
}

fn collect_directory_section(full_path: &Path, source: &ArtifactSource) -> BundleSection {
    if !full_path.is_dir() {
        return missing_section(source, "Directory not found");
    }

    let (file_count, total_bytes) = dir_stats(full_path);
    BundleSection {
        id: source.id.to_string(),
        label: source.label.to_string(),
        category: source.category.to_string(),
        status: if file_count > 0 {
            "present".to_string()
        } else {
            "missing".to_string()
        },
        artifact_path: Some(source.path.to_string()),
        schema: None,
        summary: Some(serde_json::json!({
            "file_count": file_count,
            "total_bytes": total_bytes,
        })),
        diagnostics: None,
        file_count,
        total_bytes,
    }
}

#[derive(Debug, Default)]
struct JsonFileAnalysis {
    status: String,
    schema: Option<String>,
    summary: Option<Value>,
    diagnostics: Option<String>,
}

fn artifact_uses_json_schema(source: &ArtifactSource) -> bool {
    Path::new(source.path)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
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

fn analyze_json_file(full_path: &Path, source: &ArtifactSource) -> JsonFileAnalysis {
    let Some(val) = load_json(full_path) else {
        return JsonFileAnalysis {
            status: "invalid".to_string(),
            diagnostics: Some("Failed to parse JSON".to_string()),
            ..JsonFileAnalysis::default()
        };
    };

    let mut analysis = JsonFileAnalysis {
        status: "present".to_string(),
        schema: val.get("schema").and_then(Value::as_str).map(String::from),
        ..JsonFileAnalysis::default()
    };

    if let Some(expected) = source.expected_schema {
        if let Some(actual) = analysis.schema.as_deref() {
            if !actual.starts_with(expected) {
                analysis.status = "invalid".to_string();
                analysis.diagnostics = Some(format!(
                    "Schema mismatch: expected prefix '{expected}', found '{actual}'"
                ));
            }
        } else {
            analysis.status = "invalid".to_string();
            analysis.diagnostics = Some(format!(
                "Missing schema field (expected prefix '{expected}')"
            ));
        }
    }

    if source.id == "must_pass_gate" {
        match validate_must_pass_gate_payload(&val) {
            Ok(payload) => {
                analysis.summary = Some(payload);
            }
            Err(err) => {
                analysis.status = "invalid".to_string();
                analysis.diagnostics = Some(err);
            }
        }
    } else if source.id == "perf_comparison" {
        match validate_perf_comparison_payload(&val) {
            Ok(payload) => {
                analysis.summary = Some(payload);
            }
            Err(err) => {
                analysis.status = "invalid".to_string();
                analysis.diagnostics = Some(err);
            }
        }
    } else if source.id == "parameter_sweeps" {
        match validate_parameter_sweeps_payload(&val) {
            Ok(payload) => {
                analysis.summary = Some(payload);
            }
            Err(err) => {
                analysis.status = "invalid".to_string();
                analysis.diagnostics = Some(err);
            }
        }
    } else {
        analysis.summary = extract_summary(&val, source.id);
    }

    analysis
}

fn collect_file_section(full_path: &Path, source: &ArtifactSource) -> BundleSection {
    if !full_path.is_file() {
        return missing_section(source, "File not found");
    }

    let file_size = std::fs::metadata(full_path).map_or(0, |m| m.len());
    let (status, schema, summary, diagnostics) = if artifact_uses_json_schema(source) {
        let analysis = analyze_json_file(full_path, source);
        (
            analysis.status,
            analysis.schema,
            analysis.summary,
            analysis.diagnostics,
        )
    } else {
        ("present".to_string(), None, None, None)
    };

    BundleSection {
        id: source.id.to_string(),
        label: source.label.to_string(),
        category: source.category.to_string(),
        status,
        artifact_path: Some(source.path.to_string()),
        schema,
        summary,
        diagnostics,
        file_count: 1,
        total_bytes: file_size,
    }
}

fn collect_parameter_sweeps_section(root: &Path, source: &ArtifactSource) -> BundleSection {
    let Some(full_path) = find_latest_parameter_sweeps(root) else {
        return missing_section(source, PARAMETER_SWEEPS_MISSING_DIAGNOSTIC);
    };

    let file_size = std::fs::metadata(&full_path).map_or(0, |m| m.len());
    let analysis = analyze_json_file(&full_path, source);
    let artifact_path = full_path.strip_prefix(root).map_or_else(
        |_| full_path.display().to_string(),
        |relative| relative.display().to_string(),
    );

    BundleSection {
        id: source.id.to_string(),
        label: source.label.to_string(),
        category: source.category.to_string(),
        status: analysis.status,
        artifact_path: Some(artifact_path),
        schema: analysis.schema,
        summary: analysis.summary,
        diagnostics: analysis.diagnostics,
        file_count: 1,
        total_bytes: file_size,
    }
}

/// Collect a section from an artifact source.
fn collect_section(root: &Path, source: &ArtifactSource) -> BundleSection {
    if source.id == "parameter_sweeps" {
        return collect_parameter_sweeps_section(root, source);
    }

    let full_path = root.join(source.path);

    if source.is_directory {
        collect_directory_section(&full_path, source)
    } else {
        collect_file_section(&full_path, source)
    }
}

/// Extract a lightweight summary from a JSON artifact for the bundle index.
fn extract_summary(val: &Value, section_id: &str) -> Option<Value> {
    match section_id {
        "conformance_summary" => {
            let counts = val.get("counts")?;
            Some(serde_json::json!({
                "total": counts.get("total"),
                "pass": counts.get("pass"),
                "fail": counts.get("fail"),
                "pass_rate_pct": val.get("pass_rate_pct"),
                "generated_at": val.get("generated_at"),
            }))
        }
        "conformance_baseline" => {
            let ec = val.get("extension_conformance")?;
            Some(serde_json::json!({
                "tested": ec.get("tested"),
                "passed": ec.get("passed"),
                "failed": ec.get("failed"),
                "pass_rate_pct": ec.get("pass_rate_pct"),
                "generated_at": val.get("generated_at"),
            }))
        }
        "regression_verdict" => Some(serde_json::json!({
            "status": val.get("status"),
            "effective_pass_rate_pct": val.get("effective_pass_rate_pct"),
        })),
        "quarantine_report" => Some(serde_json::json!({
            "active_count": val.get("active_count"),
            "expired_count": val.get("expired_count"),
        })),
        "perf_comparison" => {
            let summary = val.get("summary")?;
            Some(serde_json::json!({
                "generated_at": val.get("generated_at"),
                "overall_verdict": summary.get("overall_verdict"),
                "faster_count": summary.get("faster_count"),
                "slower_count": summary.get("slower_count"),
                "comparable_count": summary.get("comparable_count"),
            }))
        }
        "parameter_sweeps" => {
            let readiness = val.get("readiness")?;
            Some(serde_json::json!({
                "generated_at": val.get("generated_at"),
                "readiness_status": readiness.get("status"),
                "ready_for_phase5": readiness.get("ready_for_phase5"),
                "blocking_reasons_count": readiness.get("blocking_reasons").and_then(Value::as_array).map(Vec::len),
            }))
        }
        "stress_triage" => Some(serde_json::json!({
            "pass": val.get("pass"),
            "generated_at": val.get("generated_at"),
        })),
        "extension_inventory" => Some(serde_json::json!({
            "total_extensions": val.get("total_extensions"),
        })),
        _ => None,
    }
}

fn summary_string_field(
    sections: &[BundleSection],
    section_id: &str,
    field: &str,
) -> Result<String, String> {
    let section = sections
        .iter()
        .find(|section| section.id == section_id)
        .ok_or_else(|| format!("missing required section '{section_id}'"))?;
    if section.status != "present" {
        return Err(format!(
            "section '{section_id}' must be present, found status '{}'",
            section.status
        ));
    }
    let summary = section
        .summary
        .as_ref()
        .ok_or_else(|| format!("section '{section_id}' missing summary payload"))?;
    let value = summary
        .get(field)
        .and_then(Value::as_str)
        .map_or("", str::trim);
    if value.is_empty() {
        return Err(format!(
            "section '{section_id}' missing non-empty summary field '{field}'"
        ));
    }
    Ok(value.to_string())
}

fn summary_generated_at(
    sections: &[BundleSection],
    section_id: &str,
) -> Result<chrono::DateTime<chrono::Utc>, String> {
    let generated_at = summary_string_field(sections, section_id, "generated_at")?;
    chrono::DateTime::parse_from_rfc3339(&generated_at)
        .map(|ts| ts.with_timezone(&chrono::Utc))
        .map_err(|err| {
            format!("section '{section_id}' has invalid generated_at '{generated_at}': {err}")
        })
}

fn validate_perf3x_lineage_contract(sections: &[BundleSection]) -> Result<Value, String> {
    let run_id = summary_string_field(sections, "must_pass_gate", "run_id")?;
    let correlation_id = summary_string_field(sections, "must_pass_gate", "correlation_id")?;
    if !correlation_id.contains(&run_id) {
        return Err(format!(
            "must_pass_gate correlation_id '{correlation_id}' must include run_id '{run_id}'"
        ));
    }

    let must_pass_generated_at = summary_generated_at(sections, "must_pass_gate")?;
    let conformance_generated_at = summary_generated_at(sections, "conformance_summary")?;
    let stress_generated_at = summary_generated_at(sections, "stress_triage")?;

    let oldest = [
        must_pass_generated_at,
        conformance_generated_at,
        stress_generated_at,
    ]
    .iter()
    .min()
    .copied()
    .expect("lineage timestamp set is non-empty");
    let newest = [
        must_pass_generated_at,
        conformance_generated_at,
        stress_generated_at,
    ]
    .iter()
    .max()
    .copied()
    .expect("lineage timestamp set is non-empty");

    let span = newest.signed_duration_since(oldest);
    if span > chrono::Duration::days(PERF3X_LINEAGE_MAX_ARTIFACT_SPAN_DAYS) {
        return Err(format!(
            "PERF-3X lineage span exceeds {PERF3X_LINEAGE_MAX_ARTIFACT_SPAN_DAYS} days \
             for run_id '{run_id}' (oldest={oldest}, newest={newest})"
        ));
    }

    Ok(serde_json::json!({
        "run_id": run_id,
        "correlation_id": correlation_id,
        "must_pass_generated_at": must_pass_generated_at.to_rfc3339(),
        "conformance_generated_at": conformance_generated_at.to_rfc3339(),
        "stress_generated_at": stress_generated_at.to_rfc3339(),
        "artifact_span_minutes": span.num_minutes(),
        "max_allowed_span_days": PERF3X_LINEAGE_MAX_ARTIFACT_SPAN_DAYS,
    }))
}

fn build_perf3x_lineage_section(sections: &[BundleSection]) -> BundleSection {
    match validate_perf3x_lineage_contract(sections) {
        Ok(summary) => BundleSection {
            id: "perf3x_lineage_contract".to_string(),
            label: "PERF-3X lineage coherence contract".to_string(),
            category: "performance".to_string(),
            status: "present".to_string(),
            artifact_path: Some(PERF3X_LINEAGE_CONTRACT_ARTIFACTS.to_string()),
            schema: Some(PERF3X_LINEAGE_CONTRACT_SCHEMA.to_string()),
            summary: Some(summary),
            diagnostics: None,
            file_count: 0,
            total_bytes: 0,
        },
        Err(err) => BundleSection {
            id: "perf3x_lineage_contract".to_string(),
            label: "PERF-3X lineage coherence contract".to_string(),
            category: "performance".to_string(),
            status: "invalid".to_string(),
            artifact_path: Some(PERF3X_LINEAGE_CONTRACT_ARTIFACTS.to_string()),
            schema: Some(PERF3X_LINEAGE_CONTRACT_SCHEMA.to_string()),
            summary: None,
            diagnostics: Some(err),
            file_count: 0,
            total_bytes: 0,
        },
    }
}

/// Build the unified evidence bundle.
///
/// Run with:
/// `cargo test --test ci_evidence_bundle -- build_evidence_bundle --nocapture`
#[test]
#[allow(clippy::too_many_lines, clippy::cast_precision_loss)]
fn build_evidence_bundle() {
    use chrono::{SecondsFormat, Utc};
    use std::fmt::Write as _;

    let root = repo_root();
    let bundle_dir = root.join("tests").join("evidence_bundle");
    let _ = std::fs::create_dir_all(&bundle_dir);

    let git_ref = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(&root)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map_or_else(|| "unknown".to_string(), |s| s.trim().to_string());

    let ci_run_id = std::env::var("GITHUB_RUN_ID")
        .or_else(|_| std::env::var("CI_RUN_ID"))
        .unwrap_or_else(|_| format!("local-{}", Utc::now().format("%Y%m%dT%H%M%SZ")));

    eprintln!("\n=== Unified CI Evidence Bundle (bd-1f42.6.8) ===");
    eprintln!("  Git ref:    {git_ref}");
    eprintln!("  CI run:     {ci_run_id}");
    eprintln!("  Bundle dir: {}", bundle_dir.display());
    eprintln!();

    // ── Collect all sections ──
    let mut sections: Vec<BundleSection> = Vec::new();

    for source in ARTIFACT_SOURCES {
        eprint!("  [{:.<40}] ", source.label);
        let section = collect_section(&root, source);
        match section.status.as_str() {
            "present" => eprintln!(
                "PRESENT  ({} file(s), {} bytes)",
                section.file_count, section.total_bytes
            ),
            "invalid" => eprintln!("INVALID  {}", section.diagnostics.as_deref().unwrap_or("")),
            _ => eprintln!("MISSING"),
        }
        sections.push(section);
    }

    let perf3x_lineage_section = build_perf3x_lineage_section(&sections);
    eprint!("  [{:.<40}] ", perf3x_lineage_section.label);
    match perf3x_lineage_section.status.as_str() {
        "present" => eprintln!("PRESENT"),
        "invalid" => eprintln!(
            "INVALID  {}",
            perf3x_lineage_section.diagnostics.as_deref().unwrap_or("")
        ),
        status => eprintln!("{status}"),
    }
    let lineage_failed = perf3x_lineage_section.status == "invalid";
    sections.push(perf3x_lineage_section);

    // ── Compute summary ──
    let present = sections.iter().filter(|s| s.status == "present").count();
    let missing = sections.iter().filter(|s| s.status == "missing").count();
    let invalid = sections.iter().filter(|s| s.status == "invalid").count();
    let total_artifacts: usize = sections.iter().map(|s| s.file_count).sum();
    let total_bytes: u64 = sections.iter().map(|s| s.total_bytes).sum();

    let required_present = ARTIFACT_SOURCES
        .iter()
        .zip(sections.iter())
        .filter(|(src, sec)| src.required && sec.status == "present")
        .count();
    let required_total = ARTIFACT_SOURCES.iter().filter(|s| s.required).count();

    let verdict = if lineage_failed {
        "insufficient"
    } else if required_present == required_total && invalid == 0 {
        "complete"
    } else if required_present > 0 {
        "partial"
    } else {
        "insufficient"
    };

    let bundle = EvidenceBundle {
        schema: "pi.ci.evidence_bundle.v1".to_string(),
        generated_at: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
        git_ref: git_ref.clone(),
        ci_run_id: ci_run_id.clone(),
        sections: sections.clone(),
        summary: BundleSummary {
            total_sections: sections.len(),
            present_sections: present,
            missing_sections: missing,
            invalid_sections: invalid,
            total_artifacts,
            total_bytes,
            verdict: verdict.to_string(),
        },
    };

    // ── Write index.json ──
    let index_path = bundle_dir.join("index.json");
    let _ = std::fs::write(
        &index_path,
        serde_json::to_string_pretty(&bundle).unwrap_or_default(),
    );

    // ── Write events.jsonl ──
    let events_path = bundle_dir.join("events.jsonl");
    let mut event_lines: Vec<String> = Vec::new();
    for section in &sections {
        let line = serde_json::json!({
            "schema": "pi.ci.evidence_bundle_event.v1",
            "section_id": section.id,
            "category": section.category,
            "status": section.status,
            "file_count": section.file_count,
            "total_bytes": section.total_bytes,
            "artifact_path": section.artifact_path,
            "ts": Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
        });
        event_lines.push(serde_json::to_string(&line).unwrap_or_default());
    }
    let _ = std::fs::write(&events_path, event_lines.join("\n") + "\n");

    // ── Write bundle_report.md ──
    let mut md = String::new();
    md.push_str("# Unified CI Evidence Bundle\n\n");
    let _ = writeln!(
        md,
        "> Generated: {}",
        Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
    );
    let _ = writeln!(md, "> Git ref: {git_ref}");
    let _ = writeln!(md, "> CI run: {ci_run_id}");
    let _ = writeln!(md, "> Verdict: **{}**\n", verdict.to_uppercase());

    md.push_str("## Summary\n\n");
    md.push_str("| Metric | Value |\n|--------|-------|\n");
    let _ = writeln!(md, "| Total sections | {} |", sections.len());
    let _ = writeln!(md, "| Present | {present} |");
    let _ = writeln!(md, "| Missing | {missing} |");
    let _ = writeln!(md, "| Invalid | {invalid} |");
    let _ = writeln!(md, "| Total artifacts | {total_artifacts} |");
    let _ = writeln!(md, "| Total size | {:.1} KB |", total_bytes as f64 / 1024.0);
    let _ = writeln!(
        md,
        "| Required present | {required_present}/{required_total} |"
    );
    md.push('\n');

    // Group by category.
    let categories: Vec<&str> = {
        let mut cats: Vec<&str> = sections.iter().map(|s| s.category.as_str()).collect();
        cats.dedup();
        cats
    };

    for cat in &categories {
        let cat_sections: Vec<&BundleSection> =
            sections.iter().filter(|s| s.category == *cat).collect();

        let _ = writeln!(md, "## {} ({})\n", capitalize(cat), cat_sections.len());
        md.push_str(
            "| Section | Status | Files | Size | Path |\n|---------|--------|-------|------|------|\n",
        );
        for s in &cat_sections {
            let status_icon = match s.status.as_str() {
                "present" => "PASS",
                "invalid" => "WARN",
                _ => "MISS",
            };
            let _ = writeln!(
                md,
                "| {} | {} | {} | {} B | `{}` |",
                s.label,
                status_icon,
                s.file_count,
                s.total_bytes,
                s.artifact_path.as_deref().unwrap_or("-"),
            );
        }
        md.push('\n');
    }

    // Failures section for quick navigation.
    let failures: Vec<&BundleSection> = sections
        .iter()
        .filter(|s| s.status == "missing" || s.status == "invalid")
        .collect();
    if !failures.is_empty() {
        md.push_str("## Missing / Invalid Sections\n\n");
        for s in &failures {
            let required_marker = if ARTIFACT_SOURCES
                .iter()
                .any(|src| src.id == s.id && src.required)
            {
                " **(REQUIRED)**"
            } else {
                ""
            };
            let _ = writeln!(
                md,
                "- **{}** ({}): {}{}\n  Path: `{}`",
                s.label,
                s.status,
                s.diagnostics.as_deref().unwrap_or(""),
                required_marker,
                s.artifact_path.as_deref().unwrap_or("-"),
            );
        }
        md.push('\n');
    }

    let md_path = bundle_dir.join("bundle_report.md");
    let _ = std::fs::write(&md_path, &md);

    // ── Print summary ──
    eprintln!("\n=== Evidence Bundle Summary ===");
    eprintln!("  Verdict:    {}", verdict.to_uppercase());
    eprintln!("  Sections:   {present}/{} present", sections.len());
    eprintln!("  Missing:    {missing}");
    eprintln!("  Invalid:    {invalid}");
    eprintln!("  Artifacts:  {total_artifacts} files");
    eprintln!("  Size:       {:.1} KB", total_bytes as f64 / 1024.0);
    eprintln!("  Required:   {required_present}/{required_total}");
    eprintln!();
    eprintln!("  Reports:");
    eprintln!("    Index: {}", index_path.display());
    eprintln!("    JSONL: {}", events_path.display());
    eprintln!("    MD:    {}", md_path.display());
    eprintln!();
}

/// Verify the evidence bundle index has the correct structure.
#[test]
fn evidence_bundle_index_schema() {
    let bundle_path = repo_root()
        .join("tests")
        .join("evidence_bundle")
        .join("index.json");

    // Bundle may not exist yet on first run; skip gracefully.
    let Some(val) = load_json(&bundle_path) else {
        eprintln!(
            "  SKIP: Bundle index not found at {}. Run build_evidence_bundle first.",
            bundle_path.display()
        );
        return;
    };

    // Validate schema.
    assert_eq!(
        val.get("schema").and_then(Value::as_str),
        Some("pi.ci.evidence_bundle.v1"),
        "Bundle index must have schema pi.ci.evidence_bundle.v1"
    );

    // Must have sections array.
    let sections = val
        .get("sections")
        .and_then(Value::as_array)
        .expect("Bundle must have sections array");
    assert!(
        !sections.is_empty(),
        "Bundle must have at least one section"
    );

    // Each section must have required fields.
    for section in sections {
        assert!(
            section.get("id").and_then(Value::as_str).is_some(),
            "Section missing id"
        );
        assert!(
            section.get("status").and_then(Value::as_str).is_some(),
            "Section missing status"
        );
        assert!(
            section.get("category").and_then(Value::as_str).is_some(),
            "Section missing category"
        );
    }

    // Must have summary.
    let summary = val.get("summary").expect("Bundle must have summary");
    assert!(
        summary.get("verdict").and_then(Value::as_str).is_some(),
        "Summary must have verdict"
    );
    assert!(
        summary.get("total_sections").is_some(),
        "Summary must have total_sections"
    );
}

/// Verify that every failing section in the bundle points to a precise path.
#[test]
fn evidence_bundle_failures_have_paths() {
    let bundle_path = repo_root()
        .join("tests")
        .join("evidence_bundle")
        .join("index.json");

    let Some(val) = load_json(&bundle_path) else {
        eprintln!("  SKIP: Bundle not found. Run build_evidence_bundle first.");
        return;
    };

    let sections = val
        .get("sections")
        .and_then(Value::as_array)
        .unwrap_or(&Vec::new())
        .clone();

    for section in &sections {
        let status = section.get("status").and_then(Value::as_str).unwrap_or("");
        if status == "missing" || status == "invalid" {
            let has_path = section
                .get("artifact_path")
                .and_then(Value::as_str)
                .is_some_and(|p| !p.is_empty());
            assert!(
                has_path,
                "Failing section {:?} must have artifact_path",
                section.get("id")
            );
        }
    }
}

#[test]
fn must_pass_gate_source_is_required_json_verdict_file() {
    let source = ARTIFACT_SOURCES
        .iter()
        .find(|source| source.id == "must_pass_gate")
        .expect("must_pass_gate source must exist");
    assert!(
        !source.is_directory,
        "must_pass_gate must target a JSON verdict artifact, not a directory"
    );
    assert!(
        source.path.ends_with("must_pass_gate_verdict.json"),
        "must_pass_gate path must target must_pass_gate_verdict.json"
    );
    assert!(
        source.required,
        "must_pass_gate should be required for complete evidence bundles"
    );
}

#[test]
fn perf_comparison_source_is_required_json_artifact() {
    let source = ARTIFACT_SOURCES
        .iter()
        .find(|source| source.id == "perf_comparison")
        .expect("perf_comparison source must exist");
    assert!(
        !source.is_directory,
        "perf_comparison source must target a JSON artifact"
    );
    assert!(
        source.path.ends_with("perf_comparison.json"),
        "perf_comparison source must point to perf_comparison.json"
    );
    assert!(
        source.required,
        "perf_comparison source should be required for PERF-3X evidence completeness"
    );
}

#[test]
fn parameter_sweeps_source_is_required_json_artifact() {
    let source = ARTIFACT_SOURCES
        .iter()
        .find(|source| source.id == "parameter_sweeps")
        .expect("parameter_sweeps source must exist");
    assert!(
        !source.is_directory,
        "parameter_sweeps source must target a JSON artifact"
    );
    assert!(
        source.path.ends_with("parameter_sweeps.json"),
        "parameter_sweeps source must point to parameter_sweeps.json"
    );
    assert!(
        source.required,
        "parameter_sweeps source should be required for PERF-3X evidence completeness"
    );
}

#[test]
fn full_cert_diagnostics_are_required_for_complete_verdict() {
    let health_delta = ARTIFACT_SOURCES
        .iter()
        .find(|source| source.id == "health_delta")
        .expect("health_delta source must exist");
    assert!(
        health_delta.required,
        "health_delta should be required for complete evidence bundles"
    );

    let journey_reports = ARTIFACT_SOURCES
        .iter()
        .find(|source| source.id == "journey_reports")
        .expect("journey_reports source must exist");
    assert!(
        journey_reports.required,
        "journey_reports should be required for complete evidence bundles"
    );
}

#[test]
fn validate_must_pass_gate_payload_accepts_current_shape() {
    let payload = serde_json::json!({
        "schema": "pi.ext.must_pass_gate.v1",
        "generated_at": "2026-02-17T03:00:00.000Z",
        "run_id": "ci-123",
        "correlation_id": "corr-123",
        "status": "pass",
        "observed": {
            "must_pass_total": 208,
            "must_pass_passed": 208,
            "must_pass_failed": 0,
            "must_pass_skipped": 0,
            "must_pass_pass_rate_pct": 100.0
        }
    });

    let summary = validate_must_pass_gate_payload(&payload)
        .expect("current must-pass payload shape should validate");
    assert_eq!(summary["status"], "pass");
    assert_eq!(summary["must_pass_total"], 208);
    assert_eq!(summary["must_pass_passed"], 208);
}

#[test]
fn validate_must_pass_gate_payload_rejects_missing_lineage() {
    let payload = serde_json::json!({
        "schema": "pi.ext.must_pass_gate.v1",
        "generated_at": "2026-02-17T03:00:00.000Z",
        "status": "pass",
        "observed": {
            "must_pass_total": 208,
            "must_pass_passed": 208
        }
    });

    let err = validate_must_pass_gate_payload(&payload)
        .expect_err("payload without run/correlation lineage must fail closed");
    assert!(
        err.contains("run_id"),
        "expected run_id validation error, got: {err}"
    );
}

#[test]
fn validate_perf_comparison_payload_accepts_current_shape() {
    let payload = serde_json::json!({
        "schema": "pi.ext.perf_comparison.v1",
        "generated_at": "2026-02-17T03:00:00.000Z",
        "summary": {
            "overall_verdict": "faster",
            "faster_count": 7,
            "slower_count": 1,
            "comparable_count": 2
        }
    });

    let summary = validate_perf_comparison_payload(&payload)
        .expect("current perf_comparison payload shape should validate");
    assert_eq!(summary["overall_verdict"], "faster");
    assert_eq!(summary["faster_count"], 7);
    assert_eq!(summary["slower_count"], 1);
    assert_eq!(summary["comparable_count"], 2);
}

#[test]
fn validate_perf_comparison_payload_rejects_missing_overall_verdict() {
    let payload = serde_json::json!({
        "schema": "pi.ext.perf_comparison.v1",
        "generated_at": "2026-02-17T03:00:00.000Z",
        "summary": {
            "faster_count": 7,
            "slower_count": 1,
            "comparable_count": 2
        }
    });

    let err = validate_perf_comparison_payload(&payload)
        .expect_err("perf_comparison without overall_verdict should fail closed");
    assert!(
        err.contains("overall_verdict"),
        "expected overall_verdict validation error, got: {err}"
    );
}

#[test]
fn validate_parameter_sweeps_payload_accepts_current_shape() {
    let payload = serde_json::json!({
        "schema": "pi.perf.parameter_sweeps.v1",
        "generated_at": "2026-02-17T03:00:00.000Z",
        "readiness": {
            "status": "ready",
            "ready_for_phase5": true,
            "blocking_reasons": []
        },
        "source_identity": {
            "source_artifact": "tests/perf/runs/results/phase1_matrix_validation.json"
        }
    });

    let summary = validate_parameter_sweeps_payload(&payload)
        .expect("current parameter_sweeps payload shape should validate");
    assert_eq!(summary["readiness_status"], "ready");
    assert_eq!(summary["ready_for_phase5"], true);
    assert_eq!(summary["blocking_reasons_count"], 0);
}

#[test]
fn validate_parameter_sweeps_payload_rejects_unknown_readiness_status() {
    let payload = serde_json::json!({
        "schema": "pi.perf.parameter_sweeps.v1",
        "generated_at": "2026-02-17T03:00:00.000Z",
        "readiness": {
            "status": "unknown",
            "ready_for_phase5": false,
            "blocking_reasons": ["lineage_missing"]
        },
        "source_identity": {
            "source_artifact": "tests/perf/runs/results/phase1_matrix_validation.json"
        }
    });

    let err = validate_parameter_sweeps_payload(&payload)
        .expect_err("parameter_sweeps with non-contract readiness status must fail closed");
    assert!(
        err.contains("ready|blocked"),
        "expected readiness status validation error, got: {err}"
    );
}

fn lineage_fixture_section(id: &str, summary: Value) -> BundleSection {
    BundleSection {
        id: id.to_string(),
        label: id.to_string(),
        category: "performance".to_string(),
        status: "present".to_string(),
        artifact_path: Some(format!("{id}.json")),
        schema: None,
        summary: Some(summary),
        diagnostics: None,
        file_count: 1,
        total_bytes: 1,
    }
}

fn unique_temp_root(label: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    std::env::temp_dir().join(format!(
        "ci-evidence-bundle-{label}-{}-{nanos}",
        std::process::id()
    ))
}

fn write_fixture_json(path: &Path, payload: &Value) {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let text = serde_json::to_string_pretty(payload).expect("fixture JSON should serialize");
    std::fs::write(path, text).expect("fixture JSON should write");
}

#[test]
fn stress_triage_source_is_required_json_artifact() {
    let source = ARTIFACT_SOURCES
        .iter()
        .find(|source| source.id == "stress_triage")
        .expect("stress_triage source must exist");
    assert!(
        !source.is_directory,
        "stress_triage source must target a JSON artifact"
    );
    assert!(
        source.path.ends_with("stress_triage.json"),
        "stress_triage source must point to stress_triage.json"
    );
    assert!(
        source.required,
        "stress_triage source should be required for PERF-3X lineage contract"
    );
}

#[test]
fn validate_perf3x_lineage_contract_accepts_coherent_generated_at_fields() {
    let sections = vec![
        lineage_fixture_section(
            "must_pass_gate",
            serde_json::json!({
                "run_id": "local-20260217T030608928Z",
                "correlation_id": "corr-local-20260217T030608928Z",
                "generated_at": "2026-02-17T03:06:08.928Z"
            }),
        ),
        lineage_fixture_section(
            "conformance_summary",
            serde_json::json!({
                "generated_at": "2026-02-16T20:45:35Z"
            }),
        ),
        lineage_fixture_section(
            "stress_triage",
            serde_json::json!({
                "generated_at": "2026-02-06T01:29:10Z"
            }),
        ),
    ];

    let summary = validate_perf3x_lineage_contract(&sections)
        .expect("coherent lineage metadata should pass contract validation");
    assert_eq!(summary["run_id"], "local-20260217T030608928Z");
}

#[test]
fn validate_perf3x_lineage_contract_rejects_excessive_artifact_span() {
    let sections = vec![
        lineage_fixture_section(
            "must_pass_gate",
            serde_json::json!({
                "run_id": "run-123",
                "correlation_id": "corr-run-123",
                "generated_at": "2026-02-17T03:06:08.928Z"
            }),
        ),
        lineage_fixture_section(
            "conformance_summary",
            serde_json::json!({
                "generated_at": "2026-02-16T20:45:35Z"
            }),
        ),
        lineage_fixture_section(
            "stress_triage",
            serde_json::json!({
                "generated_at": "2026-01-01T00:00:00Z"
            }),
        ),
    ];

    let err = validate_perf3x_lineage_contract(&sections)
        .expect_err("lineage span beyond threshold must fail closed");
    assert!(
        err.contains("span exceeds"),
        "expected span-threshold failure detail, got: {err}"
    );
}

#[test]
fn validate_perf3x_lineage_contract_rejects_missing_generated_at() {
    let sections = vec![
        lineage_fixture_section(
            "must_pass_gate",
            serde_json::json!({
                "run_id": "run-123",
                "correlation_id": "corr-run-123",
                "generated_at": "2026-02-17T03:06:08.928Z"
            }),
        ),
        lineage_fixture_section("conformance_summary", serde_json::json!({})),
        lineage_fixture_section(
            "stress_triage",
            serde_json::json!({
                "generated_at": "2026-02-06T01:29:10Z"
            }),
        ),
    ];

    let err = validate_perf3x_lineage_contract(&sections)
        .expect_err("missing generated_at metadata must fail closed");
    assert!(
        err.contains("generated_at"),
        "expected generated_at validation detail, got: {err}"
    );
}

#[test]
fn collect_section_reports_missing_file_path_diagnostics() {
    let root = unique_temp_root("missing-file");
    let _ = std::fs::create_dir_all(&root);
    let source = ArtifactSource {
        id: "missing_file",
        label: "Missing file",
        category: "unit",
        path: "does/not/exist.json",
        expected_schema: Some("pi.test"),
        is_directory: false,
        required: false,
    };

    let section = collect_section(&root, &source);
    assert_eq!(section.status, "missing");
    assert_eq!(section.file_count, 0);
    assert_eq!(section.total_bytes, 0);
    assert_eq!(
        section.artifact_path.as_deref(),
        Some("does/not/exist.json")
    );
    assert_eq!(section.diagnostics.as_deref(), Some("File not found"));
}

#[test]
fn collect_section_reports_missing_directory_path_diagnostics() {
    let root = unique_temp_root("missing-directory");
    let _ = std::fs::create_dir_all(&root);
    let source = ArtifactSource {
        id: "missing_dir",
        label: "Missing dir",
        category: "unit",
        path: "does/not/exist",
        expected_schema: None,
        is_directory: true,
        required: false,
    };

    let section = collect_section(&root, &source);
    assert_eq!(section.status, "missing");
    assert_eq!(section.file_count, 0);
    assert_eq!(section.total_bytes, 0);
    assert_eq!(section.artifact_path.as_deref(), Some("does/not/exist"));
    assert_eq!(section.diagnostics.as_deref(), Some("Directory not found"));
}

#[test]
fn collect_section_parameter_sweeps_reports_custom_missing_diagnostic() {
    let root = unique_temp_root("parameter-sweeps-missing");
    let _ = std::fs::create_dir_all(&root);
    let source = ARTIFACT_SOURCES
        .iter()
        .find(|source| source.id == "parameter_sweeps")
        .expect("parameter_sweeps source must exist");

    let section = collect_section(&root, source);
    assert_eq!(section.status, "missing");
    assert_eq!(section.artifact_path.as_deref(), Some(source.path));
    assert_eq!(
        section.diagnostics.as_deref(),
        Some(PARAMETER_SWEEPS_MISSING_DIAGNOSTIC)
    );
}

#[test]
fn collect_section_parameter_sweeps_uses_discovered_artifact_path() {
    let root = unique_temp_root("parameter-sweeps-discovery");
    let _ = std::fs::create_dir_all(&root);
    let source = ARTIFACT_SOURCES
        .iter()
        .find(|source| source.id == "parameter_sweeps")
        .expect("parameter_sweeps source must exist");
    let discovered_path = root.join("tests/e2e_results/run-42/results/parameter_sweeps.json");
    write_fixture_json(
        &discovered_path,
        &serde_json::json!({
            "schema": "pi.perf.parameter_sweeps.v1",
            "generated_at": "2026-02-17T04:00:00.000Z",
            "readiness": {
                "status": "blocked",
                "ready_for_phase5": false,
                "blocking_reasons": ["need_additional_runs"]
            },
            "source_identity": {
                "source_artifact": "tests/perf/runs/results/phase1_matrix_validation.json"
            }
        }),
    );

    let section = collect_section(&root, source);
    assert_eq!(section.status, "present");
    assert_eq!(
        section.artifact_path.as_deref(),
        Some("tests/e2e_results/run-42/results/parameter_sweeps.json")
    );
    assert_eq!(section.file_count, 1);
    assert!(
        section.total_bytes > 0,
        "parameter_sweeps section should include file size for discovered artifact"
    );
    let summary = section
        .summary
        .as_ref()
        .expect("parameter_sweeps section should include summary payload");
    assert_eq!(
        summary.get("readiness_status").and_then(Value::as_str),
        Some("blocked")
    );
    assert_eq!(
        summary.get("ready_for_phase5").and_then(Value::as_bool),
        Some(false)
    );
    assert_eq!(
        summary
            .get("blocking_reasons_count")
            .and_then(Value::as_u64),
        Some(1)
    );
}

/// Capitalize the first letter of a string.
fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    c.next()
        .map_or_else(String::new, |f| f.to_uppercase().to_string() + c.as_str())
}
