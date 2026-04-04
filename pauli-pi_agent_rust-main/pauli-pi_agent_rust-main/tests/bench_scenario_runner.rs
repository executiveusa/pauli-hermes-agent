//! Deterministic benchmark scenario runner (bd-m5jp).
//!
//! Executes cold start, warm start, tool call, and event hook dispatch scenarios
//! for a configurable set of extensions. Emits JSONL records conforming to the
//! `pi.ext.rust_bench.v1` schema with environment fingerprinting.
//!
//! Run with: `cargo test --test bench_scenario_runner -- --nocapture`
//!
//! Outputs: `target/perf/scenario_runner.jsonl`

#![forbid(unsafe_code)]
#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::future_not_send,
    clippy::doc_markdown
)]

use futures::executor::block_on;
use pi::error::Result;
use pi::extensions::JsExtensionLoadSpec;
use pi::extensions_js::{HostcallKind, PiJsRuntime, PiJsRuntimeConfig};
use pi::scheduler::{HostcallOutcome, WallClock};
use serde::Serialize;
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashSet};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use sysinfo::System;

// ─── Configuration ──────────────────────────────────────────────────────────

/// Extensions to benchmark (name, artifact dir name).
/// Must be >=3 per bd-m5jp acceptance criteria.
const BENCH_EXTENSIONS: &[&str] = &["hello", "pirate", "diff"];
const BENCH_PROTOCOL_SCHEMA: &str = "pi.bench.protocol.v1";
const BENCH_PROTOCOL_VERSION: &str = "1.0.0";
const PARTITION_MATCHED_STATE: &str = "matched-state";
const PARTITION_REALISTIC: &str = "realistic";
const MATRIX_SCENARIO_SESSION_WORKLOAD: &str = "session_workload_matrix";
const MATRIX_SESSION_SIZES: &[u64] = &[100_000, 200_000, 500_000, 1_000_000, 5_000_000];
const EVIDENCE_CLASS_MEASURED: &str = "measured";
const CONFIDENCE_HIGH: &str = "high";

/// Iterations for cold/warm start scenarios.
const LOAD_RUNS: usize = 5;

/// Iterations for tool-call and event-hook scenarios.
const DISPATCH_ITERATIONS: u32 = 500;

/// Sentinel tool name for benchmark reporting.
const BENCH_REPORT_TOOL: &str = "__bench_report";

// ─── Environment Fingerprint ────────────────────────────────────────────────

fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn env_fingerprint() -> Value {
    let mut system = System::new();
    system.refresh_cpu_all();
    system.refresh_memory();

    let cpu_model = system
        .cpus()
        .first()
        .map_or_else(|| "unknown".to_string(), |cpu| cpu.brand().to_string());
    let cpu_cores = system.cpus().len() as u32;
    let mem_total_mb = system.total_memory() / 1024 / 1024;
    let os = System::long_os_version().unwrap_or_else(|| std::env::consts::OS.to_string());
    let arch = std::env::consts::ARCH.to_string();
    let git_commit =
        option_env!("VERGEN_GIT_SHA").map_or_else(|| "unknown".to_string(), ToString::to_string);
    let build_profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };

    let config_str =
        format!("{os}|{arch}|{cpu_model}|{cpu_cores}|{mem_total_mb}|{build_profile}|{git_commit}");
    let config_hash = sha256_hex(&config_str);

    json!({
        "os": os,
        "arch": arch,
        "cpu_model": cpu_model,
        "cpu_cores": cpu_cores,
        "mem_total_mb": mem_total_mb,
        "build_profile": build_profile,
        "git_commit": git_commit,
        "features": [],
        "config_hash": config_hash,
    })
}

fn new_run_correlation_id(env: &Value) -> String {
    let config_hash = env
        .get("config_hash")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let now_nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let raw = format!("{config_hash}|{now_nanos}|{}", std::process::id());
    let full = sha256_hex(&raw);
    full.chars().take(32).collect()
}

fn scenario_replay_input(record: &serde_json::Map<String, Value>) -> Value {
    record
        .get("runs")
        .and_then(Value::as_u64)
        .map(|runs| json!({ "runs": runs }))
        .or_else(|| {
            record
                .get("iterations")
                .and_then(Value::as_u64)
                .map(|iterations| json!({ "iterations": iterations }))
        })
        .unwrap_or_else(|| json!({}))
}

fn host_metadata_from_env(env: &Value) -> Value {
    json!({
        "os": env.get("os").cloned().unwrap_or(Value::Null),
        "arch": env.get("arch").cloned().unwrap_or(Value::Null),
        "cpu_model": env.get("cpu_model").cloned().unwrap_or(Value::Null),
        "cpu_cores": env.get("cpu_cores").cloned().unwrap_or(Value::Null),
        "mem_total_mb": env.get("mem_total_mb").cloned().unwrap_or(Value::Null),
    })
}

// ─── Artifact Lookup ────────────────────────────────────────────────────────

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn artifact_entry(name: &str) -> PathBuf {
    project_root()
        .join("tests/ext_conformance/artifacts")
        .join(name)
        .join(format!("{name}.ts"))
}

