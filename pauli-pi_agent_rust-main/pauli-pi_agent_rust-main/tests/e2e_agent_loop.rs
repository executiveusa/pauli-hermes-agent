//! E2E: full agent loop integration tests (bd-2q00).
//!
//! These tests run the real `AgentSession` + `ToolRegistry` loop end-to-end
//! with a deterministic in-process provider script. No network, no fixture
//! playback, and no mock HTTP servers.

mod common;

use async_trait::async_trait;
use common::{TestHarness, run_async};
use futures::Stream;
use pi::agent::{Agent, AgentConfig, AgentEvent, AgentSession};
use pi::compaction::ResolvedCompactionSettings;
use pi::error::{Error, Result};
use pi::model::{
    AssistantMessage, ContentBlock, Message, StopReason, StreamEvent, TextContent, ToolCall,
    ToolResultMessage, Usage,
};
use pi::provider::{Context, Provider, StreamOptions};
use pi::session::Session;
use pi::tools::ToolRegistry;
use serde_json::json;
use std::collections::BTreeMap;
use std::io::Write as _;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Debug, Clone)]
enum Scenario {
    SimpleConversation,
    ToolRoundTrip {
        read_path: String,
        expected_fragment: String,
    },
    MultiTool {
        file_path: String,
        content: String,
    },
    BashTool,
    ErrorRecovery,
}

#[derive(Debug)]
struct ScriptedProvider {
    scenario: Scenario,
    stream_calls: AtomicUsize,
}

impl ScriptedProvider {
    const fn new(scenario: Scenario) -> Self {
        Self {
            scenario,
            stream_calls: AtomicUsize::new(0),
        }
    }

    fn assistant_message(
        &self,
        stop_reason: StopReason,
        content: Vec<ContentBlock>,
        total_tokens: u64,
    ) -> AssistantMessage {
        AssistantMessage {
            content,
            api: self.api().to_string(),
            provider: self.name().to_string(),
            model: self.model_id().to_string(),
            usage: Usage {
                total_tokens,
                output: total_tokens,
                ..Usage::default()
            },
            stop_reason,
            error_message: None,
            timestamp: 0,
        }
    }

    fn stream_done(
        &self,
        message: AssistantMessage,
    ) -> Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send>> {
        let partial = self.assistant_message(StopReason::Stop, Vec::new(), 0);
        Box::pin(futures::stream::iter(vec![
            Ok(StreamEvent::Start { partial }),
            Ok(StreamEvent::Done {
                reason: message.stop_reason,
                message,
            }),
        ]))
    }

    fn context_tool_results<'a>(context: &'a Context<'a>) -> Vec<&'a ToolResultMessage> {
        context
            .messages
            .iter()
            .filter_map(|message| match message {
                Message::ToolResult(result) => Some(result.as_ref()),
                _ => None,
            })
            .collect()
    }
}

#[async_trait]
#[allow(clippy::unnecessary_literal_bound)]
#[allow(clippy::too_many_lines)]
impl Provider for ScriptedProvider {
    fn name(&self) -> &str {
        "scripted-provider"
    }

    fn api(&self) -> &str {
        "scripted-api"
    }

    fn model_id(&self) -> &str {
        "scripted-model"
    }

