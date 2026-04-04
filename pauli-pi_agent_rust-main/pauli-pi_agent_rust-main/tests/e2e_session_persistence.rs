//! E2E: session persistence lifecycle tests (bd-277x).
//!
//! These tests exercise the real `AgentSession` + `Session` persistence path
//! using deterministic in-process provider streams.

mod common;

use asupersync::runtime::RuntimeBuilder;
use async_trait::async_trait;
use clap::Parser;
use common::TestHarness;
#[cfg(unix)]
use common::tmux::{TmuxInstance, sh_escape};
use futures::Stream;
use pi::agent::{Agent, AgentConfig, AgentSession};
use pi::cli::Cli;
use pi::compaction::ResolvedCompactionSettings;
use pi::config::Config;
use pi::error::{Error, Result};
use pi::model::{
    AssistantMessage, ContentBlock, Message, StopReason, StreamEvent, TextContent, ToolCall, Usage,
    UserContent,
};
use pi::provider::{Context, Provider, StreamOptions};
#[cfg(unix)]
use pi::session::encode_cwd;
use pi::session::{Session, SessionEntry, SessionMessage, SessionStoreKind};
use pi::tools::ToolRegistry;
use serde_json::json;
use std::collections::{BTreeMap, HashSet};
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct PlannedStep {
    stop_reason: StopReason,
    content: Vec<ContentBlock>,
    min_context_messages: usize,
    total_tokens: u64,
}

#[derive(Debug)]
struct PlannedProvider {
    steps: Vec<PlannedStep>,
    call_count: AtomicUsize,
}

impl PlannedProvider {
    const fn new(steps: Vec<PlannedStep>) -> Self {
        Self {
            steps,
            call_count: AtomicUsize::new(0),
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
}

#[async_trait]
#[allow(clippy::unnecessary_literal_bound)]
impl Provider for PlannedProvider {
    fn name(&self) -> &str {
        "planned-provider"
    }

    fn api(&self) -> &str {
        "planned-api"
    }

    fn model_id(&self) -> &str {
        "planned-model"
    }

    async fn stream(
        &self,
        context: &Context<'_>,
        _options: &StreamOptions,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send>>> {
        let index = self.call_count.fetch_add(1, Ordering::SeqCst);
        let Some(step) = self.steps.get(index) else {
            return Err(Error::api("planned provider exhausted its scripted steps"));
        };
        if context.messages.len() < step.min_context_messages {
            return Err(Error::api(format!(
                "planned provider expected >= {} context messages, got {}",
                step.min_context_messages,
                context.messages.len()
            )));
        }

        let message =
            self.assistant_message(step.stop_reason, step.content.clone(), step.total_tokens);
        let partial = self.assistant_message(StopReason::Stop, Vec::new(), 0);
        Ok(Box::pin(futures::stream::iter(vec![
            Ok(StreamEvent::Start { partial }),
            Ok(StreamEvent::Done {
                reason: message.stop_reason,
                message,
            }),
        ])))
    }
}

fn run_async_test<F>(future: F)
where
    F: std::future::Future<Output = ()>,
{
    let runtime = RuntimeBuilder::current_thread()
        .build()
        .expect("runtime build");
    runtime.block_on(future);
}

fn text_step(text: &str, min_context_messages: usize, total_tokens: u64) -> PlannedStep {
    PlannedStep {
        stop_reason: StopReason::Stop,
        content: vec![ContentBlock::Text(TextContent::new(text))],
        min_context_messages,
        total_tokens,
    }
}

fn tool_step(tool_call: ToolCall, min_context_messages: usize, total_tokens: u64) -> PlannedStep {
    PlannedStep {
        stop_reason: StopReason::ToolUse,
        content: vec![ContentBlock::ToolCall(tool_call)],
        min_context_messages,
        total_tokens,
    }
}

const fn tool_names() -> [&'static str; 7] {
    ["read", "write", "edit", "bash", "grep", "find", "ls"]
}

const DEFAULT_CLI_TIMEOUT_SECS: u64 = 120;

#[derive(Debug)]
struct CliResult {
    exit_code: i32,
    stdout: String,
    stderr: String,
    duration: Duration,
}

fn cli_binary_path() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_pi"))
}

fn isolated_cli_env(harness: &TestHarness) -> BTreeMap<String, String> {
    let mut env = BTreeMap::new();
    let env_root = harness.temp_path("pi-env");
    let _ = std::fs::create_dir_all(&env_root);

    env.insert(
        "PI_CODING_AGENT_DIR".to_string(),
        env_root.join("agent").display().to_string(),
    );
    env.insert(
        "PI_CONFIG_PATH".to_string(),
        env_root.join("settings.json").display().to_string(),
    );
    env.insert(
        "PI_SESSIONS_DIR".to_string(),
        env_root.join("sessions").display().to_string(),
    );
    env.insert(
        "PI_PACKAGE_DIR".to_string(),
        env_root.join("packages").display().to_string(),
    );
    env.insert("PI_TEST_MODE".to_string(), "1".to_string());

    env
}

fn run_cli(
    harness: &TestHarness,
    env: &BTreeMap<String, String>,
    args: &[&str],
    stdin: Option<&[u8]>,
) -> CliResult {
    harness
        .log()
        .info("action", format!("Running CLI: {}", args.join(" ")));
    harness.log().info_ctx("action", "CLI env", |ctx| {
        for (key, value) in env {
            ctx.push((key.clone(), value.clone()));
        }
    });

    let mut command = Command::new(cli_binary_path());
    command.env_remove("ANTHROPIC_API_KEY");
    command.env_remove("OPENAI_API_KEY");
    command.env_remove("GEMINI_API_KEY");
    command.env_remove("GROQ_API_KEY");
    command.env_remove("KIMI_API_KEY");
    command.env_remove("AZURE_OPENAI_API_KEY");
    command.env_remove("PI_OPENROUTER_API_KEY");
    command.env_remove("PI_AWS_ACCESS_KEY_ID");
    command
        .args(args)
        .envs(env.clone())
        .current_dir(harness.temp_dir())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if stdin.is_some() {
        command.stdin(Stdio::piped());
    } else {
        command.stdin(Stdio::null());
    }

    let start = Instant::now();
    let mut child = command.spawn().expect("run pi");
    let mut child_stdout = child.stdout.take().expect("child stdout piped");
    let mut child_stderr = child.stderr.take().expect("child stderr piped");
    let stdout_handle = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = std::io::Read::read_to_end(&mut child_stdout, &mut buf);
        buf
    });
    let stderr_handle = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = std::io::Read::read_to_end(&mut child_stderr, &mut buf);
        buf
    });

    if let Some(input) = stdin {
        if let Some(mut child_stdin) = child.stdin.take() {
            child_stdin.write_all(input).expect("write stdin");
        }
    }

    let timeout = Duration::from_secs(DEFAULT_CLI_TIMEOUT_SECS);
    let mut timed_out = false;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {}
            Err(err) => panic!("try_wait failed: {err}"),
        }

        if start.elapsed() > timeout {
            timed_out = true;
            let _ = child.kill();
            break child.wait().expect("wait child after kill");
        }

        std::thread::sleep(Duration::from_millis(25));
    };

    let stdout_bytes = stdout_handle.join().unwrap_or_default();
    let stderr_bytes = stderr_handle.join().unwrap_or_default();
    let duration = start.elapsed();
    let stdout = String::from_utf8_lossy(&stdout_bytes).to_string();
    let mut stderr = String::from_utf8_lossy(&stderr_bytes).to_string();
    let exit_code = if timed_out {
        stderr = format!("ERROR: timed out after {timeout:?}\n{stderr}");
        -1
    } else {
        status.code().unwrap_or(-1)
    };

    harness.log().info_ctx("result", "CLI completed", |ctx| {
        ctx.push(("exit_code".into(), exit_code.to_string()));
        ctx.push(("duration_ms".into(), duration.as_millis().to_string()));
        ctx.push(("stdout_len".into(), stdout.len().to_string()));
        ctx.push(("stderr_len".into(), stderr.len().to_string()));
    });

    CliResult {
        exit_code,
        stdout,
        stderr,
        duration,
    }
}