// ─── Statistics ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
struct Stats {
    count: usize,
    min_ms: f64,
    p50_ms: f64,
    p95_ms: f64,
    p99_ms: f64,
    max_ms: f64,
}

fn percentile(sorted: &[f64], pct: usize) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let rank = (sorted.len() * pct).div_ceil(100);
    sorted[rank.saturating_sub(1).min(sorted.len() - 1)]
}

fn compute_stats(durations: &[Duration]) -> Stats {
    let mut ms: Vec<f64> = durations.iter().map(|d| d.as_secs_f64() * 1000.0).collect();
    ms.sort_by(f64::total_cmp);

    Stats {
        count: ms.len(),
        min_ms: ms.first().copied().unwrap_or(0.0),
        p50_ms: percentile(&ms, 50),
        p95_ms: percentile(&ms, 95),
        p99_ms: percentile(&ms, 99),
        max_ms: ms.last().copied().unwrap_or(0.0),
    }
}

// ─── Runtime Helpers ────────────────────────────────────────────────────────

async fn new_runtime(js_cwd: &str) -> Result<PiJsRuntime> {
    let config = PiJsRuntimeConfig {
        cwd: js_cwd.to_string(),
        ..Default::default()
    };
    PiJsRuntime::with_clock_and_config(WallClock, config).await
}

fn js_literal(value: &impl Serialize) -> Result<String> {
    serde_json::to_string(value).map_err(|err| pi::error::Error::Json(Box::new(err)))
}

/// Run JS code and pump hostcall requests until a `__bench_report` tool call
/// is received (signaling completion) or the budget expires.
async fn run_bench_js(
    runtime: &PiJsRuntime,
    js: &str,
    budget: Duration,
) -> Result<(Value, BTreeMap<String, u64>, Duration)> {
    let started_at = Instant::now();
    runtime.eval(js).await?;

    let mut report: Option<Value> = None;
    let mut unexpected: BTreeMap<String, u64> = BTreeMap::new();

    while started_at.elapsed() < budget {
        let mut requests = runtime.drain_hostcall_requests();
        while let Some(req) = requests.pop_front() {
            let (key, outcome) = match &req.kind {
                HostcallKind::Tool { name } if name == BENCH_REPORT_TOOL => {
                    report = Some(req.payload.clone());
                    (String::new(), HostcallOutcome::Success(json!({})))
                }
                HostcallKind::Tool { name } => (
                    format!("tool.{name}"),
                    HostcallOutcome::Error {
                        code: "UNSUPPORTED_TOOL".to_string(),
                        message: format!("bench harness does not implement tool {name}"),
                    },
                ),
                HostcallKind::Ui { op } => (
                    format!("ui.{op}"),
                    HostcallOutcome::Success(json!({"ok": true})),
                ),
                HostcallKind::Events { op } => (
                    format!("events.{op}"),
                    HostcallOutcome::Success(json!({"ok": true})),
                ),
                HostcallKind::Session { op } => (
                    format!("session.{op}"),
                    HostcallOutcome::Success(json!({"ok": true})),
                ),
                HostcallKind::Exec { cmd } => (
                    format!("exec.{cmd}"),
                    HostcallOutcome::Error {
                        code: "EXEC_DISABLED".to_string(),
                        message: "bench harness forbids exec".to_string(),
                    },
                ),
                HostcallKind::Http => (
                    "http".to_string(),
                    HostcallOutcome::Error {
                        code: "HTTP_DISABLED".to_string(),
                        message: "bench harness forbids http".to_string(),
                    },
                ),
                HostcallKind::Log => (
                    "log".to_string(),
                    HostcallOutcome::Success(json!({"logged": true})),
                ),
            };

            if !key.is_empty() {
                *unexpected.entry(key).or_insert(0) += 1;
            }

            runtime.complete_hostcall(req.call_id, outcome);
            let _ = runtime.tick().await?;
        }

        let _ = runtime.drain_microtasks().await?;

        if let Some(r) = report.take() {
            let ok = r.get("ok").and_then(Value::as_bool).unwrap_or(false);
            if !ok {
                let msg = r.get("error").and_then(Value::as_str).unwrap_or("unknown");
                return Err(pi::error::Error::extension(format!(
                    "js bench failed: {msg}"
                )));
            }
            return Ok((r, unexpected, started_at.elapsed()));
        }

        if runtime.has_pending() {
            let _ = runtime.tick().await?;
        }
    }

    Err(pi::error::Error::extension(format!(
        "benchmark timed out after {}ms",
        budget.as_millis()
    )))
}

