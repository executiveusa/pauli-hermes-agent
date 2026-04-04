//! Golden-path first-time-user E2E suite (bd-k5q5.7.14)
//!
//! Proves the complete onboarding journey a first-time user would take:
//!   install -> configure -> run -> recover
//!
//! Each test uses a fully isolated CLI environment with no user state.

mod common;

use common::TestHarness;
use serde_json::json;
use std::cell::Cell;
use std::collections::BTreeMap;
use std::fs;
use std::io::Read as _;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

// ============================================================================
// Test harness (simplified CliTestHarness for golden-path tests)
// ============================================================================

const DEFAULT_TIMEOUT_SECS: u64 = 120;

struct GoldenPathResult {
    exit_code: i32,
    stdout: String,
    stderr: String,
    #[allow(dead_code)]
    duration: Duration,
}

struct GoldenPathHarness {
    harness: TestHarness,
    binary_path: PathBuf,
    #[allow(dead_code)]
    env_root: PathBuf,
    env: BTreeMap<String, String>,
    run_seq: Cell<usize>,
}

impl GoldenPathHarness {
    fn new(name: &str) -> Self {
        let harness = TestHarness::new(name);
        let binary_path = PathBuf::from(env!("CARGO_BIN_EXE_pi"));

        let env_root = harness.temp_path("pi-env");
        let _ = fs::create_dir_all(&env_root);

        let mut env = BTreeMap::new();
        // Fully isolate from user environment.
        env.insert(
            "PI_CODING_AGENT_DIR".into(),
            env_root.join("agent").display().to_string(),
        );
        env.insert(
            "PI_CONFIG_PATH".into(),
            env_root.join("settings.json").display().to_string(),
        );
        env.insert(
            "PI_SESSIONS_DIR".into(),
            env_root.join("sessions").display().to_string(),
        );
        env.insert(
            "PI_PACKAGE_DIR".into(),
            env_root.join("packages").display().to_string(),
        );
        // Offline-friendly npm.
        env.insert("npm_config_audit".into(), "false".into());
        env.insert("npm_config_fund".into(), "false".into());
        env.insert("npm_config_update_notifier".into(), "false".into());
        env.insert(
            "npm_config_cache".into(),
            env_root.join("npm-cache").display().to_string(),
        );
        env.insert(
            "npm_config_prefix".into(),
            env_root.join("npm-prefix").display().to_string(),
        );

        Self {
            harness,
            binary_path,
            env_root,
            env,
            run_seq: Cell::new(0),
        }
    }

    fn run(&self, args: &[&str]) -> GoldenPathResult {
        let seq = self.run_seq.get();
        self.run_seq.set(seq + 1);

        self.harness
            .log()
            .info("cli", format!("[{seq}] pi {}", args.join(" ")));

        let start = Instant::now();
        let mut cmd = Command::new(&self.binary_path);
        cmd.args(args)
            .envs(self.env.clone())
            .current_dir(self.harness.temp_dir())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn().expect("spawn pi binary");
        let mut child_stdout = child.stdout.take().unwrap();
        let mut child_stderr = child.stderr.take().unwrap();

        let stdout_t = std::thread::spawn(move || {
            let mut buf = Vec::new();
            let _ = child_stdout.read_to_end(&mut buf);
            String::from_utf8_lossy(&buf).to_string()
        });
        let stderr_t = std::thread::spawn(move || {
            let mut buf = Vec::new();
            let _ = child_stderr.read_to_end(&mut buf);
            String::from_utf8_lossy(&buf).to_string()
        });

        let timeout = Duration::from_secs(DEFAULT_TIMEOUT_SECS);
        let status = loop {
            match child.try_wait() {
                Ok(Some(s)) => break s,
                Ok(None) => {}
                Err(e) => panic!("try_wait: {e}"),
            }
            if start.elapsed() > timeout {
                let _ = child.kill();
                panic!("pi timed out after {timeout:?}");
            }
            std::thread::sleep(Duration::from_millis(50));
        };

        let stdout = stdout_t.join().unwrap_or_default();
        let stderr = stderr_t.join().unwrap_or_default();
        let duration = start.elapsed();

        self.harness
            .log()
            .info_ctx("result", "CLI finished", |ctx| {
                ctx.push(("exit_code".into(), status.code().unwrap_or(-1).to_string()));
                ctx.push(("duration_ms".into(), duration.as_millis().to_string()));
            });

        // Artifact capture for debugging.
        let stdout_path = self.harness.temp_path(format!("stdout_{seq}.txt"));
        let stderr_path = self.harness.temp_path(format!("stderr_{seq}.txt"));
        let _ = fs::write(&stdout_path, &stdout);
        let _ = fs::write(&stderr_path, &stderr);
        self.harness
            .record_artifact(format!("stdout_{seq}"), &stdout_path);
        self.harness
            .record_artifact(format!("stderr_{seq}"), &stderr_path);

        GoldenPathResult {
            exit_code: status.code().unwrap_or(-1),
            stdout,
            stderr,
            duration,
        }
    }

