//! `PiJS` workload harness for deterministic perf baselines.
#![forbid(unsafe_code)]

use clap::{Parser, ValueEnum};
use futures::executor::block_on;
use pi::error::{Error, Result};
use pi::extensions::{
    ExtensionManager, ExtensionRuntimeHandle, NativeRustExtensionLoadSpec,
    NativeRustExtensionRuntimeHandle,
};
use pi::extensions_js::PiJsRuntime;
use pi::perf_build;
use pi::scheduler::HostcallOutcome;
use serde_json::json;
use std::collections::VecDeque;
use std::fs;
use std::sync::Arc;
use std::time::{Duration, Instant};

const BENCH_BEGIN_FN: &str = "__bench_begin_roundtrip";
const BENCH_ASSERT_FN: &str = "__bench_assert_roundtrip";
const NATIVE_RUNTIME_TOOL_NAME: &str = "bench_tool";

const BENCH_TOOL_SETUP: &str = r#"
__pi_begin_extension("ext.bench", { name: "Bench" });
pi.registerTool({
  name: "bench_tool",
  description: "Benchmark tool",
  parameters: { type: "object", properties: { value: { type: "number" } } },
  execute: async (_callId, input) => {
    return { ok: true, value: input.value };
  },
});
globalThis.__bench_done = false;
globalThis.__bench_begin_roundtrip = () => {
  globalThis.__bench_done = false;
  return pi.tool("bench_tool", { value: 1 }).then(() => { globalThis.__bench_done = true; });
};
globalThis.__bench_assert_roundtrip = () => {
  if (!globalThis.__bench_done) {
    throw new Error("bench tool call did not resolve");
  }
};
__pi_end_extension();
"#;

const NATIVE_RUNTIME_DESCRIPTOR: &str = r#"
{
  "id": "ext.native.bench",
  "name": "Native Bench",
  "version": "0.0.0",
  "apiVersion": "1.0.0",
  "tools": [
    {
      "name": "bench_tool",
      "description": "Benchmark tool",
      "parameters": {
        "type": "object",
        "properties": {
          "value": { "type": "number" }
        }
      }
    }
  ],
  "toolOutputs": {
    "bench_tool": {
      "content": [
        { "type": "text", "text": "ok" }
      ],
      "details": { "ok": true, "runtime": "native-rust-runtime" },
      "is_error": false
    }
  }
}
"#;

#[derive(Parser, Debug)]
#[command(name = "pijs_workload")]
#[command(about = "Deterministic PiJS workload runner for perf baselines")]
struct Args {
    /// Outer loop iterations.
    #[arg(long, default_value_t = 200)]
    iterations: usize,
    /// Tool calls per iteration.
    #[arg(long, default_value_t = 1)]
    tool_calls: usize,
    /// Runtime engine used by the benchmark harness.
    #[arg(long, value_enum, default_value_t = WorkloadRuntimeEngine::Quickjs)]
    runtime_engine: WorkloadRuntimeEngine,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum WorkloadRuntimeEngine {
    Quickjs,
    NativeRustPreview,
    NativeRustRuntime,
}

impl WorkloadRuntimeEngine {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Quickjs => "quickjs",
            Self::NativeRustPreview => "native_rust_preview",
            Self::NativeRustRuntime => "native_rust_runtime",
        }
    }
}

#[derive(Debug)]
struct NativeHostcallRequest {
    call_id: u64,
}

#[derive(Debug, Default)]
struct NativeBenchRuntime {
    next_call_id: u64,
    pending: VecDeque<NativeHostcallRequest>,
    inflight_call_id: Option<u64>,
    roundtrip_done: bool,
}

impl NativeBenchRuntime {
    fn begin_roundtrip(&mut self) {
        self.roundtrip_done = false;
        self.next_call_id = self.next_call_id.saturating_add(1);
        let call_id = self.next_call_id;
        self.inflight_call_id = Some(call_id);
        self.pending.push_back(NativeHostcallRequest { call_id });
    }

    fn drain_hostcall_request(&mut self) -> Result<NativeHostcallRequest> {
        self.pending.pop_front().ok_or_else(|| {
            Error::extension("native workload: missing pending hostcall request".to_string())
        })
    }

    fn complete_hostcall(&mut self, call_id: u64, outcome: HostcallOutcome) -> Result<()> {
        let expected = self.inflight_call_id.take().ok_or_else(|| {
            Error::extension("native workload: no inflight hostcall to complete".to_string())
        })?;

        if expected != call_id {
            return Err(Error::extension(format!(
                "native workload: call_id mismatch (expected {expected}, got {call_id})"
            )));
        }

        match outcome {
            HostcallOutcome::Success(value) => {
                if value.as_bool() == Some(true) {
                    self.roundtrip_done = true;
                    Ok(())
                } else {
                    Err(Error::extension(
                        "native workload: completion payload missing boolean true".to_string(),
                    ))
                }
            }
            other => Err(Error::extension(format!(
                "native workload: unsupported completion outcome: {other:?}"
            ))),
        }
    }