    async fn stream(
        &self,
        context: &Context<'_>,
        _options: &StreamOptions,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send>>> {
        let call_index = self.stream_calls.fetch_add(1, Ordering::SeqCst);

        match &self.scenario {
            Scenario::SimpleConversation => {
                if call_index > 0 {
                    return Err(Error::api(
                        "simple_conversation expected exactly one provider call",
                    ));
                }
                Ok(self.stream_done(self.assistant_message(
                    StopReason::Stop,
                    vec![ContentBlock::Text(TextContent::new(
                        "hello from scripted provider",
                    ))],
                    12,
                )))
            }
            Scenario::ToolRoundTrip {
                read_path,
                expected_fragment,
            } => {
                if call_index == 0 {
                    return Ok(self.stream_done(self.assistant_message(
                        StopReason::ToolUse,
                        vec![ContentBlock::ToolCall(ToolCall {
                            id: "read-1".to_string(),
                            name: "read".to_string(),
                            arguments: json!({ "path": read_path }),
                            thought_signature: None,
                        })],
                        30,
                    )));
                }
                if call_index == 1 {
                    let results = Self::context_tool_results(context);
                    let Some(result) = results
                        .iter()
                        .rev()
                        .copied()
                        .find(|r| r.tool_call_id == "read-1")
                    else {
                        return Err(Error::api("tool_round_trip expected read-1 tool result"));
                    };
                    let response_content = result
                        .content
                        .iter()
                        .filter_map(|block| match block {
                            ContentBlock::Text(text) => Some(text.text.as_str()),
                            _ => None,
                        })
                        .collect::<String>();
                    if !response_content.contains(expected_fragment) {
                        return Err(Error::api("tool_round_trip missing expected read output"));
                    }
                    return Ok(self.stream_done(self.assistant_message(
                        StopReason::Stop,
                        vec![ContentBlock::Text(TextContent::new(
                            "package name detected: pi-agent-rust",
                        ))],
                        24,
                    )));
                }
                Err(Error::api(
                    "tool_round_trip received unexpected provider call",
                ))
            }
            Scenario::MultiTool { file_path, content } => {
                // Turn 0: write the file
                if call_index == 0 {
                    return Ok(self.stream_done(self.assistant_message(
                        StopReason::ToolUse,
                        vec![ContentBlock::ToolCall(ToolCall {
                            id: "write-1".to_string(),
                            name: "write".to_string(),
                            arguments: json!({
                                "path": file_path,
                                "content": content,
                            }),
                            thought_signature: None,
                        })],
                        40,
                    )));
                }
                // Turn 1: verify write succeeded, then read the file back
                if call_index == 1 {
                    let results = Self::context_tool_results(context);
                    let Some(write_result) = results
                        .iter()
                        .rev()
                        .copied()
                        .find(|r| r.tool_call_id == "write-1")
                    else {
                        return Err(Error::api("multi_tool expected write-1 result"));
                    };
                    if write_result.is_error {
                        return Err(Error::api("multi_tool expected successful write result"));
                    }
                    return Ok(self.stream_done(self.assistant_message(
                        StopReason::ToolUse,
                        vec![ContentBlock::ToolCall(ToolCall {
                            id: "read-1".to_string(),
                            name: "read".to_string(),
                            arguments: json!({ "path": file_path }),
                            thought_signature: None,
                        })],
                        30,
                    )));
                }
                // Turn 2: verify read succeeded and contains expected content
                if call_index == 2 {
                    let results = Self::context_tool_results(context);
                    let Some(read_result) = results
                        .iter()
                        .rev()
                        .copied()
                        .find(|r| r.tool_call_id == "read-1")
                    else {
                        return Err(Error::api("multi_tool expected read-1 result"));
                    };
                    if read_result.is_error {
                        return Err(Error::api("multi_tool expected successful read result"));
                    }
                    let read_text = read_result
                        .content
                        .iter()
                        .filter_map(|block| match block {
                            ContentBlock::Text(text) => Some(text.text.as_str()),
                            _ => None,
                        })
                        .collect::<String>();
                    if !read_text.contains(content) {
                        return Err(Error::api(
                            "multi_tool read output missing expected content",
                        ));
                    }
                    return Ok(self.stream_done(self.assistant_message(
                        StopReason::Stop,
                        vec![ContentBlock::Text(TextContent::new("multi-tool complete"))],
                        26,
                    )));
                }
                Err(Error::api("multi_tool received unexpected provider call"))
            }
            Scenario::BashTool => {
                if call_index == 0 {
                    return Ok(self.stream_done(self.assistant_message(
                        StopReason::ToolUse,
                        vec![ContentBlock::ToolCall(ToolCall {
                            id: "bash-1".to_string(),
                            name: "bash".to_string(),
                            arguments: json!({ "command": "echo hello-agent-loop" }),
                            thought_signature: None,
                        })],
                        32,
                    )));
                }
                if call_index == 1 {
                    let results = Self::context_tool_results(context);
                    let Some(result) = results
                        .iter()
                        .rev()
                        .copied()
                        .find(|r| r.tool_call_id == "bash-1")
                    else {
                        return Err(Error::api("bash_tool_e2e expected bash-1 tool result"));
                    };
                    let text = result
                        .content
                        .iter()
                        .filter_map(|block| match block {
                            ContentBlock::Text(text) => Some(text.text.as_str()),
                            _ => None,
                        })
                        .collect::<String>();
                    if !text.contains("hello-agent-loop") {
                        return Err(Error::api(
                            "bash_tool_e2e output missing expected echo content",
                        ));
                    }
                    return Ok(self.stream_done(self.assistant_message(
                        StopReason::Stop,
                        vec![ContentBlock::Text(TextContent::new(
                            "bash output verified: hello-agent-loop",
                        ))],
                        20,
                    )));
                }
                Err(Error::api(
                    "bash_tool_e2e received unexpected provider call",
                ))
            }
            Scenario::ErrorRecovery => {
                if call_index == 0 {
                    return Ok(self.stream_done(self.assistant_message(
                        StopReason::ToolUse,
                        vec![ContentBlock::ToolCall(ToolCall {
                            id: "bad-1".to_string(),
                            name: "missing_tool".to_string(),
                            arguments: json!({}),
                            thought_signature: None,
                        })],
                        18,
                    )));
                }
                if call_index == 1 {
                    let results = Self::context_tool_results(context);
                    let Some(result) = results
                        .iter()
                        .rev()
                        .copied()
                        .find(|r| r.tool_call_id == "bad-1")
                    else {
                        return Err(Error::api("error_recovery expected bad-1 tool result"));
                    };
                    if !result.is_error {
                        return Err(Error::api(
                            "error_recovery expected tool result marked error",
                        ));
                    }
                    let text = result
                        .content
                        .iter()
                        .filter_map(|block| match block {
                            ContentBlock::Text(text) => Some(text.text.as_str()),
                            _ => None,
                        })
                        .collect::<String>();
                    if !text.contains("not found") {
                        return Err(Error::api("error_recovery expected not found message"));
                    }
                    return Ok(self.stream_done(self.assistant_message(
                        StopReason::Stop,
                        vec![ContentBlock::Text(TextContent::new(
                            "recovered after invalid tool call",
                        ))],
                        16,
                    )));
                }
                Err(Error::api(
                    "error_recovery received unexpected provider call",
                ))
            }
        }
    }
}

#[derive(Debug, Default)]
struct EventCapture {
    timeline: Vec<serde_json::Value>,
    turn_starts: BTreeMap<usize, Instant>,
    turn_durations_ms: BTreeMap<usize, u128>,
    tool_starts: usize,
    tool_ends: usize,
}

#[derive(Debug)]
struct RunOutcome {
    message: AssistantMessage,
    capture: EventCapture,
    total_tokens: u64,
}

const fn event_label(event: &AgentEvent) -> &'static str {
    match event {
        AgentEvent::AgentStart { .. } => "agent_start",
        AgentEvent::AgentEnd { .. } => "agent_end",
        AgentEvent::TurnStart { .. } => "turn_start",
        AgentEvent::TurnEnd { .. } => "turn_end",
        AgentEvent::MessageStart { .. } => "message_start",
        AgentEvent::MessageUpdate { .. } => "message_update",
        AgentEvent::MessageEnd { .. } => "message_end",
        AgentEvent::ToolExecutionStart { .. } => "tool_start",
        AgentEvent::ToolExecutionUpdate { .. } => "tool_update",
        AgentEvent::ToolExecutionEnd { .. } => "tool_end",
        AgentEvent::AutoCompactionStart { .. } => "auto_compaction_start",
        AgentEvent::AutoCompactionEnd { .. } => "auto_compaction_end",
        AgentEvent::AutoRetryStart { .. } => "auto_retry_start",
        AgentEvent::AutoRetryEnd { .. } => "auto_retry_end",
        AgentEvent::ExtensionError { .. } => "extension_error",
    }
}