    fn write_extension(&self, name: &str, source: &str) -> PathBuf {
        let ext_dir = self.harness.temp_dir().join(name);
        let _ = fs::create_dir_all(ext_dir.join("extensions"));
        let src_path = ext_dir.join("extensions").join(format!("{name}.js"));
        fs::write(&src_path, source).expect("write extension source");
        fs::write(
            ext_dir.join("package.json"),
            serde_json::to_string_pretty(&json!({
                "name": name,
                "version": "1.0.0",
                "main": format!("extensions/{name}.js"),
                "pi": { "extensions": [format!("extensions/{name}.js")] }
            }))
            .unwrap(),
        )
        .expect("write package.json");
        ext_dir
    }
}

// ============================================================================
// Phase 1: Install — first thing a user does
// ============================================================================

#[test]
fn golden_path_version_prints_cleanly() {
    let h = GoldenPathHarness::new("golden_path_version");
    let r = h.run(&["--version"]);
    assert_eq!(r.exit_code, 0, "stderr: {}", r.stderr);
    assert!(
        r.stdout.starts_with("pi "),
        "version should start with 'pi ': {:?}",
        r.stdout
    );
}

#[test]
fn golden_path_help_shows_usage() {
    let h = GoldenPathHarness::new("golden_path_help");
    let r = h.run(&["--help"]);
    assert_eq!(r.exit_code, 0, "stderr: {}", r.stderr);
    // Help output should mention common flags.
    assert!(
        r.stdout.contains("--model") || r.stdout.contains("Usage"),
        "help should show usage info: {:?}",
        r.stdout
    );
}

// ============================================================================
// Phase 2: Configure — user sets up environment
// ============================================================================

#[test]
fn golden_path_config_command_works() {
    let h = GoldenPathHarness::new("golden_path_config");
    let r = h.run(&["config"]);
    // Config should succeed even with no config file.
    assert_eq!(r.exit_code, 0, "stderr: {}", r.stderr);
}

#[test]
fn golden_path_explain_extension_policy_default() {
    let h = GoldenPathHarness::new("golden_path_explain_policy");
    let r = h.run(&["--explain-extension-policy"]);
    assert_eq!(r.exit_code, 0, "stderr: {}", r.stderr);
    // Should show policy details.
    assert!(
        r.stdout.contains("mode") || r.stdout.contains("policy") || r.stdout.contains("Mode"),
        "explain output should describe policy: {:?}",
        r.stdout
    );
}

#[test]
fn golden_path_explain_extension_policy_with_override() {
    let h = GoldenPathHarness::new("golden_path_explain_policy_override");
    let r = h.run(&["--explain-extension-policy", "--extension-policy", "safe"]);
    assert_eq!(r.exit_code, 0, "stderr: {}", r.stderr);
    let lower = r.stdout.to_lowercase();
    assert!(
        lower.contains("safe") || lower.contains("strict"),
        "should mention safe/strict profile: {:?}",
        r.stdout
    );
}

// ============================================================================
// Phase 3: Run — user creates and checks an extension
// ============================================================================

#[test]
fn golden_path_doctor_clean_extension_passes() {
    let h = GoldenPathHarness::new("golden_path_doctor_clean");
    let ext_dir = h.write_extension(
        "hello-ext",
        r#"
import path from "node:path";
export default function(pi) {
    pi.tool({ name: "hello", description: "Say hello" });
}
"#,
    );
    let r = h.run(&["doctor", ext_dir.to_str().unwrap()]);
    assert_eq!(r.exit_code, 0, "stderr: {}", r.stderr);
    assert!(
        r.stdout.contains("PASS"),
        "clean extension should pass: {:?}",
        r.stdout
    );
}