    fn assert_roundtrip(&self) -> Result<()> {
        if self.roundtrip_done && self.pending.is_empty() {
            Ok(())
        } else {
            Err(Error::extension(
                "native workload: tool roundtrip did not resolve".to_string(),
            ))
        }
    }
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

#[allow(clippy::too_many_lines)]
fn run() -> Result<()> {
    let args = Args::parse();
    let build_profile = perf_build::detect_build_profile();
    let allocator = perf_build::resolve_bench_allocator();
    let binary_path = std::env::current_exe()
        .ok()
        .map_or_else(|| "unknown".to_string(), |path| path.display().to_string());

    let quickjs_runtime = if args.runtime_engine == WorkloadRuntimeEngine::Quickjs {
        let runtime = block_on(PiJsRuntime::new())?;
        block_on(runtime.eval(BENCH_TOOL_SETUP))?;
        Some(runtime)
    } else {
        None
    };
    let mut native_runtime = if args.runtime_engine == WorkloadRuntimeEngine::NativeRustPreview {
        Some(NativeBenchRuntime::default())
    } else {
        None
    };
    let native_runtime_handle = if args.runtime_engine == WorkloadRuntimeEngine::NativeRustRuntime {
        Some(setup_native_runtime_bench_handle()?)
    } else {
        None
    };

    let start = Instant::now();
    for _ in 0..args.iterations {
        for _ in 0..args.tool_calls {
            match args.runtime_engine {
                WorkloadRuntimeEngine::Quickjs => {
                    if let Some(runtime) = quickjs_runtime.as_ref() {
                        run_tool_roundtrip_quickjs(runtime)?;
                    } else {
                        return Err(Error::extension(
                            "quickjs runtime unexpectedly unavailable".to_string(),
                        ));
                    }
                }
                WorkloadRuntimeEngine::NativeRustPreview => {
                    if let Some(runtime) = native_runtime.as_mut() {
                        run_tool_roundtrip_native(runtime)?;
                    } else {
                        return Err(Error::extension(
                            "native runtime unexpectedly unavailable".to_string(),
                        ));
                    }
                }
                WorkloadRuntimeEngine::NativeRustRuntime => {
                    if let Some(runtime) = native_runtime_handle.as_ref() {
                        run_tool_roundtrip_native_runtime(runtime)?;
                    } else {
                        return Err(Error::extension(
                            "native runtime handle unexpectedly unavailable".to_string(),
                        ));
                    }
                }
            }
        }
    }
    let elapsed = start.elapsed();

    let total_calls = args.iterations.saturating_mul(args.tool_calls);
    let elapsed_millis = elapsed.as_millis();
    let elapsed_micros = elapsed.as_micros();
    let total_calls_u128 = total_calls as u128;

    let per_call_us = elapsed_micros.checked_div(total_calls_u128).unwrap_or(0);
    let calls_count_u32 = u32::try_from(total_calls_u128).unwrap_or(u32::MAX);
    let calls_count_float = f64::from(calls_count_u32);
    let per_call_micros_f64 = if total_calls_u128 == 0 {
        0.0
    } else {
        elapsed.as_secs_f64() * 1_000_000.0 / calls_count_float
    };
    let per_call_nanos_f64 = if total_calls_u128 == 0 {
        0.0
    } else {
        elapsed.as_secs_f64() * 1_000_000_000.0 / calls_count_float
    };
    let calls_per_sec = total_calls_u128
        .saturating_mul(1_000_000)
        .checked_div(elapsed_micros)
        .unwrap_or(0);

    println!(
        "{}",
        json!({
            "schema": "pi.perf.workload.v1",
            "tool": "pijs_workload",
            "scenario": "tool_call_roundtrip",
            "iterations": args.iterations,
            "tool_calls_per_iteration": args.tool_calls,
            "total_calls": total_calls,
            "elapsed_ms": elapsed_millis,
            "elapsed_us": elapsed_micros,
            "per_call_us": per_call_us,
            "per_call_us_f64": per_call_micros_f64,
            "per_call_ns_f64": per_call_nanos_f64,
            "calls_per_sec": calls_per_sec,
            "build_profile": build_profile,
            "runtime_engine": args.runtime_engine.as_str(),
            "allocator_requested": allocator.requested,
            "allocator_request_source": allocator.requested_source,
            "allocator_effective": allocator.effective.as_str(),
            "allocator_fallback_reason": allocator.fallback_reason,
            "binary_path": binary_path,
        })
    );

    if let Some(runtime) = native_runtime_handle {
        let _ = block_on(runtime.shutdown(Duration::from_secs(5)));
    }

    Ok(())
}

fn setup_native_runtime_bench_handle() -> Result<ExtensionRuntimeHandle> {
    let descriptor_path = std::env::temp_dir().join(format!(
        "pi_agent_rust_native_bench_descriptor_{}.native.json",
        std::process::id()
    ));
    fs::write(&descriptor_path, NATIVE_RUNTIME_DESCRIPTOR).map_err(|err| {
        Error::extension(format!(
            "native workload: failed to write descriptor {}: {err}",
            descriptor_path.display()
        ))
    })?;

    let runtime = block_on(NativeRustExtensionRuntimeHandle::start())?;
    let manager = ExtensionManager::new();
    manager.set_runtime(ExtensionRuntimeHandle::NativeRust(runtime.clone()));

    let spec = NativeRustExtensionLoadSpec::from_entry_path(&*descriptor_path.to_string_lossy())?;
    block_on(manager.load_native_extensions(vec![spec]))?;
    Ok(ExtensionRuntimeHandle::NativeRust(runtime))
}

fn run_tool_roundtrip_quickjs(runtime: &PiJsRuntime) -> Result<()> {
    block_on(async {
        runtime.call_global_void(BENCH_BEGIN_FN).await?;
        let mut requests = runtime.drain_hostcall_requests();
        let request = requests
            .pop_front()
            .ok_or_else(|| Error::extension("bench workload: missing hostcall request"))?;
        if !requests.is_empty() {
            return Err(Error::extension(
                "bench workload: unexpected extra hostcall requests",
            ));
        }

        runtime.complete_hostcall(
            request.call_id,
            HostcallOutcome::Success(json!({"ok": true})),
        );
        runtime.tick().await?;
        runtime.call_global_void(BENCH_ASSERT_FN).await?;
        Ok(())
    })
}

fn run_tool_roundtrip_native(runtime: &mut NativeBenchRuntime) -> Result<()> {
    runtime.begin_roundtrip();
    let request = runtime.drain_hostcall_request()?;
    runtime.complete_hostcall(request.call_id, HostcallOutcome::Success(json!(true)))?;
    runtime.assert_roundtrip()
}

fn run_tool_roundtrip_native_runtime(runtime: &ExtensionRuntimeHandle) -> Result<()> {
    block_on(async {
        let output = runtime
            .execute_tool(
                NATIVE_RUNTIME_TOOL_NAME.to_string(),
                "bench-native-call".to_string(),
                json!({ "value": 1 }),
                Arc::new(json!({})),
                60_000,
            )
            .await?;
        if output.get("is_error").and_then(serde_json::Value::as_bool) == Some(false) {
            Ok(())
        } else {
            Err(Error::extension(format!(
                "native workload: runtime output indicates error: {output}"
            )))
        }
    })
}

#[cfg(test)]
mod tests {
    use pi::perf_build::profile_from_target_path;
    use std::path::Path;
    use std::time::Duration;