fn assert_contains(harness: &TestHarness, haystack: &str, needle: &str) {
    harness.assert_log(format!("assert contains: {needle}").as_str());
    assert!(
        haystack.contains(needle),
        "expected output to contain '{needle}'"
    );
}

fn write_minimal_session(path: &Path, cwd: &Path, session_id: &str, message: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create session parent dir");
    }

    let header = json!({
        "type": "session",
        "version": 3,
        "id": session_id,
        "timestamp": "2026-02-06T00:00:00.000Z",
        "cwd": cwd.display().to_string(),
        "provider": "anthropic",
        "modelId": "claude-sonnet-4-5"
    });
    let user_entry = json!({
        "type": "message",
        "id": "entry-user-1",
        "timestamp": "2026-02-06T00:00:01.000Z",
        "message": {
            "role": "user",
            "content": message
        }
    });
    std::fs::write(path, format!("{header}\n{user_entry}\n")).expect("write minimal session");
}

fn make_agent_session(
    cwd: &Path,
    provider: Arc<dyn Provider>,
    session: Arc<asupersync::sync::Mutex<Session>>,
) -> AgentSession {
    let agent = Agent::new(
        provider,
        ToolRegistry::new(&tool_names(), cwd, None),
        AgentConfig {
            max_tool_iterations: 12,
            stream_options: StreamOptions {
                api_key: Some("test-key".to_string()),
                ..StreamOptions::default()
            },
            ..AgentConfig::default()
        },
    );
    AgentSession::new(agent, session, true, ResolvedCompactionSettings::default())
}

fn write_jsonl_artifacts(harness: &TestHarness, test_name: &str) {
    let log_path = harness.temp_path(format!("{test_name}.log.jsonl"));
    harness
        .write_jsonl_logs(&log_path)
        .expect("write jsonl logs");
    harness.record_artifact(format!("{test_name}.log.jsonl"), &log_path);

    let normalized_log_path = harness.temp_path(format!("{test_name}.log.normalized.jsonl"));
    harness
        .write_jsonl_logs_normalized(&normalized_log_path)
        .expect("write normalized jsonl logs");
    harness.record_artifact(
        format!("{test_name}.log.normalized.jsonl"),
        &normalized_log_path,
    );

    let artifacts_path = harness.temp_path(format!("{test_name}.artifacts.jsonl"));
    harness
        .write_artifact_index_jsonl(&artifacts_path)
        .expect("write artifact index");
    harness.record_artifact(format!("{test_name}.artifacts.jsonl"), &artifacts_path);

    let normalized_artifacts_path =
        harness.temp_path(format!("{test_name}.artifacts.normalized.jsonl"));
    harness
        .write_artifact_index_jsonl_normalized(&normalized_artifacts_path)
        .expect("write normalized artifact index");
    harness.record_artifact(
        format!("{test_name}.artifacts.normalized.jsonl"),
        &normalized_artifacts_path,
    );
}