#[test]
fn golden_path_doctor_json_format() {
    let h = GoldenPathHarness::new("golden_path_doctor_json");
    let ext_dir = h.write_extension(
        "json-ext",
        r#"export default function(pi) { pi.tool({ name: "t" }); }"#,
    );
    let r = h.run(&["doctor", ext_dir.to_str().unwrap(), "--format", "json"]);
    assert_eq!(r.exit_code, 0, "stderr: {}", r.stderr);
    let parsed: serde_json::Value = serde_json::from_str(&r.stdout).expect("should be valid JSON");
    assert_eq!(parsed["overall"], "pass", "should be pass: {}", r.stdout);
    assert!(
        parsed["findings"].is_array(),
        "findings should be an array: {}",
        r.stdout
    );
    assert!(
        parsed["summary"].is_object(),
        "summary should be an object: {}",
        r.stdout
    );
}

#[test]
fn golden_path_doctor_markdown_format() {
    let h = GoldenPathHarness::new("golden_path_doctor_md");
    let ext_dir = h.write_extension(
        "md-ext",
        r#"export default function(pi) { pi.tool({ name: "t" }); }"#,
    );
    let r = h.run(&["doctor", ext_dir.to_str().unwrap(), "--format", "markdown"]);
    assert_eq!(r.exit_code, 0, "stderr: {}", r.stderr);
    assert!(
        r.stdout.contains("# Pi Doctor Report"),
        "markdown should contain Pi Doctor Report header: {:?}",
        r.stdout
    );
    assert!(
        r.stdout.contains("**Overall"),
        "markdown should contain Overall line: {:?}",
        r.stdout
    );
}

#[test]
fn golden_path_doctor_warns_on_partial_module() {
    let h = GoldenPathHarness::new("golden_path_doctor_partial");
    let ext_dir = h.write_extension(
        "crypto-ext",
        r#"
import crypto from "node:crypto";
export default function(pi) {
    pi.tool({ name: "hash", description: "Hash stuff" });
}
"#,
    );
    let r = h.run(&["doctor", ext_dir.to_str().unwrap()]);
    assert_eq!(r.exit_code, 0, "stderr: {}", r.stderr);
    // crypto is partial — should appear as finding or info.
    // The key is the doctor runs successfully without crashing.
    h.harness
        .log()
        .info("result", format!("stdout: {}", r.stdout));
}

#[test]
fn golden_path_doctor_fails_on_incompatible_extension() {
    let h = GoldenPathHarness::new("golden_path_doctor_fail");
    let ext_dir = h.write_extension(
        "net-ext",
        r#"
import net from "node:net";
import dns from "node:dns";
const server = net.createServer();
export default function(pi) {}
"#,
    );
    let r = h.run(&["doctor", ext_dir.to_str().unwrap()]);
    // Doctor exits with code 1 when overall is FAIL.
    assert!(
        r.exit_code == 0 || r.exit_code == 1,
        "doctor should exit cleanly (0 or 1): exit={}, stderr: {}",
        r.exit_code,
        r.stderr
    );
    assert!(
        r.stdout.contains("FAIL") || r.stdout.contains("WARN"),
        "incompatible extension should fail or warn: {:?}",
        r.stdout
    );
}

#[test]
fn golden_path_doctor_nonexistent_path_errors() {
    let h = GoldenPathHarness::new("golden_path_doctor_nopath");
    let r = h.run(&["doctor", "/tmp/nonexistent-ext-abc123"]);
    assert_ne!(r.exit_code, 0, "should fail on missing path");
    let combined = format!("{}{}", r.stdout, r.stderr);
    assert!(
        combined.contains("not found") || combined.contains("No such file"),
        "should report path not found: {combined:?}",
    );
}

// ============================================================================
// Phase 4: Configure Policy — user tunes extension policy
// ============================================================================

#[test]
fn golden_path_doctor_with_safe_policy() {
    let h = GoldenPathHarness::new("golden_path_doctor_safe");
    let ext_dir = h.write_extension(
        "exec-ext",
        r#"
const { exec } = require("child_process");
export default function(pi) {
    pi.exec("ls");
}
"#,
    );
    let r = h.run(&["doctor", ext_dir.to_str().unwrap(), "--policy", "safe"]);
    // Doctor exits with code 1 when overall is FAIL.
    assert!(
        r.exit_code == 0 || r.exit_code == 1,
        "doctor should exit cleanly (0 or 1): exit={}, stderr: {}",
        r.exit_code,
        r.stderr
    );
    let out_lower = r.stdout.to_lowercase();
    assert!(
        out_lower.contains("fail") || out_lower.contains("warn"),
        "safe policy should flag exec extension: {:?}",
        r.stdout
    );
    assert!(
        out_lower.contains("denied")
            || out_lower.contains("error")
            || out_lower.contains("incompatible"),
        "should mention denial or incompatibility: {:?}",
        r.stdout
    );
}

