//! Tests for streaming hostcall delivery via the `QuickJS` bridge.
//!
//! These are integration tests, so they only use `PiJsRuntime`'s public API.
//! We validate streaming semantics by having JS report observed chunks back
//! to the host via `pi.tool("__report", ...)` hostcalls.

use pi::extensions_js::{HostcallKind, HostcallRequest, PiJsRuntime};
use pi::scheduler::{DeterministicClock, HostcallOutcome};
use serde_json::{Value, json};
use std::collections::VecDeque;

fn drain_one(runtime: &PiJsRuntime<DeterministicClock>) -> HostcallRequest {
    let mut queue = runtime.drain_hostcall_requests();
    queue
        .pop_front()
        .expect("expected a hostcall request to be queued")
}

fn assert_tool(req: &HostcallRequest, expected_name: &str) {
    match &req.kind {
        HostcallKind::Tool { name } => assert_eq!(name, expected_name),
        other => unreachable!("expected tool hostcall {expected_name}, got {other:?}"),
    }
}

const INTERLEAVED_ERROR_SCRIPT: &str = r#"
(async () => {
  (async () => {
    const stream = pi.exec("cmd-a", [], { stream: true });
    try {
      for await (const chunk of stream) {
        await pi.tool("__a_chunk", { chunk });
      }
      await pi.tool("__a_done", { ok: true });
    } catch (e) {
      await pi.tool("__a_err", { message: e.message, code: e.code ?? null });
    }
  })();

  (async () => {
    const result = await pi.exec("cmd-b", [], {
      stream: true,
      onChunk: (chunk, isFinal) => { pi.tool("__b_chunk", { chunk, isFinal }); },
    });
    await pi.tool("__b_done", { value: result });
  })();
})();
"#;

const INTERLEAVED_FINALIZATION_SCRIPT: &str = r#"
(async () => {
  (async () => {
    const result = await pi.exec("left-stream", [], {
      stream: true,
      onChunk: (chunk, isFinal) => { pi.tool("__left_chunk", { chunk, isFinal }); },
    });
    await pi.tool("__left_done", { value: result });
  })();

  (async () => {
    const result = await pi.exec("right-stream", [], {
      stream: true,
      onChunk: (chunk, isFinal) => { pi.tool("__right_chunk", { chunk, isFinal }); },
    });
    await pi.tool("__right_done", { value: result });
  })();
})();
"#;

fn exec_call_ids(
    requests: &VecDeque<HostcallRequest>,
    first_cmd: &str,
    second_cmd: &str,
) -> (String, String) {
    let mut first_call: Option<String> = None;
    let mut second_call: Option<String> = None;
    for request in requests {
        match &request.kind {
            HostcallKind::Exec { cmd } if cmd == first_cmd => {
                first_call = Some(request.call_id.clone());
            }
            HostcallKind::Exec { cmd } if cmd == second_cmd => {
                second_call = Some(request.call_id.clone());
            }
            other => unreachable!("unexpected hostcall kind in setup: {other:?}"),
        }
    }

    (
        first_call.expect("first call id"),
        second_call.expect("second call id"),
    )
}

fn complete_expected_tools_unordered(
    runtime: &PiJsRuntime<DeterministicClock>,
    context: &str,
    expected: &[(&str, Value)],
) {
    let mut requests = runtime.drain_hostcall_requests();
    assert_eq!(
        requests.len(),
        expected.len(),
        "{context}: unexpected number of tool calls",
    );

    let mut remaining: Vec<(&str, Value)> = expected.to_vec();
    while let Some(request) = requests.pop_front() {
        let tool_name = match &request.kind {
            HostcallKind::Tool { name } => name.as_str(),
            other => unreachable!("{context}: unexpected hostcall kind: {other:?}"),
        };
        let index = remaining
            .iter()
            .position(|(name, payload)| *name == tool_name && *payload == request.payload)
            .unwrap_or_else(|| {
                panic!(
                    "{context}: unexpected tool payload for {tool_name}: {}",
                    request.payload
                )
            });
        remaining.remove(index);
        runtime.complete_hostcall(request.call_id, HostcallOutcome::Success(Value::Null));
    }

    assert!(
        remaining.is_empty(),
        "{context}: missing expected tool callbacks",
    );
}