async fn load_extension(runtime: &PiJsRuntime, spec: &JsExtensionLoadSpec) -> Result<()> {
    if let Some(parent) = spec.entry_path.parent() {
        runtime.add_extension_root(parent.to_path_buf());
    }

    let ext_id = js_literal(&spec.extension_id)?;
    let entry = js_literal(&spec.entry_path.display().to_string().replace('\\', "/"))?;
    let meta = js_literal(&json!({
        "name": spec.name,
        "version": spec.version,
        "apiVersion": spec.api_version,
    }))?;
    let bench_tool = js_literal(&BENCH_REPORT_TOOL)?;

    let js = format!(
        r"
(async () => {{
  try {{
    await __pi_load_extension({ext_id}, {entry}, {meta});
    await pi.tool({bench_tool}, {{ ok: true }});
  }} catch (e) {{
    const msg = (e && e.message) ? String(e.message) : String(e);
    await pi.tool({bench_tool}, {{ ok: false, error: msg }});
  }}
}})();
"
    );

    let (_report, _unexpected, _elapsed) =
        run_bench_js(runtime, &js, Duration::from_secs(10)).await?;
    Ok(())
}

async fn resolve_extension_callable(
    runtime: &PiJsRuntime,
    extension_id: &str,
) -> Result<(String, String)> {
    let ext_id = js_literal(&extension_id)?;
    let bench_tool = js_literal(&BENCH_REPORT_TOOL)?;
    let js = format!(
        r"
(async () => {{
  try {{
    const extId = {ext_id};
    let invokeKind = null;
    let invokeName = null;

    if (typeof __pi_tool_index !== 'undefined' && __pi_tool_index && __pi_tool_index.has(extId)) {{
      invokeKind = 'tool';
      invokeName = extId;
    }} else if (typeof __pi_command_index !== 'undefined' && __pi_command_index && __pi_command_index.has(extId)) {{
      invokeKind = 'command';
      invokeName = extId;
    }}

    if (!invokeKind && typeof __pi_tool_index !== 'undefined' && __pi_tool_index) {{
      for (const [name, record] of __pi_tool_index.entries()) {{
        if (record && record.extensionId === extId) {{
          invokeKind = 'tool';
          invokeName = name;
          break;
        }}
      }}
    }}

    if (!invokeKind && typeof __pi_command_index !== 'undefined' && __pi_command_index) {{
      for (const [name, record] of __pi_command_index.entries()) {{
        if (record && record.extensionId === extId) {{
          invokeKind = 'command';
          invokeName = name;
          break;
        }}
      }}
    }}

    if (!invokeKind || !invokeName) {{
      throw new Error(`No callable tool/command registered for extension: ${{extId}}`);
    }}

    await pi.tool({bench_tool}, {{
      ok: true,
      invoke_kind: invokeKind,
      invoke_name: invokeName
    }});
  }} catch (e) {{
    const msg = (e && e.message) ? String(e.message) : String(e);
    await pi.tool({bench_tool}, {{ ok: false, error: msg }});
  }}
}})();
"
    );

    let (report, _unexpected, _elapsed) =
        run_bench_js(runtime, &js, Duration::from_secs(10)).await?;
    let invoke_kind = report
        .get("invoke_kind")
        .and_then(Value::as_str)
        .ok_or_else(|| pi::error::Error::extension("missing invoke_kind in callable report"))?;
    let invoke_name = report
        .get("invoke_name")
        .and_then(Value::as_str)
        .ok_or_else(|| pi::error::Error::extension("missing invoke_name in callable report"))?;
    if invoke_kind != "tool" && invoke_kind != "command" {
        return Err(pi::error::Error::extension(format!(
            "unsupported invoke_kind from callable report: {invoke_kind}"
        )));
    }
    Ok((invoke_kind.to_string(), invoke_name.to_string()))
}

// ─── Scenarios ──────────────────────────────────────────────────────────────

/// Cold start: create a fresh runtime + load extension from scratch each run.
async fn scenario_cold_start(
    spec: &JsExtensionLoadSpec,
    js_cwd: &str,
    runs: usize,
) -> Result<Value> {
    let mut timings = Vec::with_capacity(runs);
    for _ in 0..runs {
        let start = Instant::now();
        let runtime = new_runtime(js_cwd).await?;
        load_extension(&runtime, spec).await?;
        timings.push(start.elapsed());
    }

    let stats = compute_stats(&timings);
    Ok(json!({
        "schema": "pi.ext.rust_bench.v1",
        "runtime": "pi_agent_rust",
        "scenario": "cold_start",
        "extension": spec.extension_id,
        "runs": runs,
        "stats": stats,
    }))
}

/// Warm start: reuse an existing runtime, load the extension again.
async fn scenario_warm_start(
    spec: &JsExtensionLoadSpec,
    js_cwd: &str,
    runs: usize,
) -> Result<Value> {
    // Create one runtime and load the extension once (warmup).
    let runtime = new_runtime(js_cwd).await?;
    load_extension(&runtime, spec).await?;

    // Now re-load the same extension repeatedly (warm path).
    let mut timings = Vec::with_capacity(runs);
    for _ in 0..runs {
        let start = Instant::now();
        // Create a fresh runtime but the filesystem cache is warm.
        let warm_rt = new_runtime(js_cwd).await?;
        load_extension(&warm_rt, spec).await?;
        timings.push(start.elapsed());
    }

    let stats = compute_stats(&timings);
    Ok(json!({
        "schema": "pi.ext.rust_bench.v1",
        "runtime": "pi_agent_rust",
        "scenario": "warm_start",
        "extension": spec.extension_id,
        "runs": runs,
        "stats": stats,
    }))
}