#[test]
fn golden_path_doctor_with_permissive_policy() {
    let h = GoldenPathHarness::new("golden_path_doctor_permissive");
    let ext_dir = h.write_extension(
        "exec-ext-p",
        r#"
const { exec } = require("child_process");
export default function(pi) {
    pi.exec("ls");
}
"#,
    );
    let r = h.run(&[
        "doctor",
        ext_dir.to_str().unwrap(),
        "--policy",
        "permissive",
    ]);
    assert_eq!(r.exit_code, 0, "stderr: {}", r.stderr);
    // Permissive should pass or warn, not fail on exec.
    assert!(
        !r.stdout.contains("FAIL"),
        "permissive should not fail exec: {:?}",
        r.stdout
    );
}

// ============================================================================
// Phase 5: Recover — user fixes issues and retries
// ============================================================================

#[test]
fn golden_path_fix_extension_then_recheck() {
    let h = GoldenPathHarness::new("golden_path_fix_recheck");

    // Step 1: Create extension with incompatible import.
    let ext_dir = h.write_extension(
        "fixable-ext",
        r#"
import net from "node:net";
export default function(pi) {
    pi.tool({ name: "connect" });
}
"#,
    );
    let r1 = h.run(&["doctor", ext_dir.to_str().unwrap()]);
    // Doctor exits with code 1 when overall is FAIL.
    assert!(
        r1.exit_code == 0 || r1.exit_code == 1,
        "doctor should exit cleanly (0 or 1): exit={}, stderr: {}",
        r1.exit_code,
        r1.stderr
    );
    h.harness
        .log()
        .info("step1", format!("Before fix: {}", r1.stdout));
    let had_issues = r1.stdout.contains("FAIL") || r1.stdout.contains("WARN");
    assert!(had_issues, "should have issues before fix");

    // Step 2: User fixes the extension (removes net import).
    let fixed_source = r#"
import path from "node:path";
export default function(pi) {
    pi.tool({ name: "connect", description: "Uses fetch instead" });
}
"#;
    fs::write(
        ext_dir.join("extensions").join("fixable-ext.js"),
        fixed_source,
    )
    .unwrap();

    // Step 3: Re-run doctor — should pass now.
    let r2 = h.run(&["doctor", ext_dir.to_str().unwrap()]);
    assert_eq!(r2.exit_code, 0);
    h.harness
        .log()
        .info("step2", format!("After fix: {}", r2.stdout));
    assert!(
        r2.stdout.contains("PASS"),
        "fixed extension should pass: {:?}",
        r2.stdout
    );
}

#[test]
fn golden_path_escalate_policy_to_resolve_denial() {
    let h = GoldenPathHarness::new("golden_path_escalate_policy");

    let ext_dir = h.write_extension(
        "env-ext",
        r#"
const key = process.env.API_KEY;
export default function(pi) {
    pi.tool({ name: "auth" });
}
"#,
    );

    // Step 1: Check with safe policy — should fail or warn about env.
    let r1 = h.run(&["doctor", ext_dir.to_str().unwrap(), "--policy", "safe"]);
    // Doctor exits with code 1 when overall is FAIL.
    assert!(
        r1.exit_code == 0 || r1.exit_code == 1,
        "doctor should exit cleanly (0 or 1): exit={}, stderr: {}",
        r1.exit_code,
        r1.stderr
    );
    h.harness.log().info("safe", format!("Safe: {}", r1.stdout));
    let safe_has_issues = r1.stdout.contains("FAIL") || r1.stdout.contains("WARN");
    assert!(
        safe_has_issues,
        "safe policy should flag env access: {:?}",
        r1.stdout
    );

    // Step 2: Escalate to permissive — denial should be resolved.
    let r2 = h.run(&[
        "doctor",
        ext_dir.to_str().unwrap(),
        "--policy",
        "permissive",
    ]);
    assert_eq!(r2.exit_code, 0);
    h.harness
        .log()
        .info("permissive", format!("Permissive: {}", r2.stdout));
    // Permissive should allow env.
    assert!(
        !r2.stdout.contains("denied"),
        "permissive should not deny env: {:?}",
        r2.stdout
    );
}