#[allow(clippy::future_not_send)]
async fn emit_left_nonfinal_chunk_and_ack(
    runtime: &PiJsRuntime<DeterministicClock>,
    left_call: &str,
) {
    runtime.complete_hostcall(
        left_call.to_string(),
        HostcallOutcome::StreamChunk {
            sequence: 0,
            chunk: json!("left-0"),
            is_final: false,
        },
    );
    runtime.tick().await.expect("tick left-0");

    let left_chunk_req = drain_one(runtime);
    assert_tool(&left_chunk_req, "__left_chunk");
    assert_eq!(
        left_chunk_req.payload,
        json!({ "chunk": "left-0", "isFinal": false })
    );
    runtime.complete_hostcall(
        left_chunk_req.call_id,
        HostcallOutcome::Success(Value::Null),
    );
    runtime.tick().await.expect("tick __left_chunk");
}

#[allow(clippy::future_not_send)]
async fn emit_b_nonfinal_chunk_and_ack(runtime: &PiJsRuntime<DeterministicClock>, call_b: &str) {
    runtime.complete_hostcall(
        call_b.to_string(),
        HostcallOutcome::StreamChunk {
            sequence: 0,
            chunk: json!("b0"),
            is_final: false,
        },
    );
    runtime.tick().await.expect("tick b0");

    let b0_req = drain_one(runtime);
    assert_tool(&b0_req, "__b_chunk");
    assert_eq!(b0_req.payload, json!({ "chunk": "b0", "isFinal": false }));
    runtime.complete_hostcall(b0_req.call_id, HostcallOutcome::Success(Value::Null));
    runtime.tick().await.expect("tick b0 report");
}

#[allow(clippy::future_not_send)]
async fn emit_a_error_and_ack(runtime: &PiJsRuntime<DeterministicClock>, call_a: &str) {
    runtime.complete_hostcall(
        call_a.to_string(),
        HostcallOutcome::Error {
            code: "io".to_string(),
            message: "stream-a failed".to_string(),
        },
    );
    runtime.tick().await.expect("tick stream-a error");

    let a_err_req = drain_one(runtime);
    assert_tool(&a_err_req, "__a_err");
    assert_eq!(
        a_err_req.payload,
        json!({ "message": "stream-a failed", "code": "io" })
    );
    runtime.complete_hostcall(a_err_req.call_id, HostcallOutcome::Success(Value::Null));
    runtime.tick().await.expect("tick __a_err completion");
}

#[test]
fn streaming_async_iterator_delivers_chunks() {
    futures::executor::block_on(async {
        let runtime = PiJsRuntime::with_clock(DeterministicClock::new(0))
            .await
            .expect("create runtime");

        runtime
            .eval(
                r#"
globalThis.done = false;
(async () => {
  const stream = pi.exec("tail", ["-f", "/dev/null"], { stream: true });
  for await (const chunk of stream) {
    await pi.tool("__report", { chunk });
  }
  await pi.tool("__done", { ok: true });
  globalThis.done = true;
})();
"#,
            )
            .await
            .expect("eval");

        // Initial hostcall: streaming exec.
        let exec_req = drain_one(&runtime);
        let exec_call_id = exec_req.call_id.clone();

        // Chunk 0
        runtime.complete_hostcall(
            exec_call_id.clone(),
            HostcallOutcome::StreamChunk {
                sequence: 0,
                chunk: json!("line 1\n"),
                is_final: false,
            },
        );
        runtime.tick().await.expect("tick chunk 0");

        let report_req = drain_one(&runtime);
        assert_tool(&report_req, "__report");
        assert_eq!(report_req.payload, json!({ "chunk": "line 1\n" }));
        runtime.complete_hostcall(report_req.call_id, HostcallOutcome::Success(Value::Null));
        runtime.tick().await.expect("tick report 0");

        // Chunk 1
        runtime.complete_hostcall(
            exec_call_id.clone(),
            HostcallOutcome::StreamChunk {
                sequence: 1,
                chunk: json!("line 2\n"),
                is_final: false,
            },
        );
        runtime.tick().await.expect("tick chunk 1");

        let report_req = drain_one(&runtime);
        assert_tool(&report_req, "__report");
        assert_eq!(report_req.payload, json!({ "chunk": "line 2\n" }));
        runtime.complete_hostcall(report_req.call_id, HostcallOutcome::Success(Value::Null));
        runtime.tick().await.expect("tick report 1");

        // Final chunk: end-of-stream signal (is_final + null).
        runtime.complete_hostcall(
            exec_call_id,
            HostcallOutcome::StreamChunk {
                sequence: 2,
                chunk: Value::Null,
                is_final: true,
            },
        );
        runtime.tick().await.expect("tick final");

        // The loop should terminate and emit a "__done" tool call.
        let done_req = drain_one(&runtime);
        assert_tool(&done_req, "__done");
        assert_eq!(done_req.payload, json!({ "ok": true }));
        runtime.complete_hostcall(done_req.call_id, HostcallOutcome::Success(Value::Null));
        runtime.tick().await.expect("tick done");

        assert!(runtime.drain_hostcall_requests().is_empty());
    });
}