fn assistant_text(message: &AssistantMessage) -> String {
    message
        .content
        .iter()
        .filter_map(|block| match block {
            ContentBlock::Text(text) => Some(text.text.as_str()),
            _ => None,
        })
        .collect::<String>()
}

const fn tool_names() -> [&'static str; 7] {
    ["read", "write", "edit", "bash", "grep", "find", "ls"]
}

fn total_assistant_tokens(messages: &[Message]) -> u64 {
    messages
        .iter()
        .filter_map(|message| match message {
            Message::Assistant(assistant) => Some(assistant.usage.total_tokens),
            _ => None,
        })
        .sum()
}

fn write_timeline_artifact(harness: &TestHarness, test_name: &str, capture: &EventCapture) {
    let timeline_path = harness.temp_path(format!("{test_name}.timeline.jsonl"));
    let mut file = std::fs::File::create(&timeline_path).expect("create timeline artifact");
    for entry in &capture.timeline {
        let line = serde_json::to_string(entry).expect("serialize timeline entry");
        let _ = writeln!(file, "{line}");
    }
    harness.record_artifact(format!("{test_name}.timeline.jsonl"), &timeline_path);
}

fn write_jsonl_artifacts(harness: &TestHarness, test_name: &str) {
    let log_path = harness.temp_path(format!("{test_name}.log.jsonl"));
    harness
        .write_jsonl_logs(&log_path)
        .expect("write jsonl log");
    harness.record_artifact(format!("{test_name}.log.jsonl"), &log_path);

    let normalized_log_path = harness.temp_path(format!("{test_name}.log.normalized.jsonl"));
    harness
        .write_jsonl_logs_normalized(&normalized_log_path)
        .expect("write normalized jsonl log");
    harness.record_artifact(
        format!("{test_name}.log.normalized.jsonl"),
        &normalized_log_path,
    );

    let artifacts_path = harness.temp_path(format!("{test_name}.artifacts.jsonl"));
    harness
        .write_artifact_index_jsonl(&artifacts_path)
        .expect("write artifact index jsonl");
    harness.record_artifact(format!("{test_name}.artifacts.jsonl"), &artifacts_path);
}