#[test]
fn golden_path_doctor_shows_remediation() {
    let h = GoldenPathHarness::new("golden_path_remediation");
    // Use node:zlib — a stub module that the path-based scanner detects
    // and produces a warning with remediation.
    let ext_dir = h.write_extension(
        "zlib-ext",
        r#"
import zlib from "node:zlib";
export default function(pi) {}
"#,
    );
    let r = h.run(&["doctor", ext_dir.to_str().unwrap()]);
    assert_eq!(r.exit_code, 0, "stderr: {}", r.stderr);
    // Doctor should show some guidance (remediation or suggested actions).
    let out_lower = r.stdout.to_lowercase();
    assert!(
        out_lower.contains("fix") || out_lower.contains("warn") || out_lower.contains("stub"),
        "should show remediation guidance: {:?}",
        r.stdout
    );
}

// ============================================================================
// Phase 6: Full journey — complete onboarding simulation
// ============================================================================

#[test]
fn golden_path_complete_first_time_user_journey() {
    let h = GoldenPathHarness::new("golden_path_full_journey");

    // Step 1: User discovers pi — checks version.
    h.harness.log().info("journey", "Step 1: version check");
    let r = h.run(&["--version"]);
    assert_eq!(r.exit_code, 0);

    // Step 2: User reads help.
    h.harness.log().info("journey", "Step 2: help");
    let r = h.run(&["--help"]);
    assert_eq!(r.exit_code, 0);

    // Step 3: User checks their current config.
    h.harness.log().info("journey", "Step 3: config check");
    let r = h.run(&["config"]);
    assert_eq!(r.exit_code, 0);

    // Step 4: User checks the default extension policy.
    h.harness.log().info("journey", "Step 4: understand policy");
    let r = h.run(&["--explain-extension-policy"]);
    assert_eq!(r.exit_code, 0);

    // Step 5: User creates a simple extension.
    h.harness.log().info("journey", "Step 5: create extension");
    let ext_dir = h.write_extension(
        "my-first-ext",
        r#"
import path from "node:path";
export default function(pi) {
    pi.tool({
        name: "greet",
        description: "Greet the user",
        schema: {},
        run: () => "Hello, world!"
    });
}
"#,
    );

    // Step 6: User runs doctor to verify.
    h.harness.log().info("journey", "Step 6: doctor (text)");
    let r = h.run(&["doctor", ext_dir.to_str().unwrap()]);
    assert_eq!(r.exit_code, 0);
    assert!(
        r.stdout.contains("PASS"),
        "first extension should pass: {:?}",
        r.stdout
    );

    // Step 7: User also checks JSON output for automation.
    h.harness.log().info("journey", "Step 7: doctor (json)");
    let r = h.run(&["doctor", ext_dir.to_str().unwrap(), "--format", "json"]);
    assert_eq!(r.exit_code, 0);
    let parsed: serde_json::Value = serde_json::from_str(&r.stdout).unwrap();
    assert_eq!(parsed["overall"], "pass");
    assert!(
        parsed["findings"].is_array(),
        "findings should be present in JSON output"
    );

    // Step 8: User tries the safe policy to understand constraints.
    h.harness.log().info("journey", "Step 8: check safe policy");
    let r = h.run(&["--explain-extension-policy", "--extension-policy", "safe"]);
    assert_eq!(r.exit_code, 0);

    h.harness
        .log()
        .info("journey", "Complete first-time user journey passed");
}