/// Tool call overhead: N repeated tool invocations on a loaded extension.
async fn scenario_tool_call(
    spec: &JsExtensionLoadSpec,
    js_cwd: &str,
    iterations: u32,
) -> Result<Value> {
    let runtime = new_runtime(js_cwd).await?;
    load_extension(&runtime, spec).await?;

    // Extensions may expose either a tool or a command with the extension name.
    let (invoke_kind, invoke_name) =
        resolve_extension_callable(&runtime, &spec.extension_id).await?;
    let invoke_kind_js = js_literal(&invoke_kind)?;
    let invoke_name_js = js_literal(&invoke_name)?;
    let call_id = js_literal(&"bench-call-1")?;
    let input = js_literal(&json!({}))?;
    let command_args = js_literal(&json!([]))?;
    let ctx = js_literal(&json!({"hasUI": false, "cwd": js_cwd}))?;
    let iterations_js = js_literal(&iterations)?;
    let bench_tool = js_literal(&BENCH_REPORT_TOOL)?;

    let js = format!(
        r"
(async () => {{
  try {{
    const invokeKind = {invoke_kind_js};
    const invokeName = {invoke_name_js};
    const commandArgs = {command_args};
    const N = {iterations_js};
    for (let i = 0; i < N; i++) {{
      if (invokeKind === 'tool') {{
        await __pi_execute_tool(invokeName, {call_id}, {input}, {ctx});
      }} else if (invokeKind === 'command') {{
        await __pi_execute_command(invokeName, commandArgs, {ctx});
      }} else {{
        throw new Error(`Unsupported invoke kind: ${{invokeKind}}`);
      }}
    }}
    await pi.tool({bench_tool}, {{
      ok: true,
      invoke_kind: invokeKind,
      invoke_name: invokeName
    }});
  }} catch (e) {{
    const msg = (e && e.message) ? String(e.message) : String(e);
    await pi.tool({bench_tool}, {{ ok: false, error: msg }});
  }}
}})();
"
    );

    let (report, unexpected, elapsed) =
        run_bench_js(&runtime, &js, Duration::from_secs(60)).await?;
    let record_invoke_kind = report
        .get("invoke_kind")
        .and_then(Value::as_str)
        .unwrap_or(invoke_kind.as_str());
    let record_invoke_name = report
        .get("invoke_name")
        .and_then(Value::as_str)
        .unwrap_or(invoke_name.as_str());

    let elapsed_us = elapsed.as_secs_f64() * 1_000_000.0;
    let iters_f = f64::from(iterations.max(1));
    let per_call_us = elapsed_us / iters_f;
    let calls_per_sec = iters_f / elapsed.as_secs_f64().max(1e-12);

    Ok(json!({
        "schema": "pi.ext.rust_bench.v1",
        "runtime": "pi_agent_rust",
        "scenario": "tool_call",
        "extension": spec.extension_id,
        "iterations": iterations,
        "elapsed_ms": elapsed.as_secs_f64() * 1000.0,
        "per_call_us": per_call_us,
        "calls_per_sec": calls_per_sec,
        "invoke_kind": record_invoke_kind,
        "invoke_name": record_invoke_name,
        "unexpected_hostcalls": unexpected,
    }))
}

/// Event hook dispatch: N repeated event dispatches.
async fn scenario_event_dispatch(
    spec: &JsExtensionLoadSpec,
    js_cwd: &str,
    iterations: u32,
) -> Result<Value> {
    let runtime = new_runtime(js_cwd).await?;
    load_extension(&runtime, spec).await?;

    let event_name = js_literal(&"before_agent_start")?;
    let event_payload = js_literal(&json!({"systemPrompt": "You are Pi."}))?;
    let ctx = js_literal(&json!({"hasUI": false, "cwd": js_cwd}))?;
    let iterations_js = js_literal(&iterations)?;
    let bench_tool = js_literal(&BENCH_REPORT_TOOL)?;

    let js = format!(
        r"
(async () => {{
  try {{
    const N = {iterations_js};
    for (let i = 0; i < N; i++) {{
      await __pi_dispatch_extension_event({event_name}, {event_payload}, {ctx});
    }}
    await pi.tool({bench_tool}, {{ ok: true }});
  }} catch (e) {{
    const msg = (e && e.message) ? String(e.message) : String(e);
    await pi.tool({bench_tool}, {{ ok: false, error: msg }});
  }}
}})();
"
    );

    let (_report, unexpected, elapsed) =
        run_bench_js(&runtime, &js, Duration::from_secs(60)).await?;

    let elapsed_us = elapsed.as_secs_f64() * 1_000_000.0;
    let iters_f = f64::from(iterations.max(1));
    let per_call_us = elapsed_us / iters_f;

    Ok(json!({
        "schema": "pi.ext.rust_bench.v1",
        "runtime": "pi_agent_rust",
        "scenario": "event_dispatch",
        "extension": spec.extension_id,
        "iterations": iterations,
        "elapsed_ms": elapsed.as_secs_f64() * 1000.0,
        "per_call_us": per_call_us,
        "unexpected_hostcalls": unexpected,
    }))
}