fn run_scenario(
    harness: &TestHarness,
    scenario: Scenario,
    user_prompt: &str,
    max_tool_iterations: usize,
) -> RunOutcome {
    let cwd = harness.temp_dir().to_path_buf();
    let user_prompt = user_prompt.to_string();
    run_async(async move {
        let provider: Arc<dyn Provider> = Arc::new(ScriptedProvider::new(scenario));
        let tools = ToolRegistry::new(&tool_names(), &cwd, None);
        let config = AgentConfig {
            system_prompt: None,
            max_tool_iterations,
            stream_options: StreamOptions {
                api_key: Some("test-key".to_string()),
                ..StreamOptions::default()
            },
            block_images: false,
        };
        let agent = Agent::new(provider, tools, config);
        let session = Arc::new(asupersync::sync::Mutex::new(Session::create_with_dir(
            Some(cwd.clone()),
        )));
        let mut agent_session = AgentSession::new(
            agent,
            Arc::clone(&session),
            true,
            ResolvedCompactionSettings::default(),
        );

        let started_at = Instant::now();
        let capture = Arc::new(Mutex::new(EventCapture::default()));
        let capture_ref = Arc::clone(&capture);
        let message = agent_session
            .run_text(user_prompt, move |event| {
                let elapsed_ms = started_at.elapsed().as_millis();
                let mut guard = capture_ref.lock().expect("lock event capture");
                match &event {
                    AgentEvent::TurnStart { turn_index, .. } => {
                        guard.turn_starts.insert(*turn_index, Instant::now());
                    }
                    AgentEvent::TurnEnd { turn_index, .. } => {
                        if let Some(start) = guard.turn_starts.remove(turn_index) {
                            guard
                                .turn_durations_ms
                                .insert(*turn_index, start.elapsed().as_millis());
                        }
                    }
                    AgentEvent::ToolExecutionStart { .. } => {
                        guard.tool_starts += 1;
                    }
                    AgentEvent::ToolExecutionEnd { .. } => {
                        guard.tool_ends += 1;
                    }
                    _ => {}
                }
                guard.timeline.push(json!({
                    "event": event_label(&event),
                    "elapsedMs": elapsed_ms,
                }));
                drop(guard);
            })
            .await
            .expect("run agent scenario");

        agent_session
            .persist_session()
            .await
            .expect("persist session");

        let messages = {
            let cx = asupersync::Cx::for_testing();
            let guard = session.lock(&cx).await.expect("lock session");
            guard.to_messages_for_current_path()
        };
        let capture = Arc::try_unwrap(capture)
            .expect("single capture owner")
            .into_inner()
            .expect("extract event capture");
        RunOutcome {
            message,
            capture,
            total_tokens: total_assistant_tokens(&messages),
        }
    })
}

#[test]
fn simple_conversation() {
    let test_name = "e2e_agent_loop_simple_conversation";
    let harness = TestHarness::new(test_name);

    let outcome = run_scenario(&harness, Scenario::SimpleConversation, "Say hello.", 4);

    assert_eq!(outcome.message.stop_reason, StopReason::Stop);
    assert!(assistant_text(&outcome.message).contains("hello"));
    assert_eq!(outcome.capture.tool_starts, 0);
    assert_eq!(outcome.capture.tool_ends, 0);
    assert!(outcome.total_tokens > 0);

    harness
        .log()
        .info_ctx("summary", "simple_conversation complete", |ctx| {
            ctx.push((
                "turn_count".into(),
                outcome.capture.turn_durations_ms.len().to_string(),
            ));
            ctx.push(("total_tokens".into(), outcome.total_tokens.to_string()));
        });
    write_timeline_artifact(&harness, test_name, &outcome.capture);
    write_jsonl_artifacts(&harness, test_name);
}