async fn current_session_path(session: &Arc<asupersync::sync::Mutex<Session>>) -> PathBuf {
    let cx = asupersync::Cx::for_testing();
    let guard = session.lock(&cx).await.expect("lock session");
    guard.path.clone().expect("session path")
}

async fn current_messages(session: &Arc<asupersync::sync::Mutex<Session>>) -> Vec<Message> {
    let cx = asupersync::Cx::for_testing();
    let guard = session.lock(&cx).await.expect("lock session");
    guard.to_messages_for_current_path()
}

fn user_texts_in_order(messages: &[Message]) -> Vec<String> {
    messages
        .iter()
        .filter_map(|message| match message {
            Message::User(user) => match &user.content {
                UserContent::Text(text) => Some(text.clone()),
                UserContent::Blocks(_) => None,
            },
            _ => None,
        })
        .collect()
}

fn assert_no_duplicate_user_texts(user_texts: &[String], context: &str) {
    let unique: HashSet<&String> = user_texts.iter().collect();
    assert_eq!(
        unique.len(),
        user_texts.len(),
        "duplicate user text detected in {context}"
    );
}

#[test]
fn create_and_save() {
    let test_name = "e2e_session_create_and_save";
    let harness = TestHarness::new(test_name);
    run_async_test(async {
        let cwd = harness.temp_dir().to_path_buf();
        let session = Arc::new(asupersync::sync::Mutex::new(Session::create_with_dir(
            Some(cwd.clone()),
        )));
        let provider: Arc<dyn Provider> = Arc::new(PlannedProvider::new(vec![text_step(
            "created session",
            1,
            12,
        )]));
        let mut agent_session = make_agent_session(&cwd, provider, Arc::clone(&session));

        let response = agent_session
            .run_text("hello persistence".to_string(), |_| {})
            .await
            .expect("run first turn");
        assert_eq!(response.stop_reason, StopReason::Stop);

        agent_session
            .persist_session()
            .await
            .expect("persist session");
        let path = current_session_path(&session).await;
        harness.record_artifact("session.jsonl", &path);

        assert!(path.exists(), "session file should exist");
        let raw = std::fs::read_to_string(&path).expect("read session jsonl");
        let lines = raw
            .lines()
            .filter(|line| !line.trim().is_empty())
            .collect::<Vec<_>>();
        assert!(
            lines.len() >= 3,
            "expected header + user + assistant entries"
        );

        let header: serde_json::Value = serde_json::from_str(lines[0]).expect("parse header line");
        assert_eq!(
            header.get("type").and_then(serde_json::Value::as_str),
            Some("session")
        );
        assert!(
            lines.iter().any(|line| line.contains("\"role\":\"user\"")),
            "missing user message entry"
        );
        assert!(
            lines
                .iter()
                .any(|line| line.contains("\"role\":\"assistant\"")),
            "missing assistant message entry"
        );
    });
    write_jsonl_artifacts(&harness, test_name);
}