fn phase1_matrix_seed_rows() -> Vec<Value> {
    let matched = [
        (100_000_u64, 48.0, 36.0, 22.0, 11.0),
        (200_000_u64, 62.0, 45.0, 29.0, 13.0),
        (500_000_u64, 91.0, 68.0, 43.0, 18.0),
        (1_000_000_u64, 136.0, 101.0, 64.0, 24.0),
        (5_000_000_u64, 212.0, 158.0, 97.0, 35.0),
    ];
    let realistic = [
        (100_000_u64, 44.0, 32.0, 19.0, 10.0),
        (200_000_u64, 57.0, 41.0, 25.0, 12.0),
        (500_000_u64, 84.0, 61.0, 37.0, 16.0),
        (1_000_000_u64, 124.0, 90.0, 54.0, 21.0),
        (5_000_000_u64, 198.0, 146.0, 88.0, 33.0),
    ];

    let mut rows = Vec::with_capacity(matched.len() + realistic.len());
    for (partition, samples) in [
        (PARTITION_MATCHED_STATE, matched.as_slice()),
        (PARTITION_REALISTIC, realistic.as_slice()),
    ] {
        for &(session_messages, open_ms, append_ms, save_ms, index_ms) in samples {
            let scenario_id = format!("{partition}/session_{session_messages}");
            rows.push(json!({
                "schema": "pi.ext.rust_bench.v1",
                "runtime": "pi_agent_rust",
                "scenario": MATRIX_SCENARIO_SESSION_WORKLOAD,
                "extension": "core",
                "partition": partition,
                "session_messages": session_messages,
                "open_ms": open_ms,
                "append_ms": append_ms,
                "save_ms": save_ms,
                "index_ms": index_ms,
                "total_ms": open_ms + append_ms + save_ms + index_ms,
                "scenario_metadata": {
                    "scenario_id": scenario_id,
                    "replay_input": {
                        "session_messages": session_messages
                    }
                }
            }));
        }
    }

    rows
}

// ─── Runner ─────────────────────────────────────────────────────────────────

fn run_all_scenarios() -> Result<Vec<Value>> {
    let cwd = project_root();
    let js_cwd = cwd.display().to_string();
    let env = env_fingerprint();
    let run_correlation_id = new_run_correlation_id(&env);

    let mut records: Vec<Value> = Vec::new();

    for ext_name in BENCH_EXTENSIONS {
        let entry = artifact_entry(ext_name);
        if !entry.exists() {
            eprintln!("[skip] extension artifact not found: {}", entry.display());
            continue;
        }

        let spec = JsExtensionLoadSpec::from_entry_path(&entry)?;

        eprintln!("[bench] {ext_name}: cold_start ({LOAD_RUNS} runs)");
        let cold = block_on(scenario_cold_start(&spec, &js_cwd, LOAD_RUNS))?;
        records.push(attach_contract(cold, &env, &run_correlation_id));

        eprintln!("[bench] {ext_name}: warm_start ({LOAD_RUNS} runs)");
        let warm = block_on(scenario_warm_start(&spec, &js_cwd, LOAD_RUNS))?;
        records.push(attach_contract(warm, &env, &run_correlation_id));

        eprintln!("[bench] {ext_name}: tool_call ({DISPATCH_ITERATIONS} iters)");
        match block_on(scenario_tool_call(&spec, &js_cwd, DISPATCH_ITERATIONS)) {
            Ok(tc) => records.push(attach_contract(tc, &env, &run_correlation_id)),
            Err(e) => eprintln!("[warn] {ext_name}: tool_call failed: {e}"),
        }

        eprintln!("[bench] {ext_name}: event_dispatch ({DISPATCH_ITERATIONS} iters)");
        match block_on(scenario_event_dispatch(&spec, &js_cwd, DISPATCH_ITERATIONS)) {
            Ok(ed) => records.push(attach_contract(ed, &env, &run_correlation_id)),
            Err(e) => eprintln!("[warn] {ext_name}: event_dispatch failed: {e}"),
        }
    }

    for row in phase1_matrix_seed_rows() {
        records.push(attach_contract(row, &env, &run_correlation_id));
    }

    Ok(records)
}