    use crate::{
        NativeBenchRuntime, run_tool_roundtrip_native, run_tool_roundtrip_native_runtime,
        setup_native_runtime_bench_handle,
    };

    #[test]
    fn profile_from_target_path_detects_perf() {
        let path = Path::new("/tmp/repo/target/perf/pijs_workload");
        assert_eq!(profile_from_target_path(path).as_deref(), Some("perf"));
    }

    #[test]
    fn profile_from_target_path_detects_release_deps_binary() {
        let path = Path::new("/tmp/repo/target/release/deps/pijs_workload-abc123");
        assert_eq!(profile_from_target_path(path).as_deref(), Some("release"));
    }

    #[test]
    fn profile_from_target_path_detects_target_triple_perf() {
        let path = Path::new("/tmp/repo/target/x86_64-unknown-linux-gnu/perf/pijs_workload");
        assert_eq!(profile_from_target_path(path).as_deref(), Some("perf"));
    }

    #[test]
    fn profile_from_target_path_detects_target_triple_perf_deps() {
        let path =
            Path::new("/tmp/repo/target/x86_64-unknown-linux-gnu/perf/deps/pijs_workload-abc123");
        assert_eq!(profile_from_target_path(path).as_deref(), Some("perf"));
    }

    #[test]
    fn profile_from_target_path_returns_none_outside_target() {
        let path = Path::new("/tmp/repo/bin/pijs_workload");
        assert_eq!(profile_from_target_path(path), None);
    }

    #[test]
    fn native_runtime_roundtrip_resolves() {
        let mut runtime = NativeBenchRuntime::default();
        run_tool_roundtrip_native(&mut runtime).expect("native runtime roundtrip");
    }

    #[test]
    fn native_runtime_handle_roundtrip_resolves() {
        let runtime = setup_native_runtime_bench_handle().expect("native runtime setup");
        run_tool_roundtrip_native_runtime(&runtime).expect("native runtime handle roundtrip");
        let _ = futures::executor::block_on(runtime.shutdown(Duration::from_secs(1)));
    }
}