#[test]
fn reload_session() {
    let test_name = "e2e_session_reload_continue";
    let harness = TestHarness::new(test_name);
    run_async_test(async {
        let cwd = harness.temp_dir().to_path_buf();
        let session = Arc::new(asupersync::sync::Mutex::new(Session::create_with_dir(
            Some(cwd.clone()),
        )));
        let initial_provider: Arc<dyn Provider> = Arc::new(PlannedProvider::new(vec![text_step(
            "first response",
            1,
            10,
        )]));
        let mut first = make_agent_session(&cwd, initial_provider, Arc::clone(&session));
        first
            .run_text("first prompt".to_string(), |_| {})
            .await
            .expect("first run");
        first.persist_session().await.expect("first persist");

        let saved_path = current_session_path(&session).await;
        harness.record_artifact("initial-session.jsonl", &saved_path);
        let reopened = Session::open(saved_path.to_string_lossy().as_ref())
            .await
            .expect("reopen saved session");
        let reopened_handle = Arc::new(asupersync::sync::Mutex::new(reopened));

        let continue_provider: Arc<dyn Provider> = Arc::new(PlannedProvider::new(vec![text_step(
            "continued response",
            3,
            11,
        )]));
        let mut continued =
            make_agent_session(&cwd, continue_provider, Arc::clone(&reopened_handle));
        continued
            .run_text("second prompt".to_string(), |_| {})
            .await
            .expect("continued run");
        continued
            .persist_session()
            .await
            .expect("persist continued run");

        let messages = current_messages(&reopened_handle).await;
        let user_texts = messages
            .iter()
            .filter_map(|message| match message {
                Message::User(user) => match &user.content {
                    UserContent::Text(text) => Some(text.clone()),
                    UserContent::Blocks(_) => None,
                },
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            user_texts.len(),
            2,
            "expected two user prompts after reload"
        );
        assert!(user_texts.iter().any(|text| text == "first prompt"));
        assert!(user_texts.iter().any(|text| text == "second prompt"));
    });
    write_jsonl_artifacts(&harness, test_name);
}

#[test]
fn session_branching() {
    let test_name = "e2e_session_branching";
    let harness = TestHarness::new(test_name);
    run_async_test(async {
        let cwd = harness.temp_dir().to_path_buf();
        let session = Arc::new(asupersync::sync::Mutex::new(Session::create_with_dir(
            Some(cwd.clone()),
        )));
        let provider: Arc<dyn Provider> = Arc::new(PlannedProvider::new(vec![
            text_step("reply one", 1, 8),
            text_step("reply two", 3, 8),
            text_step("reply three", 5, 8),
        ]));
        let mut agent_session = make_agent_session(&cwd, provider, Arc::clone(&session));
        for prompt in ["turn one", "turn two", "turn three"] {
            agent_session
                .run_text(prompt.to_string(), |_| {})
                .await
                .expect("run turn");
        }

        let branched_from = {
            let cx = asupersync::Cx::for_testing();
            let mut guard = session.lock(&cx).await.expect("lock session");
            let user_ids = guard
                .entries
                .iter()
                .filter_map(|entry| match entry {
                    SessionEntry::Message(message_entry) => match &message_entry.message {
                        SessionMessage::User { .. } => message_entry.base.id.clone(),
                        _ => None,
                    },
                    _ => None,
                })
                .collect::<Vec<_>>();
            let target = user_ids
                .get(1)
                .cloned()
                .expect("second user message id for branch");
            assert!(guard.create_branch_from(&target), "create branch");
            guard.append_message(SessionMessage::User {
                content: UserContent::Text("branch turn".to_string()),
                timestamp: Some(0),
            });
            guard.save().await.expect("save branch");
            target
        };

        let path = current_session_path(&session).await;
        let reopened = Session::open(path.to_string_lossy().as_ref())
            .await
            .expect("reopen branched session");
        let summary = reopened.branch_summary();
        assert!(summary.branch_point_count >= 1);
        assert!(
            summary.branch_points.contains(&branched_from),
            "expected branch point at second user message"
        );
    });
    write_jsonl_artifacts(&harness, test_name);
}

#[test]
fn session_metadata() {
    let test_name = "e2e_session_metadata";
    let harness = TestHarness::new(test_name);
    run_async_test(async {
        let cwd = harness.temp_dir().to_path_buf();
        let mut session = Session::create_with_dir(Some(cwd));
        session.append_message(SessionMessage::User {
            content: UserContent::Text("metadata baseline".to_string()),
            timestamp: Some(0),
        });
        session.append_model_change("anthropic".to_string(), "claude-sonnet-4-5".to_string());
        session.append_thinking_level_change("high".to_string());
        session.set_model_header(
            Some("anthropic".to_string()),
            Some("claude-sonnet-4-5".to_string()),
            Some("high".to_string()),
        );
        session.save().await.expect("save metadata session");

        let path = session.path.clone().expect("metadata session path");
        harness.record_artifact("metadata-session.jsonl", &path);
        let raw = std::fs::read_to_string(&path).expect("read metadata session");
        assert!(raw.contains("\"type\":\"model_change\""));
        assert!(raw.contains("\"type\":\"thinking_level_change\""));

        let reopened = Session::open(path.to_string_lossy().as_ref())
            .await
            .expect("reopen metadata session");
        assert_eq!(reopened.header.provider.as_deref(), Some("anthropic"));
        assert_eq!(
            reopened.header.model_id.as_deref(),
            Some("claude-sonnet-4-5")
        );
        assert_eq!(reopened.header.thinking_level.as_deref(), Some("high"));
        assert!(
            reopened
                .entries
                .iter()
                .any(|entry| matches!(entry, SessionEntry::ModelChange(_)))
        );
        assert!(
            reopened
                .entries
                .iter()
                .any(|entry| matches!(entry, SessionEntry::ThinkingLevelChange(_)))
        );
    });
    write_jsonl_artifacts(&harness, test_name);
}

#[test]
fn multi_turn_persistence() {
    let test_name = "e2e_session_multi_turn_persistence";
    let harness = TestHarness::new(test_name);
    run_async_test(async {
        let cwd = harness.temp_dir().to_path_buf();
        let fixture = harness.create_file("fixtures/persist.txt", "persisted-value\n");
        let session = Arc::new(asupersync::sync::Mutex::new(Session::create_with_dir(
            Some(cwd.clone()),
        )));

        let steps = vec![
            text_step("turn one response", 1, 9),
            tool_step(
                ToolCall {
                    id: "read-1".to_string(),
                    name: "read".to_string(),
                    arguments: json!({ "path": fixture.display().to_string() }),
                    thought_signature: None,
                },
                3,
                18,
            ),
            text_step("tool turn completed", 5, 10),
            text_step("turn three response", 7, 11),
        ];
        let provider: Arc<dyn Provider> = Arc::new(PlannedProvider::new(steps));
        let mut agent_session = make_agent_session(&cwd, provider, Arc::clone(&session));

        agent_session
            .run_text("turn one".to_string(), |_| {})
            .await
            .expect("run turn one");
        agent_session
            .run_text("turn two with tool".to_string(), |_| {})
            .await
            .expect("run turn two");
        agent_session
            .run_text("turn three".to_string(), |_| {})
            .await
            .expect("run turn three");
        agent_session
            .persist_session()
            .await
            .expect("persist multi-turn session");

        let path = current_session_path(&session).await;
        harness.record_artifact("multi-turn-session.jsonl", &path);
        let reopened = Session::open(path.to_string_lossy().as_ref())
            .await
            .expect("reopen multi-turn session");

        let (mut user_count, mut assistant_count, mut tool_result_count) = (0usize, 0usize, 0usize);
        for entry in &reopened.entries {
            if let SessionEntry::Message(message_entry) = entry {
                match &message_entry.message {
                    SessionMessage::User { .. } => user_count += 1,
                    SessionMessage::Assistant { .. } => assistant_count += 1,
                    SessionMessage::ToolResult { .. } => tool_result_count += 1,
                    _ => {}
                }
            }
        }

        assert_eq!(user_count, 3, "expected three persisted user turns");
        assert!(
            assistant_count >= 4,
            "expected assistant tool-use + completion turns to persist"
        );
        assert!(
            tool_result_count >= 1,
            "expected persisted tool result entries"
        );
    });
    write_jsonl_artifacts(&harness, test_name);
}

#[test]
fn jsonl_fault_injection_flush_windows_preserve_integrity() {
    let test_name = "e2e_jsonl_fault_injection_flush_windows";
    let harness = TestHarness::new(test_name);
    harness.section("jsonl_fault_injection");

    run_async_test(async {
        let cwd = harness.temp_dir().to_path_buf();
        let mut session = Session::create_with_dir_and_store(Some(cwd), SessionStoreKind::Jsonl);
        session.append_message(SessionMessage::User {
            content: UserContent::Text("jsonl-base".to_string()),
            timestamp: Some(0),
        });
        session.save().await.expect("save baseline jsonl session");
        let stable_path = session.path.clone().expect("jsonl session path");
        harness.record_artifact("jsonl-fault-initial-session", &stable_path);

        // Pre-flush crash window: pending mutation should not corrupt persisted state.
        session.append_message(SessionMessage::User {
            content: UserContent::Text("jsonl-preflush-pending".to_string()),
            timestamp: Some(0),
        });
        drop(session);

        let reopened_pre = Session::open(stable_path.to_string_lossy().as_ref())
            .await
            .expect("reopen after pre-flush crash simulation");
        let pre_texts = user_texts_in_order(&reopened_pre.to_messages_for_current_path());
        assert_eq!(pre_texts, vec!["jsonl-base".to_string()]);
        assert_no_duplicate_user_texts(&pre_texts, "jsonl pre-flush window");

        // Mid-flush crash window: force a flush error by pointing path at a directory.
        let mut mid = reopened_pre;
        mid.append_message(SessionMessage::User {
            content: UserContent::Text("jsonl-midflush-pending".to_string()),
            timestamp: Some(0),
        });
        let fault_path = harness.create_dir("jsonl-midflush-fault-path");
        mid.path = Some(fault_path.clone());
        let flush_err = mid.save().await.expect_err("mid-flush save should fail");
        harness
            .log()
            .info_ctx("fault", "jsonl mid-flush failure", |ctx| {
                ctx.push(("fault_path".into(), fault_path.display().to_string()));
                ctx.push(("error".into(), flush_err.to_string()));
            });

        // Simulate process crash/restart after failed flush.
        drop(mid);
        let reopened_mid = Session::open(stable_path.to_string_lossy().as_ref())
            .await
            .expect("reopen after mid-flush crash simulation");
        let mid_texts = user_texts_in_order(&reopened_mid.to_messages_for_current_path());
        assert_eq!(mid_texts, vec!["jsonl-base".to_string()]);
        assert_no_duplicate_user_texts(&mid_texts, "jsonl mid-flush window");

        // Post-flush crash window: persisted mutation survives exactly once.
        let mut post = reopened_mid;
        post.append_message(SessionMessage::User {
            content: UserContent::Text("jsonl-postflush-persisted".to_string()),
            timestamp: Some(0),
        });
        post.save().await.expect("post-flush save should succeed");
        drop(post);

        let reopened_post = Session::open(stable_path.to_string_lossy().as_ref())
            .await
            .expect("reopen after post-flush crash simulation");
        let post_texts = user_texts_in_order(&reopened_post.to_messages_for_current_path());
        assert_eq!(
            post_texts,
            vec![
                "jsonl-base".to_string(),
                "jsonl-postflush-persisted".to_string()
            ],
            "jsonl post-crash ordering mismatch"
        );
        assert_no_duplicate_user_texts(&post_texts, "jsonl post-flush window");

        let summary_path = harness.temp_path("jsonl-fault-window-summary.json");
        std::fs::write(
            &summary_path,
            serde_json::to_string_pretty(&json!({
                "scenario": "jsonl_fault_windows",
                "windows": {
                    "pre_flush": pre_texts,
                    "mid_flush": mid_texts,
                    "post_flush": post_texts
                }
            }))
            .expect("serialize jsonl fault summary"),
        )
        .expect("write jsonl fault summary");
        harness.record_artifact("jsonl-fault-window-summary.json", &summary_path);
    });

    write_jsonl_artifacts(&harness, test_name);
}

#[cfg(feature = "sqlite-sessions")]
#[test]
fn sqlite_fault_injection_flush_windows_preserve_integrity() {
    let test_name = "e2e_sqlite_fault_injection_flush_windows";
    let harness = TestHarness::new(test_name);
    harness.section("sqlite_fault_injection");

    run_async_test(async {
        let cwd = harness.temp_dir().to_path_buf();
        let mut session = Session::create_with_dir_and_store(Some(cwd), SessionStoreKind::Sqlite);
        session.append_message(SessionMessage::User {
            content: UserContent::Text("sqlite-base".to_string()),
            timestamp: Some(0),
        });
        session.save().await.expect("save baseline sqlite session");
        let stable_path = session.path.clone().expect("sqlite session path");
        harness.record_artifact("sqlite-fault-initial-session", &stable_path);

        // Pre-flush crash window.
        session.append_message(SessionMessage::User {
            content: UserContent::Text("sqlite-preflush-pending".to_string()),
            timestamp: Some(0),
        });
        drop(session);

        let reopened_pre = Session::open(stable_path.to_string_lossy().as_ref())
            .await
            .expect("reopen sqlite after pre-flush crash simulation");
        let pre_texts = user_texts_in_order(&reopened_pre.to_messages_for_current_path());
        assert_eq!(pre_texts, vec!["sqlite-base".to_string()]);
        assert_no_duplicate_user_texts(&pre_texts, "sqlite pre-flush window");

        // Mid-flush crash window.
        let mut mid = reopened_pre;
        mid.append_message(SessionMessage::User {
            content: UserContent::Text("sqlite-midflush-pending".to_string()),
            timestamp: Some(0),
        });
        let fault_path = harness.create_dir("sqlite-midflush-fault-path");
        mid.path = Some(fault_path.clone());
        let flush_err = mid
            .save()
            .await
            .expect_err("sqlite mid-flush save should fail");
        harness
            .log()
            .info_ctx("fault", "sqlite mid-flush failure", |ctx| {
                ctx.push(("fault_path".into(), fault_path.display().to_string()));
                ctx.push(("error".into(), flush_err.to_string()));
            });

        drop(mid);
        let reopened_mid = Session::open(stable_path.to_string_lossy().as_ref())
            .await
            .expect("reopen sqlite after mid-flush crash simulation");
        let mid_texts = user_texts_in_order(&reopened_mid.to_messages_for_current_path());
        assert_eq!(mid_texts, vec!["sqlite-base".to_string()]);
        assert_no_duplicate_user_texts(&mid_texts, "sqlite mid-flush window");

        // Post-flush crash window.
        let mut post = reopened_mid;
        post.append_message(SessionMessage::User {
            content: UserContent::Text("sqlite-postflush-persisted".to_string()),
            timestamp: Some(0),
        });
        post.save()
            .await
            .expect("sqlite post-flush save should succeed");
        drop(post);

        let reopened_post = Session::open(stable_path.to_string_lossy().as_ref())
            .await
            .expect("reopen sqlite after post-flush crash simulation");
        let post_texts = user_texts_in_order(&reopened_post.to_messages_for_current_path());
        assert_eq!(
            post_texts,
            vec![
                "sqlite-base".to_string(),
                "sqlite-postflush-persisted".to_string()
            ],
            "sqlite post-crash ordering mismatch"
        );
        assert_no_duplicate_user_texts(&post_texts, "sqlite post-flush window");

        let summary_path = harness.temp_path("sqlite-fault-window-summary.json");
        std::fs::write(
            &summary_path,
            serde_json::to_string_pretty(&json!({
                "scenario": "sqlite_fault_windows",
                "windows": {
                    "pre_flush": pre_texts,
                    "mid_flush": mid_texts,
                    "post_flush": post_texts
                }
            }))
            .expect("serialize sqlite fault summary"),
        )
        .expect("write sqlite fault summary");
        harness.record_artifact("sqlite-fault-window-summary.json", &summary_path);
    });

    write_jsonl_artifacts(&harness, test_name);
}

#[test]
fn cli_export_html_integrity_from_session_file() {
    let test_name = "e2e_cli_export_html_integrity_from_session_file";
    let harness = TestHarness::new(test_name);

    let session_id = "session-export-123";
    let session_message = "Export integrity message";
    let session_path = harness.temp_path("fixtures/export-session.jsonl");
    write_minimal_session(
        &session_path,
        harness.temp_dir(),
        session_id,
        session_message,
    );
    harness.record_artifact("export-session.jsonl", &session_path);

    let export_path = harness.temp_path("export/session.html");
    let session_arg = session_path.to_string_lossy().to_string();
    let export_arg = export_path.to_string_lossy().to_string();

    let env = isolated_cli_env(&harness);
    let result = run_cli(
        &harness,
        &env,
        &["--export", &session_arg, &export_arg],
        None,
    );
    assert_eq!(
        result.exit_code, 0,
        "export command failed\nstderr:\n{}\nstdout:\n{}",
        result.stderr, result.stdout
    );
    assert_contains(&harness, &result.stdout, "Exported to:");
    assert!(export_path.exists(), "expected export file at {export_arg}");
    harness.record_artifact("export.html", &export_path);

    let html = std::fs::read_to_string(&export_path).expect("read export html");
    assert_contains(&harness, &html, session_id);
    assert_contains(&harness, &html, session_message);

    let header_line = std::fs::read_to_string(&session_path)
        .expect("read session file")
        .lines()
        .next()
        .expect("session header line")
        .to_string();
    let header: serde_json::Value = serde_json::from_str(&header_line).expect("parse header");
    harness
        .log()
        .info_ctx("verify", "export session metadata", |ctx| {
            ctx.push(("session_path".into(), session_path.display().to_string()));
            ctx.push((
                "session_size".into(),
                std::fs::metadata(&session_path)
                    .expect("session metadata")
                    .len()
                    .to_string(),
            ));
            ctx.push(("export_path".into(), export_path.display().to_string()));
            ctx.push((
                "export_size".into(),
                std::fs::metadata(&export_path)
                    .expect("export metadata")
                    .len()
                    .to_string(),
            ));
            ctx.push((
                "session_id".into(),
                header["id"].as_str().unwrap_or_default().to_string(),
            ));
            ctx.push((
                "provider".into(),
                header["provider"].as_str().unwrap_or_default().to_string(),
            ));
            ctx.push((
                "model_id".into(),
                header["modelId"].as_str().unwrap_or_default().to_string(),
            ));
            ctx.push((
                "duration_ms".into(),
                result.duration.as_millis().to_string(),
            ));
        });

    write_jsonl_artifacts(&harness, test_name);
}

#[test]
fn session_dir_override_and_env_sessions_path() {
    let test_name = "e2e_session_dir_override_and_env_sessions_path";
    let harness = TestHarness::new(test_name);

    let env_sessions = harness.temp_path("env-sessions");
    std::fs::create_dir_all(&env_sessions).expect("create env sessions dir");
    let mut env = isolated_cli_env(&harness);
    env.insert(
        "PI_SESSIONS_DIR".to_string(),
        env_sessions.display().to_string(),
    );

    let config_result = run_cli(&harness, &env, &["config"], None);
    assert_eq!(
        config_result.exit_code, 0,
        "config command failed\nstderr:\n{}\nstdout:\n{}",
        config_result.stderr, config_result.stdout
    );
    assert_contains(
        &harness,
        &config_result.stdout,
        env_sessions.display().to_string().as_str(),
    );

    run_async_test(async {
        let cli_sessions = harness.temp_path("cli-session-dir");
        let cli_session_arg = cli_sessions.to_string_lossy().to_string();
        let cli = Cli::parse_from(["pi", "--session-dir", cli_session_arg.as_str()]);
        let mut session = Session::new(&cli, &Config::default())
            .await
            .expect("session new with --session-dir");
        session.append_message(SessionMessage::User {
            content: UserContent::Text("session-dir override message".to_string()),
            timestamp: Some(0),
        });
        session
            .save()
            .await
            .expect("save session with --session-dir");

        let path = session.path.clone().expect("session path");
        assert!(
            path.starts_with(&cli_sessions),
            "expected session path under --session-dir. path={}, root={}",
            path.display(),
            cli_sessions.display()
        );
        harness.record_artifact("session-dir-override.jsonl", &path);
        harness
            .log()
            .info_ctx("verify", "session-dir override metadata", |ctx| {
                ctx.push(("session_path".into(), path.display().to_string()));
                ctx.push((
                    "session_size".into(),
                    std::fs::metadata(&path)
                        .expect("session metadata")
                        .len()
                        .to_string(),
                ));
            });
    });

    write_jsonl_artifacts(&harness, test_name);
}

#[test]
fn explicit_session_flag_loads_requested_session() {
    let test_name = "e2e_explicit_session_flag_loads_requested_session";
    let harness = TestHarness::new(test_name);

    run_async_test(async {
        let expected_id = "session-explicit-999";
        let expected_message = "explicit session payload";
        let session_path = harness.temp_path("explicit/session.jsonl");
        write_minimal_session(
            &session_path,
            harness.temp_dir(),
            expected_id,
            expected_message,
        );
        harness.record_artifact("explicit-session.jsonl", &session_path);

        let session_arg = session_path.to_string_lossy().to_string();
        let cli = Cli::parse_from(["pi", "--session", session_arg.as_str()]);
        let loaded = Session::new(&cli, &Config::default())
            .await
            .expect("load explicit session");

        assert_eq!(loaded.header.id, expected_id);
        let contains_user_message = loaded
            .to_messages_for_current_path()
            .iter()
            .any(|message| matches!(
                message,
                Message::User(user) if matches!(&user.content, UserContent::Text(text) if text == expected_message)
            ));
        assert!(
            contains_user_message,
            "explicitly loaded session missing expected user message"
        );
    });

    write_jsonl_artifacts(&harness, test_name);
}

#[cfg(unix)]
#[test]
#[allow(clippy::too_many_lines)]
fn cli_continue_tmux_loads_existing_session() {
    let test_name = "e2e_cli_continue_tmux_loads_existing_session";
    let harness = TestHarness::new(test_name);

    if !TmuxInstance::tmux_available() {
        harness.log().warn("tmux", "Skipping: tmux not available");
        return;
    }

    let mut env = isolated_cli_env(&harness);
    env.insert("VCR_MODE".to_string(), "playback".to_string());
    let cassette_dir = harness.temp_path("vcr-cassettes");
    std::fs::create_dir_all(&cassette_dir).expect("create cassette dir");
    env.insert(
        "VCR_CASSETTE_DIR".to_string(),
        cassette_dir.display().to_string(),
    );
    env.insert(
        "PI_VCR_TEST_NAME".to_string(),
        "e2e_continue_session".to_string(),
    );
    env.insert("ANTHROPIC_API_KEY".to_string(), "test-vcr-key".to_string());

    let cassette_path = cassette_dir.join("e2e_continue_session.json");
    std::fs::write(
        &cassette_path,
        serde_json::to_string_pretty(&json!({
            "version": "1.0",
            "test_name": "e2e_continue_session",
            "recorded_at": "2026-02-06T00:00:00.000Z",
            "interactions": []
        }))
        .expect("serialize cassette"),
    )
    .expect("write cassette");
    harness.record_artifact("continue-cassette.json", &cassette_path);

    let sessions_dir = PathBuf::from(env.get("PI_SESSIONS_DIR").expect("PI_SESSIONS_DIR"));
    let project_sessions = sessions_dir.join(encode_cwd(harness.temp_dir()));
    std::fs::create_dir_all(&project_sessions).expect("create project sessions dir");
    let session_file = project_sessions.join("2026-02-06T00-00-00.000Z_continue.jsonl");
    let session_id = "continue-session-123";
    let session_message = "Continue session baseline";
    write_minimal_session(
        &session_file,
        harness.temp_dir(),
        session_id,
        session_message,
    );
    harness.record_artifact("continue-source-session.jsonl", &session_file);

    let tmux = TmuxInstance::new(&harness);
    let script_path = harness.temp_path("continue-session.sh");
    let mut script = String::new();
    script.push_str("#!/usr/bin/env sh\nset -eu\n");
    for (key, value) in &env {
        script.push_str("export ");
        script.push_str(key);
        script.push('=');
        script.push_str(&sh_escape(value));
        script.push('\n');
    }
    script.push_str("exec ");
    script.push_str(&sh_escape(cli_binary_path().to_string_lossy().as_ref()));
    for arg in [
        "-c",
        "--provider",
        "anthropic",
        "--model",
        "claude-sonnet-4-5",
        "--api-key",
        "test-vcr-key",
        "--no-tools",
        "--no-skills",
        "--no-prompt-templates",
        "--no-extensions",
        "--no-themes",
        "--thinking",
        "off",
    ] {
        script.push(' ');
        script.push_str(&sh_escape(arg));
    }
    script.push('\n');

    std::fs::write(&script_path, script).expect("write continue script");
    let mut perms = std::fs::metadata(&script_path)
        .expect("script metadata")
        .permissions();
    #[allow(clippy::cast_possible_truncation)]
    {
        use std::os::unix::fs::PermissionsExt;
        perms.set_mode(0o755);
    }
    std::fs::set_permissions(&script_path, perms).expect("chmod script");
    harness.record_artifact("continue-script.sh", &script_path);

    tmux.start_session(harness.temp_dir(), &script_path);
    let pane = tmux.wait_for_pane_contains_any(
        &["Continuing session", "Welcome to Pi!", "Pi ("],
        Duration::from_secs(20),
    );
    assert!(
        pane.contains("Continuing session")
            || pane.contains("Welcome to Pi!")
            || pane.contains("Pi ("),
        "expected continue startup text, got:\n{pane}"
    );

    tmux.send_literal("/exit");
    tmux.send_key("Enter");
    let start = Instant::now();
    while tmux.session_exists() {
        if start.elapsed() > Duration::from_secs(10) {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    if tmux.session_exists() {
        tmux.try_send_key("C-d");
    }

    let final_pane = if tmux.session_exists() {
        tmux.capture_pane()
    } else {
        pane
    };
    let pane_path = harness.temp_path("continue-pane.txt");
    std::fs::write(&pane_path, &final_pane).expect("write pane artifact");
    harness.record_artifact("continue-pane.txt", &pane_path);

    assert!(
        session_file.exists(),
        "continued session file should still exist"
    );
    let persisted = std::fs::read_to_string(&session_file).expect("read continued session");
    assert_contains(&harness, &persisted, session_id);
    assert_contains(&harness, &persisted, session_message);

    harness
        .log()
        .info_ctx("verify", "continue lifecycle metadata", |ctx| {
            ctx.push(("session_path".into(), session_file.display().to_string()));
            ctx.push((
                "session_size".into(),
                std::fs::metadata(&session_file)
                    .expect("session metadata")
                    .len()
                    .to_string(),
            ));
            ctx.push((
                "vcr_mode".into(),
                env.get("VCR_MODE").cloned().unwrap_or_default(),
            ));
            ctx.push((
                "cassette_name".into(),
                env.get("PI_VCR_TEST_NAME").cloned().unwrap_or_default(),
            ));
            ctx.push(("cassette_path".into(), cassette_path.display().to_string()));
        });

    write_jsonl_artifacts(&harness, test_name);
}