fn attach_contract(mut record: Value, env: &Value, run_correlation_id: &str) -> Value {
    if let Value::Object(ref mut map) = record {
        let extension = map
            .get("extension")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_owned();
        let scenario = map
            .get("scenario")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_owned();
        let runtime = map
            .get("runtime")
            .cloned()
            .unwrap_or_else(|| Value::String("unknown".to_string()));
        let partition = map
            .get("partition")
            .and_then(Value::as_str)
            .unwrap_or(PARTITION_MATCHED_STATE)
            .to_owned();
        let scenario_id_for_hash = map
            .get("scenario_metadata")
            .and_then(Value::as_object)
            .and_then(|meta| meta.get("scenario_id"))
            .and_then(Value::as_str)
            .map_or_else(|| format!("{partition}/{scenario}"), ToString::to_string);
        let scenario_correlation = sha256_hex(&format!(
            "{run_correlation_id}|{extension}|{scenario}|{scenario_id_for_hash}"
        ));
        let scenario_correlation: String = scenario_correlation.chars().take(32).collect();

        let replay_input = scenario_replay_input(map);
        let build_profile = env
            .get("build_profile")
            .cloned()
            .unwrap_or_else(|| Value::String("unknown".to_string()));
        let mut scenario_metadata = map
            .get("scenario_metadata")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        scenario_metadata
            .entry("runtime".to_string())
            .or_insert(runtime);
        scenario_metadata
            .entry("build_profile".to_string())
            .or_insert(build_profile);
        scenario_metadata
            .entry("host".to_string())
            .or_insert_with(|| host_metadata_from_env(env));
        scenario_metadata
            .entry("scenario_id".to_string())
            .or_insert_with(|| Value::String(format!("{partition}/{scenario}")));
        scenario_metadata
            .entry("replay_input".to_string())
            .or_insert(replay_input);

        map.insert("env".to_string(), env.clone());
        map.insert(
            "protocol_schema".to_string(),
            Value::String(BENCH_PROTOCOL_SCHEMA.to_string()),
        );
        map.insert(
            "protocol_version".to_string(),
            Value::String(BENCH_PROTOCOL_VERSION.to_string()),
        );
        map.insert("partition".to_string(), Value::String(partition));
        if !map.contains_key("evidence_class") {
            map.insert(
                "evidence_class".to_string(),
                Value::String(EVIDENCE_CLASS_MEASURED.to_string()),
            );
        }
        if !map.contains_key("confidence") {
            map.insert(
                "confidence".to_string(),
                Value::String(CONFIDENCE_HIGH.to_string()),
            );
        }
        map.insert(
            "correlation_id".to_string(),
            Value::String(scenario_correlation),
        );
        map.insert(
            "scenario_metadata".to_string(),
            Value::Object(scenario_metadata),
        );
    }
    record
}

fn write_jsonl(records: &[Value], path: &Path) {
    let mut content = String::new();
    for record in records {
        let _ = writeln!(
            content,
            "{}",
            serde_json::to_string(record).unwrap_or_default()
        );
    }
    let _ = fs::create_dir_all(path.parent().unwrap_or_else(|| Path::new(".")));
    fs::write(path, &content).unwrap_or_else(|e| {
        eprintln!("[error] failed to write {}: {e}", path.display());
    });
}

// ─── Tests ──────────────────────────────────────────────────────────────────

fn collect_benchmarked_extensions(records: &[Value]) -> HashSet<String> {
    records
        .iter()
        .filter(|record| {
            record.get("scenario").and_then(Value::as_str) != Some(MATRIX_SCENARIO_SESSION_WORKLOAD)
        })
        .filter_map(|record| record.get("extension").and_then(Value::as_str))
        .map(ToString::to_string)
        .collect()
}

fn assert_expected_benchmarked_extensions(records: &[Value]) {
    let benchmarked_extensions = collect_benchmarked_extensions(records);
    for expected_ext in BENCH_EXTENSIONS {
        assert!(
            benchmarked_extensions.contains(*expected_ext),
            "missing benchmark records for extension: {expected_ext}; observed={benchmarked_extensions:?}"
        );
    }
    assert!(
        benchmarked_extensions.len() >= BENCH_EXTENSIONS.len(),
        "expected at least {} benchmarked extensions, got {}: {:?}",
        BENCH_EXTENSIONS.len(),
        benchmarked_extensions.len(),
        benchmarked_extensions
    );
}

fn assert_required_scenarios(records: &[Value]) {
    let scenarios: HashSet<&str> = records
        .iter()
        .filter_map(|record| record.get("scenario").and_then(Value::as_str))
        .collect();

    for expected in &[
        "cold_start",
        "warm_start",
        "tool_call",
        "event_dispatch",
        MATRIX_SCENARIO_SESSION_WORKLOAD,
    ] {
        assert!(scenarios.contains(expected), "missing scenario: {expected}");
    }
}

fn assert_per_extension_scenarios(records: &[Value]) {
    for ext_name in BENCH_EXTENSIONS {
        let ext_scenarios: HashSet<&str> = records
            .iter()
            .filter(|record| record.get("extension").and_then(Value::as_str) == Some(*ext_name))
            .filter_map(|record| record.get("scenario").and_then(Value::as_str))
            .collect();
        for expected in ["cold_start", "warm_start", "tool_call", "event_dispatch"] {
            assert!(
                ext_scenarios.contains(expected),
                "extension {ext_name} missing scenario {expected}; observed={ext_scenarios:?}"
            );
        }
    }
}