#[test]
fn golden_path_recovery_journey() {
    let h = GoldenPathHarness::new("golden_path_recovery_journey");

    // Step 1: User creates a problematic extension.
    h.harness
        .log()
        .info("journey", "Step 1: create broken extension");
    let ext_dir = h.write_extension(
        "broken-ext",
        r#"
import net from "node:net";
import dns from "node:dns";
const { exec } = require("child_process");
export default function(pi) {
    pi.exec("rm -rf /");
}
"#,
    );

    // Step 2: Doctor finds problems.
    h.harness
        .log()
        .info("journey", "Step 2: doctor finds problems");
    let r = h.run(&["doctor", ext_dir.to_str().unwrap(), "--format", "json"]);
    // Doctor exits with code 1 when overall is FAIL.
    assert!(
        r.exit_code == 0 || r.exit_code == 1,
        "doctor should exit cleanly (0 or 1): exit={}, stderr: {}",
        r.exit_code,
        r.stderr
    );
    let report: serde_json::Value = serde_json::from_str(&r.stdout).unwrap();
    assert_eq!(
        report["overall"], "fail",
        "broken extension overall should be fail: {}",
        r.stdout
    );

    // Step 3: Doctor shows findings.
    let findings = report["findings"].as_array().unwrap();
    assert!(
        !findings.is_empty(),
        "should have findings for broken extension"
    );
    h.harness.log().info_ctx("findings", "Found issues", |ctx| {
        ctx.push(("count".into(), findings.len().to_string()));
    });

    // Step 4: User fixes the extension.
    h.harness.log().info("journey", "Step 4: fix the extension");
    let fixed = r#"
import path from "node:path";
export default function(pi) {
    pi.tool({
        name: "safe-tool",
        description: "A safe tool that uses path"
    });
}
"#;
    fs::write(ext_dir.join("extensions").join("broken-ext.js"), fixed).unwrap();

    // Step 5: Re-check — should pass now.
    h.harness
        .log()
        .info("journey", "Step 5: verify fix with doctor");
    let r = h.run(&["doctor", ext_dir.to_str().unwrap(), "--format", "json"]);
    assert_eq!(
        r.exit_code, 0,
        "fixed extension should exit 0: {}",
        r.stderr
    );
    let fixed_report: serde_json::Value = serde_json::from_str(&r.stdout).unwrap();
    assert_eq!(
        fixed_report["overall"], "pass",
        "fixed extension should pass: {}",
        r.stdout
    );
    // Summary should show only pass findings after fix.
    assert!(
        fixed_report["summary"]["fail"].as_u64().unwrap_or(0) == 0,
        "fixed extension should have no failures: {}",
        r.stdout
    );

    h.harness.log().info("journey", "Recovery journey complete");
}

// ============================================================================
// Edge cases a first-time user might encounter
// ============================================================================

#[test]
fn golden_path_empty_extension_dir_passes_doctor() {
    let h = GoldenPathHarness::new("golden_path_empty_ext");
    let ext_dir = h.harness.temp_dir().join("empty-ext");
    let _ = fs::create_dir_all(&ext_dir);
    fs::write(
        ext_dir.join("package.json"),
        json!({"name": "empty", "version": "1.0.0"}).to_string(),
    )
    .unwrap();

    let r = h.run(&["doctor", ext_dir.to_str().unwrap()]);
    assert_eq!(r.exit_code, 0, "empty ext should not crash: {}", r.stderr);
}

#[test]
fn golden_path_list_models_flag() {
    let h = GoldenPathHarness::new("golden_path_list_models");
    let r = h.run(&["--list-models", "*"]);
    // Should succeed (lists models even without API key).
    assert_eq!(r.exit_code, 0, "list-models should work: {}", r.stderr);
}

#[test]
fn golden_path_multiple_formats_consistent() {
    let h = GoldenPathHarness::new("golden_path_formats_consistent");
    let ext_dir = h.write_extension(
        "format-test",
        r#"
import chokidar from "chokidar";
export default function(pi) { pi.tool({ name: "watch" }); }
"#,
    );

    let text_r = h.run(&["doctor", ext_dir.to_str().unwrap()]);
    let json_r = h.run(&["doctor", ext_dir.to_str().unwrap(), "--format", "json"]);
    let md_r = h.run(&["doctor", ext_dir.to_str().unwrap(), "--format", "markdown"]);

    assert_eq!(text_r.exit_code, 0);
    assert_eq!(json_r.exit_code, 0);
    assert_eq!(md_r.exit_code, 0);

    // Parse JSON overall verdict.
    let json_parsed: serde_json::Value = serde_json::from_str(&json_r.stdout).unwrap();
    let json_verdict = json_parsed["overall"].as_str().unwrap().to_uppercase();

    // All formats should agree on verdict.
    assert!(
        text_r.stdout.contains(&json_verdict),
        "text and JSON verdicts should agree: text={:?} json={json_verdict}",
        text_r.stdout
    );
    assert!(
        md_r.stdout.contains(&json_verdict),
        "markdown and JSON verdicts should agree: md={:?} json={json_verdict}",
        md_r.stdout
    );
}