#[test]
fn streaming_error_mid_stream_reports_error() {
    futures::executor::block_on(async {
        let runtime = PiJsRuntime::with_clock(DeterministicClock::new(0))
            .await
            .expect("create runtime");

        runtime
            .eval(
                r#"
(async () => {
  const stream = pi.exec("cat", ["bigfile"], { stream: true });
  try {
    for await (const chunk of stream) {
      await pi.tool("__report", { chunk });
    }
  } catch (e) {
    await pi.tool("__report_error", { message: e.message, code: e.code ?? null });
  }
})();
"#,
            )
            .await
            .expect("eval");

        let exec_req = drain_one(&runtime);
        let exec_call_id = exec_req.call_id.clone();

        runtime.complete_hostcall(
            exec_call_id.clone(),
            HostcallOutcome::StreamChunk {
                sequence: 0,
                chunk: json!("partial"),
                is_final: false,
            },
        );
        runtime.tick().await.expect("tick chunk");

        let report_req = drain_one(&runtime);
        assert_tool(&report_req, "__report");
        assert_eq!(report_req.payload, json!({ "chunk": "partial" }));
        runtime.complete_hostcall(report_req.call_id, HostcallOutcome::Success(Value::Null));
        runtime.tick().await.expect("tick report");

        runtime.complete_hostcall(
            exec_call_id,
            HostcallOutcome::Error {
                code: "io".to_string(),
                message: "connection reset".to_string(),
            },
        );
        runtime.tick().await.expect("tick error");

        let err_req = drain_one(&runtime);
        assert_tool(&err_req, "__report_error");
        assert_eq!(
            err_req.payload,
            json!({ "message": "connection reset", "code": "io" })
        );
        runtime.complete_hostcall(err_req.call_id, HostcallOutcome::Success(Value::Null));
        runtime.tick().await.expect("tick report_error");

        assert!(runtime.drain_hostcall_requests().is_empty());
    });
}