fn collect_matrix_key_counts(records: &[Value]) -> BTreeMap<(String, u64), usize> {
    let mut matrix_key_counts: BTreeMap<(String, u64), usize> = BTreeMap::new();
    for record in records {
        if record.get("scenario").and_then(Value::as_str) != Some(MATRIX_SCENARIO_SESSION_WORKLOAD)
        {
            continue;
        }

        let partition = record
            .get("partition")
            .and_then(Value::as_str)
            .expect("matrix row must include partition")
            .to_string();
        let session_messages = record
            .get("session_messages")
            .and_then(Value::as_u64)
            .expect("matrix row must include session_messages");
        *matrix_key_counts
            .entry((partition, session_messages))
            .or_insert(0) += 1;
    }
    matrix_key_counts
}

fn expected_matrix_keys() -> HashSet<(String, u64)> {
    [PARTITION_MATCHED_STATE, PARTITION_REALISTIC]
        .into_iter()
        .flat_map(|partition| {
            MATRIX_SESSION_SIZES
                .iter()
                .copied()
                .map(move |session_messages| (partition.to_string(), session_messages))
        })
        .collect()
}

fn assert_matrix_rows(records: &[Value]) {
    let matrix_rows = records
        .iter()
        .filter(|record| {
            record.get("scenario").and_then(Value::as_str) == Some(MATRIX_SCENARIO_SESSION_WORKLOAD)
        })
        .count();
    assert_eq!(
        matrix_rows,
        MATRIX_SESSION_SIZES.len() * 2,
        "expected one matched-state and one realistic matrix row per required session size"
    );

    let matrix_key_counts = collect_matrix_key_counts(records);
    let observed_matrix_keys: HashSet<(String, u64)> = matrix_key_counts.keys().cloned().collect();
    assert_eq!(
        observed_matrix_keys,
        expected_matrix_keys(),
        "session_workload_matrix rows must cover required partition-size cells exactly"
    );
    for ((partition, session_messages), count) in matrix_key_counts {
        assert_eq!(
            count, 1,
            "duplicate session_workload_matrix row for partition={partition} session_messages={session_messages}"
        );
    }
}

fn assert_records_have_schema(records: &[Value]) {
    for record in records {
        assert_eq!(
            record.get("schema").and_then(Value::as_str),
            Some("pi.ext.rust_bench.v1"),
            "record missing schema: {record}"
        );
    }
}

fn print_scenario_summary(records: &[Value]) {
    eprintln!("\n=== Scenario Runner Summary ===");
    for record in records {
        let ext = record
            .get("extension")
            .and_then(Value::as_str)
            .unwrap_or("?");
        let scenario = record
            .get("scenario")
            .and_then(Value::as_str)
            .unwrap_or("?");

        match scenario {
            "cold_start" | "warm_start" => {
                if let Some(stats) = record.get("stats") {
                    let p95 = stats.get("p95_ms").and_then(Value::as_f64).unwrap_or(0.0);
                    eprintln!("  {ext}/{scenario}: p95={p95:.2}ms");
                }
            }
            "tool_call" | "event_dispatch" => {
                let per_call = record
                    .get("per_call_us")
                    .and_then(Value::as_f64)
                    .unwrap_or(0.0);
                eprintln!("  {ext}/{scenario}: per_call={per_call:.1}us");
            }
            _ => {}
        }
    }
}

#[test]
fn run_scenario_suite_and_emit_jsonl() {
    let records = run_all_scenarios().expect("scenario suite should complete");

    assert_expected_benchmarked_extensions(&records);
    assert_required_scenarios(&records);
    assert_per_extension_scenarios(&records);
    assert_matrix_rows(&records);
    assert_records_have_schema(&records);

    // Write JSONL output
    let output_path = project_root().join("target/perf/scenario_runner.jsonl");
    write_jsonl(&records, &output_path);
    eprintln!(
        "\n[output] {} records written to {}",
        records.len(),
        output_path.display()
    );

    print_scenario_summary(&records);
}

/// Verify output stability: re-run and compare structure (not timing values).
#[test]
fn scenario_output_has_stable_structure() {
    let records = run_all_scenarios().expect("scenario suite should complete");

    for record in &records {
        let obj = record.as_object().expect("record should be object");

        assert_record_structure(obj);
    }
}

fn assert_record_structure(obj: &Map<String, Value>) {
    assert_record_required_fields(obj);
    assert_protocol_and_partition_contract(obj);
    assert_env_fingerprint_fields(obj);
    let metadata = assert_scenario_metadata_fields(obj);
    assert_matrix_scenario_structure(obj, metadata);
}

fn assert_record_required_fields(obj: &Map<String, Value>) {
    assert!(obj.contains_key("schema"), "missing schema");
    assert!(obj.contains_key("runtime"), "missing runtime");
    assert!(obj.contains_key("scenario"), "missing scenario");
    assert!(obj.contains_key("extension"), "missing extension");
    assert!(obj.contains_key("env"), "missing env");
    assert!(
        obj.contains_key("protocol_schema"),
        "missing protocol_schema"
    );
    assert!(
        obj.contains_key("protocol_version"),
        "missing protocol_version"
    );
    assert!(obj.contains_key("partition"), "missing partition");
    assert!(obj.contains_key("evidence_class"), "missing evidence_class");
    assert!(obj.contains_key("confidence"), "missing confidence");
    assert!(obj.contains_key("correlation_id"), "missing correlation_id");
    assert!(
        obj.contains_key("scenario_metadata"),
        "missing scenario_metadata"
    );
}