#[test]
fn tool_round_trip() {
    let test_name = "e2e_agent_loop_tool_round_trip";
    let harness = TestHarness::new(test_name);

    let fixture = harness.create_file("fixtures/package.txt", "package_name=pi-agent-rust\n");
    let outcome = run_scenario(
        &harness,
        Scenario::ToolRoundTrip {
            read_path: fixture.display().to_string(),
            expected_fragment: "package_name=pi-agent-rust".to_string(),
        },
        "Read the file and report the package name.",
        4,
    );

    assert_eq!(outcome.message.stop_reason, StopReason::Stop);
    assert!(assistant_text(&outcome.message).contains("pi-agent-rust"));
    assert_eq!(outcome.capture.tool_starts, 1);
    assert_eq!(outcome.capture.tool_ends, 1);
    assert!(outcome.total_tokens >= outcome.message.usage.total_tokens);

    harness
        .log()
        .info_ctx("summary", "tool_round_trip complete", |ctx| {
            ctx.push((
                "turn_count".into(),
                outcome.capture.turn_durations_ms.len().to_string(),
            ));
            ctx.push(("total_tokens".into(), outcome.total_tokens.to_string()));
        });
    write_timeline_artifact(&harness, test_name, &outcome.capture);
    write_jsonl_artifacts(&harness, test_name);
}

#[test]
fn multi_tool() {
    let test_name = "e2e_agent_loop_multi_tool";
    let harness = TestHarness::new(test_name);

    let multi_path = harness.temp_path("workspace/multi_tool.txt");
    let outcome = run_scenario(
        &harness,
        Scenario::MultiTool {
            file_path: multi_path.display().to_string(),
            content: "alpha-beta-gamma".to_string(),
        },
        "Write then read a file and summarize.",
        8,
    );

    assert_eq!(outcome.message.stop_reason, StopReason::Stop);
    assert!(assistant_text(&outcome.message).contains("multi-tool complete"));
    // Write and read are now in separate turns (parallel execution means
    // dependent tools must be in different turns).
    assert_eq!(outcome.capture.tool_starts, 2);
    assert_eq!(outcome.capture.tool_ends, 2);
    assert!(outcome.total_tokens > 0);

    harness
        .log()
        .info_ctx("summary", "multi_tool complete", |ctx| {
            ctx.push((
                "turn_count".into(),
                outcome.capture.turn_durations_ms.len().to_string(),
            ));
            ctx.push(("total_tokens".into(), outcome.total_tokens.to_string()));
        });
    write_timeline_artifact(&harness, test_name, &outcome.capture);
    write_jsonl_artifacts(&harness, test_name);
}

#[test]
fn bash_tool_e2e() {
    let test_name = "e2e_agent_loop_bash_tool";
    let harness = TestHarness::new(test_name);

    let outcome = run_scenario(
        &harness,
        Scenario::BashTool,
        "Run a bash command and report the output.",
        4,
    );

    assert_eq!(outcome.message.stop_reason, StopReason::Stop);
    assert!(assistant_text(&outcome.message).contains("hello-agent-loop"));
    assert_eq!(outcome.capture.tool_starts, 1);
    assert_eq!(outcome.capture.tool_ends, 1);
    assert!(outcome.total_tokens > 0);

    harness
        .log()
        .info_ctx("summary", "bash_tool_e2e complete", |ctx| {
            ctx.push((
                "turn_count".into(),
                outcome.capture.turn_durations_ms.len().to_string(),
            ));
            ctx.push(("total_tokens".into(), outcome.total_tokens.to_string()));
        });
    write_timeline_artifact(&harness, test_name, &outcome.capture);
    write_jsonl_artifacts(&harness, test_name);
}

#[test]
fn error_recovery() {
    let test_name = "e2e_agent_loop_error_recovery";
    let harness = TestHarness::new(test_name);

    let outcome = run_scenario(
        &harness,
        Scenario::ErrorRecovery,
        "Call an invalid tool and then recover.",
        4,
    );

    assert_eq!(outcome.message.stop_reason, StopReason::Stop);
    assert!(assistant_text(&outcome.message).contains("recovered"));
    assert_eq!(outcome.capture.tool_starts, 1);
    assert_eq!(outcome.capture.tool_ends, 1);
    assert!(outcome.total_tokens > 0);

    harness
        .log()
        .info_ctx("summary", "error_recovery complete", |ctx| {
            ctx.push((
                "turn_count".into(),
                outcome.capture.turn_durations_ms.len().to_string(),
            ));
            ctx.push(("total_tokens".into(), outcome.total_tokens.to_string()));
        });
    write_timeline_artifact(&harness, test_name, &outcome.capture);
    write_jsonl_artifacts(&harness, test_name);
}