#[test]
fn streaming_nonfinal_keeps_call_pending() {
    futures::executor::block_on(async {
        let runtime = PiJsRuntime::with_clock(DeterministicClock::new(0))
            .await
            .expect("create runtime");

        runtime
            .eval(r#"pi.exec("ls", [], { stream: true });"#)
            .await
            .expect("eval");

        let exec_req = drain_one(&runtime);
        let exec_call_id = exec_req.call_id.clone();
        assert_eq!(runtime.pending_hostcall_count(), 1);

        runtime.complete_hostcall(
            exec_call_id.clone(),
            HostcallOutcome::StreamChunk {
                sequence: 0,
                chunk: json!("data"),
                is_final: false,
            },
        );
        runtime.tick().await.expect("tick nonfinal");
        assert_eq!(runtime.pending_hostcall_count(), 1);

        runtime.complete_hostcall(
            exec_call_id,
            HostcallOutcome::StreamChunk {
                sequence: 1,
                chunk: Value::Null,
                is_final: true,
            },
        );
        runtime.tick().await.expect("tick final");
        assert_eq!(runtime.pending_hostcall_count(), 0);
    });
}

#[test]
fn streaming_callback_receives_chunks() {
    futures::executor::block_on(async {
        let runtime = PiJsRuntime::with_clock(DeterministicClock::new(0))
            .await
            .expect("create runtime");

        runtime
            .eval(
                r#"
(async () => {
  const result = await pi.exec("echo", ["hello"], {
    stream: true,
    onChunk: (chunk, isFinal) => { pi.tool("__report", { chunk, isFinal }); },
  });
  await pi.tool("__resolved", { value: result });
})();
"#,
            )
            .await
            .expect("eval");

        let exec_req = drain_one(&runtime);
        let call_id = exec_req.call_id.clone();

        runtime.complete_hostcall(
            call_id.clone(),
            HostcallOutcome::StreamChunk {
                sequence: 0,
                chunk: json!("chunk-A"),
                is_final: false,
            },
        );
        runtime.tick().await.expect("tick chunk-A");

        let report_req = drain_one(&runtime);
        assert_tool(&report_req, "__report");
        assert_eq!(
            report_req.payload,
            json!({ "chunk": "chunk-A", "isFinal": false })
        );
        runtime.complete_hostcall(report_req.call_id, HostcallOutcome::Success(Value::Null));
        runtime.tick().await.expect("tick report chunk-A");

        runtime.complete_hostcall(
            call_id,
            HostcallOutcome::StreamChunk {
                sequence: 1,
                chunk: json!("chunk-B"),
                is_final: true,
            },
        );
        runtime.tick().await.expect("tick chunk-B");

        let mut reqs = runtime.drain_hostcall_requests();
        assert_eq!(reqs.len(), 2, "expected report + resolved tool calls");

        // Order is deterministic (hostcall queue is FIFO), but don't over-assume.
        let mut seen_report = false;
        let mut seen_resolved = false;

        while let Some(req) = reqs.pop_front() {
            match &req.kind {
                HostcallKind::Tool { name } if name == "__report" => {
                    assert_eq!(req.payload, json!({ "chunk": "chunk-B", "isFinal": true }));
                    seen_report = true;
                }
                HostcallKind::Tool { name } if name == "__resolved" => {
                    assert_eq!(req.payload, json!({ "value": "chunk-B" }));
                    seen_resolved = true;
                }
                other => unreachable!("unexpected hostcall after final chunk: {other:?}"),
            }
            runtime.complete_hostcall(req.call_id, HostcallOutcome::Success(Value::Null));
        }

        assert!(seen_report, "missing __report tool call");
        assert!(seen_resolved, "missing __resolved tool call");

        // Deliver both tool completions.
        runtime.tick().await.expect("tick tool completion 1");
        runtime.tick().await.expect("tick tool completion 2");

        assert!(runtime.drain_hostcall_requests().is_empty());
    });
}

#[test]
fn streaming_interleaved_error_does_not_stall_sibling_callback_stream() {
    futures::executor::block_on(async {
        let runtime = PiJsRuntime::with_clock(DeterministicClock::new(0))
            .await
            .expect("create runtime");

        runtime.eval(INTERLEAVED_ERROR_SCRIPT).await.expect("eval");

        let requests = runtime.drain_hostcall_requests();
        assert_eq!(requests.len(), 2, "expected two streaming exec requests");

        let (call_a, call_b) = exec_call_ids(&requests, "cmd-a", "cmd-b");
        assert_ne!(
            call_a, call_b,
            "interleaved streams must have distinct call ids"
        );
        assert_eq!(runtime.pending_hostcall_count(), 2);
        assert!(runtime.is_hostcall_pending(&call_a));
        assert!(runtime.is_hostcall_pending(&call_b));

        emit_b_nonfinal_chunk_and_ack(&runtime, &call_b).await;
        assert_eq!(runtime.pending_hostcall_count(), 2);

        emit_a_error_and_ack(&runtime, &call_a).await;

        assert_eq!(
            runtime.pending_hostcall_count(),
            1,
            "stream-a error must not clear sibling stream-b pending state"
        );
        assert!(!runtime.is_hostcall_pending(&call_a));
        assert!(runtime.is_hostcall_pending(&call_b));

        runtime.complete_hostcall(
            call_b.clone(),
            HostcallOutcome::StreamChunk {
                sequence: 1,
                chunk: json!("b1"),
                is_final: true,
            },
        );
        runtime.tick().await.expect("tick b1 final");
        // Stream-b exec hostcall is complete; JS callbacks queued tool requests
        // (__b_chunk + __b_done) that inflate pending_hostcall_count.
        assert!(!runtime.is_hostcall_pending(&call_b));

        complete_expected_tools_unordered(
            &runtime,
            "stream-b finalization",
            &[
                ("__b_chunk", json!({ "chunk": "b1", "isFinal": true })),
                ("__b_done", json!({ "value": "b1" })),
            ],
        );

        runtime
            .tick()
            .await
            .expect("tick post-final tool completion 1");
        runtime
            .tick()
            .await
            .expect("tick post-final tool completion 2");

        assert!(runtime.drain_hostcall_requests().is_empty());
    });
}

#[test]
fn streaming_interleaved_finalization_preserves_pending_count_invariants() {
    futures::executor::block_on(async {
        let runtime = PiJsRuntime::with_clock(DeterministicClock::new(0))
            .await
            .expect("create runtime");

        runtime
            .eval(INTERLEAVED_FINALIZATION_SCRIPT)
            .await
            .expect("eval");

        let requests = runtime.drain_hostcall_requests();
        assert_eq!(requests.len(), 2, "expected two streaming exec requests");

        let (left_call, right_call) = exec_call_ids(&requests, "left-stream", "right-stream");
        assert_ne!(
            left_call, right_call,
            "concurrent streams must have distinct call ids"
        );
        assert_eq!(runtime.pending_hostcall_count(), 2);

        emit_left_nonfinal_chunk_and_ack(&runtime, &left_call).await;
        assert_eq!(runtime.pending_hostcall_count(), 2);

        runtime.complete_hostcall(
            right_call.clone(),
            HostcallOutcome::StreamChunk {
                sequence: 0,
                chunk: json!("right-final"),
                is_final: true,
            },
        );
        runtime.tick().await.expect("tick right final");
        // Right stream exec is complete; left stream still pending.
        // JS callbacks queued tool requests (__right_chunk + __right_done) that
        // inflate pending_hostcall_count beyond the 1 remaining exec stream.
        assert!(runtime.is_hostcall_pending(&left_call));
        assert!(!runtime.is_hostcall_pending(&right_call));

        complete_expected_tools_unordered(
            &runtime,
            "right stream finalization",
            &[
                (
                    "__right_chunk",
                    json!({ "chunk": "right-final", "isFinal": true }),
                ),
                ("__right_done", json!({ "value": "right-final" })),
            ],
        );

        runtime.tick().await.expect("tick right completion 1");
        runtime.tick().await.expect("tick right completion 2");
        // Left stream still pending as the only exec hostcall.
        assert!(runtime.is_hostcall_pending(&left_call));

        runtime.complete_hostcall(
            left_call.clone(),
            HostcallOutcome::StreamChunk {
                sequence: 1,
                chunk: json!("left-final"),
                is_final: true,
            },
        );
        runtime.tick().await.expect("tick left final");
        // Left stream exec is now complete; JS callbacks queue tool requests.
        assert!(!runtime.is_hostcall_pending(&left_call));

        complete_expected_tools_unordered(
            &runtime,
            "left stream finalization",
            &[
                (
                    "__left_chunk",
                    json!({ "chunk": "left-final", "isFinal": true }),
                ),
                ("__left_done", json!({ "value": "left-final" })),
            ],
        );

        runtime.tick().await.expect("tick left completion 1");
        runtime.tick().await.expect("tick left completion 2");

        assert_eq!(runtime.pending_hostcall_count(), 0);
        assert!(runtime.drain_hostcall_requests().is_empty());
    });
}