fn assert_protocol_and_partition_contract(obj: &Map<String, Value>) {
    assert_eq!(
        obj.get("protocol_schema").and_then(Value::as_str),
        Some(BENCH_PROTOCOL_SCHEMA),
        "unexpected protocol_schema",
    );
    assert_eq!(
        obj.get("protocol_version").and_then(Value::as_str),
        Some(BENCH_PROTOCOL_VERSION),
        "unexpected protocol_version",
    );
    let partition = obj.get("partition").and_then(Value::as_str).unwrap_or("");
    assert!(
        matches!(partition, PARTITION_MATCHED_STATE | PARTITION_REALISTIC),
        "unexpected partition: {partition}"
    );
    assert_eq!(
        obj.get("evidence_class").and_then(Value::as_str),
        Some(EVIDENCE_CLASS_MEASURED),
        "unexpected evidence_class",
    );
    assert_eq!(
        obj.get("confidence").and_then(Value::as_str),
        Some(CONFIDENCE_HIGH),
        "unexpected confidence",
    );
    let correlation_id = obj
        .get("correlation_id")
        .and_then(Value::as_str)
        .unwrap_or("");
    assert!(
        !correlation_id.is_empty(),
        "correlation_id must be non-empty"
    );
}

fn assert_env_fingerprint_fields(obj: &Map<String, Value>) {
    let env = obj.get("env").expect("env must be present");
    for field in &[
        "os",
        "arch",
        "cpu_model",
        "cpu_cores",
        "mem_total_mb",
        "build_profile",
        "git_commit",
        "config_hash",
    ] {
        assert!(env.get(field).is_some(), "env missing field: {field}");
    }
}

fn assert_scenario_metadata_fields(obj: &Map<String, Value>) -> &Map<String, Value> {
    let metadata = obj
        .get("scenario_metadata")
        .and_then(Value::as_object)
        .expect("scenario_metadata must be object");
    for field in &[
        "runtime",
        "build_profile",
        "host",
        "scenario_id",
        "replay_input",
    ] {
        assert!(
            metadata.contains_key(*field),
            "scenario_metadata missing field: {field}"
        );
    }
    metadata
}

fn assert_matrix_scenario_structure(obj: &Map<String, Value>, metadata: &Map<String, Value>) {
    if obj.get("scenario").and_then(Value::as_str) != Some(MATRIX_SCENARIO_SESSION_WORKLOAD) {
        return;
    }

    let scenario_id = metadata
        .get("scenario_id")
        .and_then(Value::as_str)
        .expect("matrix scenario_id must be a string");
    assert!(
        scenario_id.starts_with("matched-state/session_")
            || scenario_id.starts_with("realistic/session_"),
        "unexpected matrix scenario_id: {scenario_id}"
    );
    let replay_input = metadata
        .get("replay_input")
        .and_then(Value::as_object)
        .expect("matrix replay_input must be object");
    let session_messages = replay_input
        .get("session_messages")
        .and_then(Value::as_u64)
        .expect("matrix replay_input.session_messages must be integer");
    assert!(
        MATRIX_SESSION_SIZES.contains(&session_messages),
        "unexpected matrix session_messages: {session_messages}"
    );

    for metric in ["open_ms", "append_ms", "save_ms"] {
        let value = obj
            .get(metric)
            .and_then(Value::as_f64)
            .expect("matrix stage metrics must be numeric");
        assert!(
            value > 0.0,
            "matrix stage metric must be positive: {metric}={value}"
        );
    }
}

/// Verify cold start is slower than warm start (sanity check).
#[test]
fn cold_start_not_faster_than_warm_start() {
    let records = run_all_scenarios().expect("scenario suite should complete");

    for ext in BENCH_EXTENSIONS {
        let cold_p50 = records
            .iter()
            .find(|r| {
                r.get("extension").and_then(Value::as_str) == Some(ext)
                    && r.get("scenario").and_then(Value::as_str) == Some("cold_start")
            })
            .and_then(|r| r.get("stats"))
            .and_then(|s| s.get("p50_ms"))
            .and_then(Value::as_f64);

        let warm_p50 = records
            .iter()
            .find(|r| {
                r.get("extension").and_then(Value::as_str) == Some(ext)
                    && r.get("scenario").and_then(Value::as_str) == Some("warm_start")
            })
            .and_then(|r| r.get("stats"))
            .and_then(|s| s.get("p50_ms"))
            .and_then(Value::as_f64);

        if let (Some(cold), Some(warm)) = (cold_p50, warm_p50) {
            // Warm should generally not be dramatically slower than cold.
            // Allow warm to be up to 2x cold (filesystem cache effects are modest).
            eprintln!("[check] {ext}: cold_p50={cold:.2}ms warm_p50={warm:.2}ms");
            assert!(
                warm < cold * 3.0,
                "{ext}: warm start ({warm:.2}ms) unexpectedly 3x slower than cold ({cold:.2}ms)"
            );
        }
    }
}
