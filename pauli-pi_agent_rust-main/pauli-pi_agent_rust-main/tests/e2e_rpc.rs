#![allow(clippy::similar_names)]
#![allow(clippy::too_many_lines)]

//! E2E RPC protocol tests — comprehensive command coverage.
//!
//! These tests drive the RPC server in-process via channels, exercising the full
//! JSON-line protocol for commands that are not yet covered by `rpc_mode.rs` or
//! `rpc_protocol.rs`.

mod common;

use common::TestHarness;
use pi::agent::{Agent, AgentConfig, AgentSession};
use pi::auth::AuthStorage;
use pi::config::Config;
use pi::extensions::{ExtensionManager, ExtensionRegion, ExtensionUiRequest};
use pi::http::client::Client;
use pi::model::{AssistantMessage, ContentBlock, StopReason, TextContent, Usage, UserContent};
use pi::models::ModelEntry;
use pi::provider::{InputType, Model, ModelCost, Provider};
use pi::providers::openai::OpenAIProvider;
use pi::resources::ResourceLoader;
use pi::rpc::{RpcOptions, RpcScopedModel, run};
use pi::session::{Session, SessionMessage};
use pi::tools::ToolRegistry;
use pi::vcr::{VcrMode, VcrRecorder};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, TryRecvError};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn cassette_root() -> PathBuf {
    std::env::var("VCR_CASSETTE_DIR").map_or_else(
        |_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/vcr"),
        PathBuf::from,
    )
}

fn test_model(id: &str, reasoning: bool) -> Model {
    Model {
        id: id.to_string(),
        name: id.to_string(),
        api: "anthropic".to_string(),
        provider: "anthropic".to_string(),
        base_url: "https://api.anthropic.com".to_string(),
        reasoning,
        input: vec![InputType::Text],
        cost: ModelCost {
            input: 3.0,
            output: 15.0,
            cache_read: 0.3,
            cache_write: 3.75,
        },
        context_window: 200_000,
        max_tokens: 8192,
        headers: HashMap::new(),
    }
}

fn test_entry(id: &str, reasoning: bool) -> ModelEntry {
    ModelEntry {
        model: test_model(id, reasoning),
        api_key: None,
        headers: HashMap::new(),
        auth_header: false,
        compat: None,
        oauth_config: None,
    }
}

fn build_agent_session(session: Session, cassette_dir: &Path) -> AgentSession {
    let model = "gpt-4o-mini".to_string();
    let recorder = VcrRecorder::new_with("e2e_rpc_noop", VcrMode::Playback, cassette_dir);
    let client = Client::new().with_vcr(recorder);
    let provider: Arc<dyn Provider> = Arc::new(OpenAIProvider::new(model).with_client(client));
    let tools = ToolRegistry::new(&[], &std::env::current_dir().unwrap(), None);
    let config = AgentConfig::default();
    let agent = Agent::new(provider, tools, config);
    let session = Arc::new(asupersync::sync::Mutex::new(session));
    AgentSession::new(
        agent,
        session,
        false,
        pi::compaction::ResolvedCompactionSettings::default(),
    )
}

fn build_options(
    handle: &asupersync::runtime::RuntimeHandle,
    auth_path: PathBuf,
    available_models: Vec<ModelEntry>,
    scoped_models: Vec<RpcScopedModel>,
) -> RpcOptions {
    let auth = AuthStorage::load(auth_path).expect("load auth storage");
    RpcOptions {
        config: Config::default(),
        resources: ResourceLoader::empty(false),
        available_models,
        scoped_models,
        auth,
        runtime_handle: handle.clone(),
    }
}

async fn recv_line(rx: &Arc<Mutex<Receiver<String>>>, label: &str) -> Result<String, String> {
    let start = Instant::now();
    loop {
        let recv_result = {
            let rx = rx.lock().expect("lock rpc output receiver");
            rx.try_recv()
        };

        match recv_result {
            Ok(line) => return Ok(line),
            Err(TryRecvError::Disconnected) => {
                return Err(format!("{label}: output channel disconnected"));
            }
            Err(TryRecvError::Empty) => {}
        }

        if start.elapsed() > Duration::from_secs(10) {
            return Err(format!("{label}: timed out waiting for output"));
        }

        asupersync::time::sleep(asupersync::time::wall_now(), Duration::from_millis(5)).await;
    }
}

fn parse_response(line: &str) -> Value {
    serde_json::from_str(line.trim()).expect("parse JSON response")
}

async fn recv_response(out_rx: &Arc<Mutex<Receiver<String>>>, label: &str) -> Value {
    let start = Instant::now();

    loop {
        let line = recv_line(out_rx, label)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let value = parse_response(&line);

        match value.get("type").and_then(Value::as_str) {
            Some("response") => return value,
            Some("agent_end") => {
                let has_error = value
                    .get("error")
                    .is_some_and(|error| !error.is_null() && error != "");
                assert!(
                    !has_error,
                    "{label}: unexpected agent_end error while waiting for response: {value}"
                );
            }
            _ => {}
        }

        assert!(
            start.elapsed() <= Duration::from_secs(10),
            "{label}: timed out waiting for RPC response"
        );
    }
}

/// Send a command and get the response.
async fn send_recv(
    in_tx: &asupersync::channel::mpsc::Sender<String>,
    out_rx: &Arc<Mutex<Receiver<String>>>,
    cmd: &str,
    label: &str,
) -> Value {
    let cx = asupersync::Cx::for_testing();
    in_tx
        .send(&cx, cmd.to_string())
        .await
        .unwrap_or_else(|_| panic!("send {label}"));
    recv_response(out_rx, label).await
}

/// Assert that a response indicates success with the expected command.
fn assert_ok(resp: &Value, command: &str) {
    assert_eq!(resp["type"], "response", "response type for {command}");
    assert_eq!(resp["command"], command);
    assert_eq!(resp["success"], true, "success for {command}: {resp}");
}

/// Assert that a response indicates an error with the expected command.
fn assert_err(resp: &Value, command: &str) {
    assert_eq!(resp["type"], "response", "response type for {command}");
    assert_eq!(resp["command"], command);
    assert_eq!(
        resp["success"], false,
        "expected error for {command}: {resp}"
    );
}

// ---------------------------------------------------------------------------
// Tests: Configuration commands
// ---------------------------------------------------------------------------

#[test]
fn rpc_set_steering_mode_valid() {
    let harness = TestHarness::new("rpc_set_steering_mode_valid");
    let cassette_dir = cassette_root();
    let runtime = asupersync::runtime::RuntimeBuilder::current_thread()
        .build()
        .expect("build test runtime");
    let handle = runtime.handle();

    runtime.block_on(async move {
        let agent_session = build_agent_session(Session::in_memory(), &cassette_dir);
        let options = build_options(&handle, harness.temp_path("auth.json"), vec![], vec![]);
        let (in_tx, in_rx) = asupersync::channel::mpsc::channel::<String>(16);
        let (out_tx, out_rx) = std::sync::mpsc::channel::<String>();
        let out_rx = Arc::new(Mutex::new(out_rx));

        let server = handle.spawn(async move { run(agent_session, options, in_rx, out_tx).await });

        // Set to "all"
        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"1","type":"set_steering_mode","mode":"all"}"#,
            "set_steering_mode(all)",
        )
        .await;
        assert_ok(&resp, "set_steering_mode");
        assert_eq!(resp["id"], "1");

        // Set to "one-at-a-time"
        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"2","type":"set_steering_mode","mode":"one-at-a-time"}"#,
            "set_steering_mode(one-at-a-time)",
        )
        .await;
        assert_ok(&resp, "set_steering_mode");

        drop(in_tx);
        let result = server.await;
        assert!(result.is_ok(), "rpc server error: {result:?}");
    });
}

#[test]
fn rpc_set_steering_mode_invalid() {
    let harness = TestHarness::new("rpc_set_steering_mode_invalid");
    let cassette_dir = cassette_root();
    let runtime = asupersync::runtime::RuntimeBuilder::current_thread()
        .build()
        .expect("build test runtime");
    let handle = runtime.handle();

    runtime.block_on(async move {
        let agent_session = build_agent_session(Session::in_memory(), &cassette_dir);
        let options = build_options(&handle, harness.temp_path("auth.json"), vec![], vec![]);
        let (in_tx, in_rx) = asupersync::channel::mpsc::channel::<String>(16);
        let (out_tx, out_rx) = std::sync::mpsc::channel::<String>();
        let out_rx = Arc::new(Mutex::new(out_rx));

        let server = handle.spawn(async move { run(agent_session, options, in_rx, out_tx).await });

        // Missing mode
        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"1","type":"set_steering_mode"}"#,
            "set_steering_mode(missing)",
        )
        .await;
        assert_err(&resp, "set_steering_mode");
        assert_eq!(resp["error"], "Missing mode");

        // Invalid mode
        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"2","type":"set_steering_mode","mode":"bogus"}"#,
            "set_steering_mode(bogus)",
        )
        .await;
        assert_err(&resp, "set_steering_mode");
        assert_eq!(resp["error"], "Invalid steering mode");

        drop(in_tx);
        let result = server.await;
        assert!(result.is_ok(), "rpc server error: {result:?}");
    });
}

#[test]
fn rpc_set_follow_up_mode_valid_and_invalid() {
    let harness = TestHarness::new("rpc_set_follow_up_mode_valid_and_invalid");
    let cassette_dir = cassette_root();
    let runtime = asupersync::runtime::RuntimeBuilder::current_thread()
        .build()
        .expect("build test runtime");
    let handle = runtime.handle();

    runtime.block_on(async move {
        let agent_session = build_agent_session(Session::in_memory(), &cassette_dir);
        let options = build_options(&handle, harness.temp_path("auth.json"), vec![], vec![]);
        let (in_tx, in_rx) = asupersync::channel::mpsc::channel::<String>(16);
        let (out_tx, out_rx) = std::sync::mpsc::channel::<String>();
        let out_rx = Arc::new(Mutex::new(out_rx));

        let server = handle.spawn(async move { run(agent_session, options, in_rx, out_tx).await });

        // Valid
        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"1","type":"set_follow_up_mode","mode":"all"}"#,
            "set_follow_up_mode(all)",
        )
        .await;
        assert_ok(&resp, "set_follow_up_mode");

        // Missing mode
        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"2","type":"set_follow_up_mode"}"#,
            "set_follow_up_mode(missing)",
        )
        .await;
        assert_err(&resp, "set_follow_up_mode");
        assert_eq!(resp["error"], "Missing mode");

        // Invalid mode
        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"3","type":"set_follow_up_mode","mode":"nope"}"#,
            "set_follow_up_mode(nope)",
        )
        .await;
        assert_err(&resp, "set_follow_up_mode");
        assert_eq!(resp["error"], "Invalid follow-up mode");

        drop(in_tx);
        let result = server.await;
        assert!(result.is_ok(), "rpc server error: {result:?}");
    });
}

#[test]
fn rpc_set_auto_compaction_and_retry() {
    let harness = TestHarness::new("rpc_set_auto_compaction_and_retry");
    let cassette_dir = cassette_root();
    let runtime = asupersync::runtime::RuntimeBuilder::current_thread()
        .build()
        .expect("build test runtime");
    let handle = runtime.handle();

    runtime.block_on(async move {
        let agent_session = build_agent_session(Session::in_memory(), &cassette_dir);
        let options = build_options(&handle, harness.temp_path("auth.json"), vec![], vec![]);
        let (in_tx, in_rx) = asupersync::channel::mpsc::channel::<String>(16);
        let (out_tx, out_rx) = std::sync::mpsc::channel::<String>();
        let out_rx = Arc::new(Mutex::new(out_rx));

        let server = handle.spawn(async move { run(agent_session, options, in_rx, out_tx).await });

        // set_auto_compaction true
        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"1","type":"set_auto_compaction","enabled":true}"#,
            "set_auto_compaction(true)",
        )
        .await;
        assert_ok(&resp, "set_auto_compaction");

        // set_auto_compaction missing enabled
        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"2","type":"set_auto_compaction"}"#,
            "set_auto_compaction(missing)",
        )
        .await;
        assert_err(&resp, "set_auto_compaction");
        assert_eq!(resp["error"], "Missing enabled");

        // set_auto_retry false
        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"3","type":"set_auto_retry","enabled":false}"#,
            "set_auto_retry(false)",
        )
        .await;
        assert_ok(&resp, "set_auto_retry");

        // set_auto_retry missing enabled
        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"4","type":"set_auto_retry"}"#,
            "set_auto_retry(missing)",
        )
        .await;
        assert_err(&resp, "set_auto_retry");
        assert_eq!(resp["error"], "Missing enabled");

        // abort_retry (always succeeds)
        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"5","type":"abort_retry"}"#,
            "abort_retry",
        )
        .await;
        assert_ok(&resp, "abort_retry");

        drop(in_tx);
        let result = server.await;
        assert!(result.is_ok(), "rpc server error: {result:?}");
    });
}

// ---------------------------------------------------------------------------
// Tests: Model / Thinking Level commands
// ---------------------------------------------------------------------------

#[test]
fn rpc_get_available_models_empty() {
    let harness = TestHarness::new("rpc_get_available_models_empty");
    let cassette_dir = cassette_root();
    let runtime = asupersync::runtime::RuntimeBuilder::current_thread()
        .build()
        .expect("build test runtime");
    let handle = runtime.handle();

    runtime.block_on(async move {
        let agent_session = build_agent_session(Session::in_memory(), &cassette_dir);
        let options = build_options(&handle, harness.temp_path("auth.json"), vec![], vec![]);
        let (in_tx, in_rx) = asupersync::channel::mpsc::channel::<String>(16);
        let (out_tx, out_rx) = std::sync::mpsc::channel::<String>();
        let out_rx = Arc::new(Mutex::new(out_rx));

        let server = handle.spawn(async move { run(agent_session, options, in_rx, out_tx).await });

        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"1","type":"get_available_models"}"#,
            "get_available_models",
        )
        .await;
        assert_ok(&resp, "get_available_models");
        let models = resp["data"]["models"].as_array().unwrap();
        assert!(models.is_empty(), "expected empty model list");

        drop(in_tx);
        let result = server.await;
        assert!(result.is_ok(), "rpc server error: {result:?}");
    });
}

#[test]
fn rpc_get_available_models_populated() {
    let harness = TestHarness::new("rpc_get_available_models_populated");
    let cassette_dir = cassette_root();
    let runtime = asupersync::runtime::RuntimeBuilder::current_thread()
        .build()
        .expect("build test runtime");
    let handle = runtime.handle();

    runtime.block_on(async move {
        let agent_session = build_agent_session(Session::in_memory(), &cassette_dir);
        let models = vec![
            test_entry("claude-opus-4-6", true),
            test_entry("gpt-4o", false),
        ];
        let options = build_options(&handle, harness.temp_path("auth.json"), models, vec![]);
        let (in_tx, in_rx) = asupersync::channel::mpsc::channel::<String>(16);
        let (out_tx, out_rx) = std::sync::mpsc::channel::<String>();
        let out_rx = Arc::new(Mutex::new(out_rx));

        let server = handle.spawn(async move { run(agent_session, options, in_rx, out_tx).await });

        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"1","type":"get_available_models"}"#,
            "get_available_models",
        )
        .await;
        assert_ok(&resp, "get_available_models");
        let models = resp["data"]["models"].as_array().unwrap();
        assert_eq!(models.len(), 2);
        assert_eq!(models[0]["id"], "claude-opus-4-6");
        assert_eq!(models[0]["reasoning"], true);
        assert_eq!(models[1]["id"], "gpt-4o");
        assert_eq!(models[1]["reasoning"], false);

        drop(in_tx);
        let result = server.await;
        assert!(result.is_ok(), "rpc server error: {result:?}");
    });
}

#[test]
fn rpc_set_thinking_level_success() {
    let harness = TestHarness::new("rpc_set_thinking_level_success");
    let cassette_dir = cassette_root();
    let runtime = asupersync::runtime::RuntimeBuilder::current_thread()
        .build()
        .expect("build test runtime");
    let handle = runtime.handle();

    runtime.block_on(async move {
        let agent_session = build_agent_session(Session::in_memory(), &cassette_dir);
        let options = build_options(&handle, harness.temp_path("auth.json"), vec![], vec![]);
        let (in_tx, in_rx) = asupersync::channel::mpsc::channel::<String>(16);
        let (out_tx, out_rx) = std::sync::mpsc::channel::<String>();
        let out_rx = Arc::new(Mutex::new(out_rx));

        let server = handle.spawn(async move { run(agent_session, options, in_rx, out_tx).await });

        // Set to high
        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"1","type":"set_thinking_level","level":"high"}"#,
            "set_thinking_level(high)",
        )
        .await;
        assert_ok(&resp, "set_thinking_level");

        // Set to off
        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"2","type":"set_thinking_level","level":"off"}"#,
            "set_thinking_level(off)",
        )
        .await;
        assert_ok(&resp, "set_thinking_level");

        drop(in_tx);
        let result = server.await;
        assert!(result.is_ok(), "rpc server error: {result:?}");
    });
}

#[test]
fn rpc_set_thinking_level_errors() {
    let harness = TestHarness::new("rpc_set_thinking_level_errors");
    let cassette_dir = cassette_root();
    let runtime = asupersync::runtime::RuntimeBuilder::current_thread()
        .build()
        .expect("build test runtime");
    let handle = runtime.handle();

    runtime.block_on(async move {
        let agent_session = build_agent_session(Session::in_memory(), &cassette_dir);
        let options = build_options(&handle, harness.temp_path("auth.json"), vec![], vec![]);
        let (in_tx, in_rx) = asupersync::channel::mpsc::channel::<String>(16);
        let (out_tx, out_rx) = std::sync::mpsc::channel::<String>();
        let out_rx = Arc::new(Mutex::new(out_rx));

        let server = handle.spawn(async move { run(agent_session, options, in_rx, out_tx).await });

        // Missing level
        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"1","type":"set_thinking_level"}"#,
            "set_thinking_level(missing)",
        )
        .await;
        assert_err(&resp, "set_thinking_level");
        assert_eq!(resp["error"], "Missing level");

        // Invalid level
        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"2","type":"set_thinking_level","level":"impossible"}"#,
            "set_thinking_level(impossible)",
        )
        .await;
        assert_err(&resp, "set_thinking_level");
        assert!(
            resp["error"].as_str().is_some_and(|s| !s.is_empty()),
            "expected error message"
        );

        drop(in_tx);
        let result = server.await;
        assert!(result.is_ok(), "rpc server error: {result:?}");
    });
}

// ---------------------------------------------------------------------------
// Tests: Session data commands
// ---------------------------------------------------------------------------

#[test]
fn rpc_get_messages_empty_session() {
    let harness = TestHarness::new("rpc_get_messages_empty_session");
    let cassette_dir = cassette_root();
    let runtime = asupersync::runtime::RuntimeBuilder::current_thread()
        .build()
        .expect("build test runtime");
    let handle = runtime.handle();

    runtime.block_on(async move {
        let agent_session = build_agent_session(Session::in_memory(), &cassette_dir);
        let options = build_options(&handle, harness.temp_path("auth.json"), vec![], vec![]);
        let (in_tx, in_rx) = asupersync::channel::mpsc::channel::<String>(16);
        let (out_tx, out_rx) = std::sync::mpsc::channel::<String>();
        let out_rx = Arc::new(Mutex::new(out_rx));

        let server = handle.spawn(async move { run(agent_session, options, in_rx, out_tx).await });

        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"1","type":"get_messages"}"#,
            "get_messages(empty)",
        )
        .await;
        assert_ok(&resp, "get_messages");
        let messages = resp["data"]["messages"].as_array().unwrap();
        assert!(messages.is_empty(), "expected empty messages");

        drop(in_tx);
        let result = server.await;
        assert!(result.is_ok(), "rpc server error: {result:?}");
    });
}

#[test]
fn rpc_get_messages_with_user_messages() {
    let harness = TestHarness::new("rpc_get_messages_with_user_messages");
    let cassette_dir = cassette_root();
    let runtime = asupersync::runtime::RuntimeBuilder::current_thread()
        .build()
        .expect("build test runtime");
    let handle = runtime.handle();

    runtime.block_on(async move {
        let now = 1_700_000_000_000i64;
        let mut session = Session::in_memory();
        session.append_message(SessionMessage::User {
            content: UserContent::Text("hello".to_string()),
            timestamp: Some(now),
        });
        session.append_message(SessionMessage::User {
            content: UserContent::Text("world".to_string()),
            timestamp: Some(now + 1000),
        });

        let agent_session = build_agent_session(session, &cassette_dir);
        let options = build_options(&handle, harness.temp_path("auth.json"), vec![], vec![]);
        let (in_tx, in_rx) = asupersync::channel::mpsc::channel::<String>(16);
        let (out_tx, out_rx) = std::sync::mpsc::channel::<String>();
        let out_rx = Arc::new(Mutex::new(out_rx));

        let server = handle.spawn(async move { run(agent_session, options, in_rx, out_tx).await });

        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"1","type":"get_messages"}"#,
            "get_messages(with_users)",
        )
        .await;
        assert_ok(&resp, "get_messages");
        let messages = resp["data"]["messages"].as_array().unwrap();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0]["role"], "user");
        assert_eq!(messages[1]["role"], "user");

        drop(in_tx);
        let result = server.await;
        assert!(result.is_ok(), "rpc server error: {result:?}");
    });
}

#[test]
fn rpc_get_last_assistant_text_empty() {
    let harness = TestHarness::new("rpc_get_last_assistant_text_empty");
    let cassette_dir = cassette_root();
    let runtime = asupersync::runtime::RuntimeBuilder::current_thread()
        .build()
        .expect("build test runtime");
    let handle = runtime.handle();

    runtime.block_on(async move {
        let agent_session = build_agent_session(Session::in_memory(), &cassette_dir);
        let options = build_options(&handle, harness.temp_path("auth.json"), vec![], vec![]);
        let (in_tx, in_rx) = asupersync::channel::mpsc::channel::<String>(16);
        let (out_tx, out_rx) = std::sync::mpsc::channel::<String>();
        let out_rx = Arc::new(Mutex::new(out_rx));

        let server = handle.spawn(async move { run(agent_session, options, in_rx, out_tx).await });

        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"1","type":"get_last_assistant_text"}"#,
            "get_last_assistant_text(empty)",
        )
        .await;
        assert_ok(&resp, "get_last_assistant_text");
        assert!(resp["data"]["text"].is_null(), "expected null text");

        drop(in_tx);
        let result = server.await;
        assert!(result.is_ok(), "rpc server error: {result:?}");
    });
}

#[test]
fn rpc_get_last_assistant_text_with_assistant() {
    let harness = TestHarness::new("rpc_get_last_assistant_text_with_assistant");
    let cassette_dir = cassette_root();
    let runtime = asupersync::runtime::RuntimeBuilder::current_thread()
        .build()
        .expect("build test runtime");
    let handle = runtime.handle();

    runtime.block_on(async move {
        let now = 1_700_000_000_000i64;
        let mut session = Session::in_memory();
        session.append_message(SessionMessage::User {
            content: UserContent::Text("hello".to_string()),
            timestamp: Some(now),
        });
        session.append_message(SessionMessage::Assistant {
            message: AssistantMessage {
                content: vec![ContentBlock::Text(TextContent::new("Hi there!"))],
                api: "test".to_string(),
                provider: "test".to_string(),
                model: "test-model".to_string(),
                usage: Usage::default(),
                stop_reason: StopReason::Stop,
                error_message: None,
                timestamp: now,
            },
        });

        let agent_session = build_agent_session(session, &cassette_dir);
        let options = build_options(&handle, harness.temp_path("auth.json"), vec![], vec![]);
        let (in_tx, in_rx) = asupersync::channel::mpsc::channel::<String>(16);
        let (out_tx, out_rx) = std::sync::mpsc::channel::<String>();
        let out_rx = Arc::new(Mutex::new(out_rx));

        let server = handle.spawn(async move { run(agent_session, options, in_rx, out_tx).await });

        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"1","type":"get_last_assistant_text"}"#,
            "get_last_assistant_text(with_assistant)",
        )
        .await;
        assert_ok(&resp, "get_last_assistant_text");
        assert_eq!(resp["data"]["text"], "Hi there!");

        drop(in_tx);
        let result = server.await;
        assert!(result.is_ok(), "rpc server error: {result:?}");
    });
}

#[test]
fn rpc_get_commands_empty() {
    let harness = TestHarness::new("rpc_get_commands_empty");
    let cassette_dir = cassette_root();
    let runtime = asupersync::runtime::RuntimeBuilder::current_thread()
        .build()
        .expect("build test runtime");
    let handle = runtime.handle();

    runtime.block_on(async move {
        let agent_session = build_agent_session(Session::in_memory(), &cassette_dir);
        let options = build_options(&handle, harness.temp_path("auth.json"), vec![], vec![]);
        let (in_tx, in_rx) = asupersync::channel::mpsc::channel::<String>(16);
        let (out_tx, out_rx) = std::sync::mpsc::channel::<String>();
        let out_rx = Arc::new(Mutex::new(out_rx));

        let server = handle.spawn(async move { run(agent_session, options, in_rx, out_tx).await });

        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"1","type":"get_commands"}"#,
            "get_commands",
        )
        .await;
        assert_ok(&resp, "get_commands");
        assert!(
            resp["data"]["commands"].is_array(),
            "commands should be an array"
        );

        drop(in_tx);
        let result = server.await;
        assert!(result.is_ok(), "rpc server error: {result:?}");
    });
}

// ---------------------------------------------------------------------------
// Tests: Session management commands
// ---------------------------------------------------------------------------

#[test]
fn rpc_set_session_name_success() {
    let harness = TestHarness::new("rpc_set_session_name_success");
    let cassette_dir = cassette_root();
    let runtime = asupersync::runtime::RuntimeBuilder::current_thread()
        .build()
        .expect("build test runtime");
    let handle = runtime.handle();

    runtime.block_on(async move {
        let agent_session = build_agent_session(Session::in_memory(), &cassette_dir);
        let options = build_options(&handle, harness.temp_path("auth.json"), vec![], vec![]);
        let (in_tx, in_rx) = asupersync::channel::mpsc::channel::<String>(16);
        let (out_tx, out_rx) = std::sync::mpsc::channel::<String>();
        let out_rx = Arc::new(Mutex::new(out_rx));

        let server = handle.spawn(async move { run(agent_session, options, in_rx, out_tx).await });

        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"1","type":"set_session_name","name":"Test Session"}"#,
            "set_session_name",
        )
        .await;
        assert_ok(&resp, "set_session_name");

        drop(in_tx);
        let result = server.await;
        assert!(result.is_ok(), "rpc server error: {result:?}");
    });
}

#[test]
fn rpc_set_session_name_missing_name() {
    let harness = TestHarness::new("rpc_set_session_name_missing_name");
    let cassette_dir = cassette_root();
    let runtime = asupersync::runtime::RuntimeBuilder::current_thread()
        .build()
        .expect("build test runtime");
    let handle = runtime.handle();

    runtime.block_on(async move {
        let agent_session = build_agent_session(Session::in_memory(), &cassette_dir);
        let options = build_options(&handle, harness.temp_path("auth.json"), vec![], vec![]);
        let (in_tx, in_rx) = asupersync::channel::mpsc::channel::<String>(16);
        let (out_tx, out_rx) = std::sync::mpsc::channel::<String>();
        let out_rx = Arc::new(Mutex::new(out_rx));

        let server = handle.spawn(async move { run(agent_session, options, in_rx, out_tx).await });

        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"1","type":"set_session_name"}"#,
            "set_session_name(missing)",
        )
        .await;
        assert_err(&resp, "set_session_name");
        assert_eq!(resp["error"], "Missing name");

        drop(in_tx);
        let result = server.await;
        assert!(result.is_ok(), "rpc server error: {result:?}");
    });
}

// ---------------------------------------------------------------------------
// Tests: Bash command
// ---------------------------------------------------------------------------

#[test]
fn rpc_bash_echo() {
    let harness = TestHarness::new("rpc_bash_echo");
    let cassette_dir = cassette_root();
    let runtime = asupersync::runtime::RuntimeBuilder::current_thread()
        .build()
        .expect("build test runtime");
    let handle = runtime.handle();

    runtime.block_on(async move {
        let agent_session = build_agent_session(Session::in_memory(), &cassette_dir);
        let options = build_options(&handle, harness.temp_path("auth.json"), vec![], vec![]);
        let (in_tx, in_rx) = asupersync::channel::mpsc::channel::<String>(16);
        let (out_tx, out_rx) = std::sync::mpsc::channel::<String>();
        let out_rx = Arc::new(Mutex::new(out_rx));

        let server = handle.spawn(async move { run(agent_session, options, in_rx, out_tx).await });

        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"1","type":"bash","command":"echo hello_rpc"}"#,
            "bash(echo)",
        )
        .await;
        assert_ok(&resp, "bash");
        assert_eq!(resp["data"]["exitCode"], 0);
        let output = resp["data"]["output"].as_str().unwrap_or("");
        assert!(
            output.contains("hello_rpc"),
            "bash output should contain hello_rpc, got: {output}"
        );

        drop(in_tx);
        let result = server.await;
        assert!(result.is_ok(), "rpc server error: {result:?}");
    });
}

#[test]
fn rpc_bash_missing_command() {
    let harness = TestHarness::new("rpc_bash_missing_command");
    let cassette_dir = cassette_root();
    let runtime = asupersync::runtime::RuntimeBuilder::current_thread()
        .build()
        .expect("build test runtime");
    let handle = runtime.handle();

    runtime.block_on(async move {
        let agent_session = build_agent_session(Session::in_memory(), &cassette_dir);
        let options = build_options(&handle, harness.temp_path("auth.json"), vec![], vec![]);
        let (in_tx, in_rx) = asupersync::channel::mpsc::channel::<String>(16);
        let (out_tx, out_rx) = std::sync::mpsc::channel::<String>();
        let out_rx = Arc::new(Mutex::new(out_rx));

        let server = handle.spawn(async move { run(agent_session, options, in_rx, out_tx).await });

        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"1","type":"bash"}"#,
            "bash(missing)",
        )
        .await;
        assert_err(&resp, "bash");
        assert_eq!(resp["error"], "Missing command");

        drop(in_tx);
        let result = server.await;
        assert!(result.is_ok(), "rpc server error: {result:?}");
    });
}

#[test]
fn rpc_bash_nonzero_exit() {
    let harness = TestHarness::new("rpc_bash_nonzero_exit");
    let cassette_dir = cassette_root();
    let runtime = asupersync::runtime::RuntimeBuilder::current_thread()
        .build()
        .expect("build test runtime");
    let handle = runtime.handle();

    runtime.block_on(async move {
        let agent_session = build_agent_session(Session::in_memory(), &cassette_dir);
        let options = build_options(&handle, harness.temp_path("auth.json"), vec![], vec![]);
        let (in_tx, in_rx) = asupersync::channel::mpsc::channel::<String>(16);
        let (out_tx, out_rx) = std::sync::mpsc::channel::<String>();
        let out_rx = Arc::new(Mutex::new(out_rx));

        let server = handle.spawn(async move { run(agent_session, options, in_rx, out_tx).await });

        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"1","type":"bash","command":"exit 42"}"#,
            "bash(exit 42)",
        )
        .await;
        assert_ok(&resp, "bash");
        assert_eq!(resp["data"]["exitCode"], 42);

        drop(in_tx);
        let result = server.await;
        assert!(result.is_ok(), "rpc server error: {result:?}");
    });
}

#[cfg(target_os = "linux")]
#[test]
fn rpc_abort_bash_kills_background_children() {
    let harness = TestHarness::new("rpc_abort_bash_kills_background_children");
    let cassette_dir = cassette_root();
    let runtime = asupersync::runtime::RuntimeBuilder::current_thread()
        .build()
        .expect("build test runtime");
    let handle = runtime.handle();

    runtime.block_on(async move {
        let agent_session = build_agent_session(Session::in_memory(), &cassette_dir);
        let options = build_options(&handle, harness.temp_path("auth.json"), vec![], vec![]);
        let (in_tx, in_rx) = asupersync::channel::mpsc::channel::<String>(16);
        let (out_tx, out_rx) = std::sync::mpsc::channel::<String>();
        let out_rx = Arc::new(Mutex::new(out_rx));

        let server = handle.spawn(async move { run(agent_session, options, in_rx, out_tx).await });

        let marker = harness.temp_path("rpc_bash_survived.txt");
        let marker_str = marker.to_string_lossy();
        let command = format!(
            r#"{{"id":"1","type":"bash","command":"(sleep 3; echo leaked > '{marker_str}') & sleep 30"}}"#
        );
        let cx = asupersync::Cx::for_testing();
        in_tx
            .send(&cx, command)
            .await
            .expect("send long-running bash");

        asupersync::time::sleep(
            asupersync::time::wall_now(),
            Duration::from_millis(100),
        )
        .await;

        in_tx
            .send(&cx, r#"{"id":"2","type":"abort_bash"}"#.to_string())
            .await
            .expect("send abort_bash");

        let mut abort_resp = None;
        let mut bash_resp = None;
        for label in ["rpc_abort_bash:first", "rpc_abort_bash:second"] {
            let resp = parse_response(
                &recv_line(&out_rx, label)
                    .await
                    .unwrap_or_else(|err| panic!("{err}")),
            );
            match (resp["command"].as_str(), resp["id"].as_str()) {
                (Some("abort_bash"), Some("2")) => abort_resp = Some(resp),
                (Some("bash"), Some("1")) => bash_resp = Some(resp),
                other => panic!("unexpected response ordering/content: {other:?}"),
            }
        }

        let abort_resp = abort_resp.expect("missing abort_bash response");
        assert_ok(&abort_resp, "abort_bash");
        let bash_resp = bash_resp.expect("missing bash response");
        assert_ok(&bash_resp, "bash");
        assert_eq!(bash_resp["id"], "1");
        assert_eq!(bash_resp["data"]["cancelled"], true);

        std::thread::sleep(Duration::from_secs(4));
        assert!(
            !marker.exists(),
            "background child survived rpc abort"
        );

        drop(in_tx);
        let result = server.await;
        assert!(result.is_ok(), "rpc server error: {result:?}");
    });
}

// ---------------------------------------------------------------------------
// Tests: Request ID handling
// ---------------------------------------------------------------------------

#[test]
fn rpc_request_id_preserved() {
    let harness = TestHarness::new("rpc_request_id_preserved");
    let cassette_dir = cassette_root();
    let runtime = asupersync::runtime::RuntimeBuilder::current_thread()
        .build()
        .expect("build test runtime");
    let handle = runtime.handle();

    runtime.block_on(async move {
        let agent_session = build_agent_session(Session::in_memory(), &cassette_dir);
        let options = build_options(&handle, harness.temp_path("auth.json"), vec![], vec![]);
        let (in_tx, in_rx) = asupersync::channel::mpsc::channel::<String>(16);
        let (out_tx, out_rx) = std::sync::mpsc::channel::<String>();
        let out_rx = Arc::new(Mutex::new(out_rx));

        let server = handle.spawn(async move { run(agent_session, options, in_rx, out_tx).await });

        // With string ID
        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"custom-id-123","type":"get_state"}"#,
            "get_state(with id)",
        )
        .await;
        assert_ok(&resp, "get_state");
        assert_eq!(resp["id"], "custom-id-123");

        // With numeric ID (RPC server uses as_str(), so numeric IDs are treated as absent)
        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":42,"type":"get_state"}"#,
            "get_state(numeric id)",
        )
        .await;
        assert_ok(&resp, "get_state");
        assert!(
            resp.get("id").is_none() || resp["id"].is_null(),
            "numeric IDs should be treated as absent (parsed via as_str)"
        );

        drop(in_tx);
        let result = server.await;
        assert!(result.is_ok(), "rpc server error: {result:?}");
    });
}

#[test]
fn rpc_request_without_id() {
    let harness = TestHarness::new("rpc_request_without_id");
    let cassette_dir = cassette_root();
    let runtime = asupersync::runtime::RuntimeBuilder::current_thread()
        .build()
        .expect("build test runtime");
    let handle = runtime.handle();

    runtime.block_on(async move {
        let agent_session = build_agent_session(Session::in_memory(), &cassette_dir);
        let options = build_options(&handle, harness.temp_path("auth.json"), vec![], vec![]);
        let (in_tx, in_rx) = asupersync::channel::mpsc::channel::<String>(16);
        let (out_tx, out_rx) = std::sync::mpsc::channel::<String>();
        let out_rx = Arc::new(Mutex::new(out_rx));

        let server = handle.spawn(async move { run(agent_session, options, in_rx, out_tx).await });

        // Request without id field
        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"type":"get_state"}"#,
            "get_state(no id)",
        )
        .await;
        assert_ok(&resp, "get_state");
        // id should be null or absent
        assert!(
            resp.get("id").is_none() || resp["id"].is_null(),
            "expected no id or null id"
        );

        drop(in_tx);
        let result = server.await;
        assert!(result.is_ok(), "rpc server error: {result:?}");
    });
}

// ---------------------------------------------------------------------------
// Tests: Multiple rapid commands
// ---------------------------------------------------------------------------

#[test]
fn rpc_rapid_sequence_of_sync_commands() {
    let harness = TestHarness::new("rpc_rapid_sequence_of_sync_commands");
    let cassette_dir = cassette_root();
    let runtime = asupersync::runtime::RuntimeBuilder::current_thread()
        .build()
        .expect("build test runtime");
    let handle = runtime.handle();

    runtime.block_on(async move {
        let models = vec![test_entry("test-model", false)];
        let agent_session = build_agent_session(Session::in_memory(), &cassette_dir);
        let options = build_options(&handle, harness.temp_path("auth.json"), models, vec![]);
        let (in_tx, in_rx) = asupersync::channel::mpsc::channel::<String>(32);
        let (out_tx, out_rx) = std::sync::mpsc::channel::<String>();
        let out_rx = Arc::new(Mutex::new(out_rx));

        let server = handle.spawn(async move { run(agent_session, options, in_rx, out_tx).await });

        let cx = asupersync::Cx::for_testing();

        // Fire 8 commands rapidly.
        let commands = [
            (r#"{"id":"1","type":"get_state"}"#, "get_state"),
            (
                r#"{"id":"2","type":"get_available_models"}"#,
                "get_available_models",
            ),
            (r#"{"id":"3","type":"get_messages"}"#, "get_messages"),
            (
                r#"{"id":"4","type":"get_session_stats"}"#,
                "get_session_stats",
            ),
            (r#"{"id":"5","type":"get_commands"}"#, "get_commands"),
            (
                r#"{"id":"6","type":"get_last_assistant_text"}"#,
                "get_last_assistant_text",
            ),
            (
                r#"{"id":"7","type":"set_auto_compaction","enabled":true}"#,
                "set_auto_compaction",
            ),
            (
                r#"{"id":"8","type":"set_auto_retry","enabled":false}"#,
                "set_auto_retry",
            ),
        ];

        for (cmd, _label) in &commands {
            in_tx
                .send(&cx, cmd.to_string())
                .await
                .expect("send rapid command");
        }

        // Collect all 8 responses.
        let mut responses = Vec::new();
        for (_, label) in &commands {
            let line = recv_line(&out_rx, label)
                .await
                .unwrap_or_else(|err| panic!("{err}"));
            responses.push(parse_response(&line));
        }

        // Verify each response matches its command.
        for (i, (_, expected_cmd)) in commands.iter().enumerate() {
            assert_ok(&responses[i], expected_cmd);
            assert_eq!(
                responses[i]["id"],
                serde_json::Value::String((i + 1).to_string())
            );
        }

        drop(in_tx);
        let result = server.await;
        assert!(result.is_ok(), "rpc server error: {result:?}");
    });
}

// ---------------------------------------------------------------------------
// Tests: State reflection after mutations
// ---------------------------------------------------------------------------

#[test]
fn rpc_get_state_reflects_session_stats() {
    let harness = TestHarness::new("rpc_get_state_reflects_session_stats");
    let cassette_dir = cassette_root();
    let runtime = asupersync::runtime::RuntimeBuilder::current_thread()
        .build()
        .expect("build test runtime");
    let handle = runtime.handle();

    runtime.block_on(async move {
        let now = 1_700_000_000_000i64;
        let mut session = Session::in_memory();
        session.header.provider = Some("anthropic".to_string());
        session.header.model_id = Some("claude-opus-4-6".to_string());
        session.append_message(SessionMessage::User {
            content: UserContent::Text("hello".to_string()),
            timestamp: Some(now),
        });
        session.append_message(SessionMessage::Assistant {
            message: AssistantMessage {
                content: vec![ContentBlock::Text(TextContent::new("world"))],
                api: "test".to_string(),
                provider: "anthropic".to_string(),
                model: "claude-opus-4-6".to_string(),
                usage: Usage {
                    input: 10,
                    output: 5,
                    total_tokens: 15,
                    ..Usage::default()
                },
                stop_reason: StopReason::Stop,
                error_message: None,
                timestamp: now,
            },
        });

        let agent_session = build_agent_session(session, &cassette_dir);
        let options = build_options(&handle, harness.temp_path("auth.json"), vec![], vec![]);
        let (in_tx, in_rx) = asupersync::channel::mpsc::channel::<String>(16);
        let (out_tx, out_rx) = std::sync::mpsc::channel::<String>();
        let out_rx = Arc::new(Mutex::new(out_rx));

        let server = handle.spawn(async move { run(agent_session, options, in_rx, out_tx).await });

        // get_session_stats should reflect pre-populated messages
        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"1","type":"get_session_stats"}"#,
            "get_session_stats",
        )
        .await;
        assert_ok(&resp, "get_session_stats");
        assert_eq!(resp["data"]["userMessages"], 1);
        assert_eq!(resp["data"]["assistantMessages"], 1);
        assert_eq!(resp["data"]["totalMessages"], 2);
        assert_eq!(resp["data"]["tokens"]["input"], 10);
        assert_eq!(resp["data"]["tokens"]["output"], 5);
        assert_eq!(resp["data"]["tokens"]["total"], 15);

        // get_messages should return the 2 messages
        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"2","type":"get_messages"}"#,
            "get_messages",
        )
        .await;
        assert_ok(&resp, "get_messages");
        let msgs = resp["data"]["messages"].as_array().unwrap();
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0]["role"], "user");
        assert_eq!(msgs[1]["role"], "assistant");

        // get_last_assistant_text should return "world"
        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"3","type":"get_last_assistant_text"}"#,
            "get_last_assistant_text",
        )
        .await;
        assert_ok(&resp, "get_last_assistant_text");
        assert_eq!(resp["data"]["text"], "world");

        drop(in_tx);
        let result = server.await;
        assert!(result.is_ok(), "rpc server error: {result:?}");
    });
}

// ---------------------------------------------------------------------------
// Tests: Error path coverage
// ---------------------------------------------------------------------------

#[test]
fn rpc_prompt_missing_message() {
    let harness = TestHarness::new("rpc_prompt_missing_message");
    let cassette_dir = cassette_root();
    let runtime = asupersync::runtime::RuntimeBuilder::current_thread()
        .build()
        .expect("build test runtime");
    let handle = runtime.handle();

    runtime.block_on(async move {
        let agent_session = build_agent_session(Session::in_memory(), &cassette_dir);
        let options = build_options(&handle, harness.temp_path("auth.json"), vec![], vec![]);
        let (in_tx, in_rx) = asupersync::channel::mpsc::channel::<String>(16);
        let (out_tx, out_rx) = std::sync::mpsc::channel::<String>();
        let out_rx = Arc::new(Mutex::new(out_rx));

        let server = handle.spawn(async move { run(agent_session, options, in_rx, out_tx).await });

        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"1","type":"prompt"}"#,
            "prompt(missing message)",
        )
        .await;
        assert_err(&resp, "prompt");
        assert_eq!(resp["error"], "Missing message");

        drop(in_tx);
        let result = server.await;
        assert!(result.is_ok(), "rpc server error: {result:?}");
    });
}

#[test]
fn rpc_prompt_dispatches_registered_extension_command() {
    let harness = TestHarness::new("rpc_prompt_dispatches_registered_extension_command");
    let cassette_dir = cassette_root();
    let runtime = asupersync::runtime::RuntimeBuilder::current_thread()
        .build()
        .expect("build test runtime");
    let handle = runtime.handle();

    runtime.block_on(async move {
        let (agent_session, _manager) = build_agent_session_with_js_extension(
            Session::in_memory(),
            &cassette_dir,
            &harness,
            RPC_PROMPT_EXTENSION_COMMAND_EXT,
        )
        .await;
        let options = build_options(&handle, harness.temp_path("auth.json"), vec![], vec![]);
        let (in_tx, in_rx) = asupersync::channel::mpsc::channel::<String>(16);
        let (out_tx, out_rx) = std::sync::mpsc::channel::<String>();
        let out_rx = Arc::new(Mutex::new(out_rx));

        let server = handle.spawn(async move { run(agent_session, options, in_rx, out_tx).await });

        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"1","type":"prompt","message":"/emit-now"}"#,
            "prompt(extension command)",
        )
        .await;
        assert_ok(&resp, "prompt");

        let messages = wait_for_custom_message(
            &in_tx,
            &out_rx,
            "rpc-note",
            "rpc-message",
            "get_messages(after extension command)",
        )
        .await;
        let messages = messages["data"]["messages"]
            .as_array()
            .expect("messages array");
        assert!(
            messages.iter().any(|message| {
                message["role"] == "custom"
                    && message["customType"] == "rpc-note"
                    && message["content"] == "rpc-message"
            }),
            "expected RPC prompt command to append custom message, got {messages:?}"
        );

        drop(in_tx);
        let result = server.await;
        assert!(result.is_ok(), "rpc server error: {result:?}");
    });
}

#[test]
fn rpc_steer_missing_message() {
    let harness = TestHarness::new("rpc_steer_missing_message");
    let cassette_dir = cassette_root();
    let runtime = asupersync::runtime::RuntimeBuilder::current_thread()
        .build()
        .expect("build test runtime");
    let handle = runtime.handle();

    runtime.block_on(async move {
        let agent_session = build_agent_session(Session::in_memory(), &cassette_dir);
        let options = build_options(&handle, harness.temp_path("auth.json"), vec![], vec![]);
        let (in_tx, in_rx) = asupersync::channel::mpsc::channel::<String>(16);
        let (out_tx, out_rx) = std::sync::mpsc::channel::<String>();
        let out_rx = Arc::new(Mutex::new(out_rx));

        let server = handle.spawn(async move { run(agent_session, options, in_rx, out_tx).await });

        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"1","type":"steer"}"#,
            "steer(missing message)",
        )
        .await;
        assert_err(&resp, "steer");
        assert_eq!(resp["error"], "Missing message");

        drop(in_tx);
        let result = server.await;
        assert!(result.is_ok(), "rpc server error: {result:?}");
    });
}

#[test]
fn rpc_follow_up_missing_message() {
    let harness = TestHarness::new("rpc_follow_up_missing_message");
    let cassette_dir = cassette_root();
    let runtime = asupersync::runtime::RuntimeBuilder::current_thread()
        .build()
        .expect("build test runtime");
    let handle = runtime.handle();

    runtime.block_on(async move {
        let agent_session = build_agent_session(Session::in_memory(), &cassette_dir);
        let options = build_options(&handle, harness.temp_path("auth.json"), vec![], vec![]);
        let (in_tx, in_rx) = asupersync::channel::mpsc::channel::<String>(16);
        let (out_tx, out_rx) = std::sync::mpsc::channel::<String>();
        let out_rx = Arc::new(Mutex::new(out_rx));

        let server = handle.spawn(async move { run(agent_session, options, in_rx, out_tx).await });

        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"1","type":"follow_up"}"#,
            "follow_up(missing message)",
        )
        .await;
        assert_err(&resp, "follow_up");
        assert_eq!(resp["error"], "Missing message");

        drop(in_tx);
        let result = server.await;
        assert!(result.is_ok(), "rpc server error: {result:?}");
    });
}

#[test]
fn rpc_set_model_missing_model_id() {
    let harness = TestHarness::new("rpc_set_model_missing_model_id");
    let cassette_dir = cassette_root();
    let runtime = asupersync::runtime::RuntimeBuilder::current_thread()
        .build()
        .expect("build test runtime");
    let handle = runtime.handle();

    runtime.block_on(async move {
        let agent_session = build_agent_session(Session::in_memory(), &cassette_dir);
        let options = build_options(&handle, harness.temp_path("auth.json"), vec![], vec![]);
        let (in_tx, in_rx) = asupersync::channel::mpsc::channel::<String>(16);
        let (out_tx, out_rx) = std::sync::mpsc::channel::<String>();
        let out_rx = Arc::new(Mutex::new(out_rx));

        let server = handle.spawn(async move { run(agent_session, options, in_rx, out_tx).await });

        // Missing provider
        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"1","type":"set_model","modelId":"x"}"#,
            "set_model(missing provider)",
        )
        .await;
        assert_err(&resp, "set_model");
        assert_eq!(resp["error"], "Missing provider");

        // Missing modelId
        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"2","type":"set_model","provider":"anthropic"}"#,
            "set_model(missing modelId)",
        )
        .await;
        assert_err(&resp, "set_model");
        assert_eq!(resp["error"], "Missing modelId");

        // Model not found
        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"3","type":"set_model","provider":"anthropic","modelId":"nonexistent"}"#,
            "set_model(not found)",
        )
        .await;
        assert_err(&resp, "set_model");
        assert!(
            resp["error"]
                .as_str()
                .is_some_and(|s| s.contains("Model not found")),
            "expected model not found error: {resp}"
        );

        drop(in_tx);
        let result = server.await;
        assert!(result.is_ok(), "rpc server error: {result:?}");
    });
}

#[test]
fn rpc_fork_missing_entry_id() {
    let harness = TestHarness::new("rpc_fork_missing_entry_id");
    let cassette_dir = cassette_root();
    let runtime = asupersync::runtime::RuntimeBuilder::current_thread()
        .build()
        .expect("build test runtime");
    let handle = runtime.handle();

    runtime.block_on(async move {
        let agent_session = build_agent_session(Session::in_memory(), &cassette_dir);
        let options = build_options(&handle, harness.temp_path("auth.json"), vec![], vec![]);
        let (in_tx, in_rx) = asupersync::channel::mpsc::channel::<String>(16);
        let (out_tx, out_rx) = std::sync::mpsc::channel::<String>();
        let out_rx = Arc::new(Mutex::new(out_rx));

        let server = handle.spawn(async move { run(agent_session, options, in_rx, out_tx).await });

        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"1","type":"fork"}"#,
            "fork(missing entryId)",
        )
        .await;
        assert_err(&resp, "fork");
        assert_eq!(resp["error"], "Missing entryId");

        drop(in_tx);
        let result = server.await;
        assert!(result.is_ok(), "rpc server error: {result:?}");
    });
}

#[test]
fn rpc_export_html_empty_session() {
    let harness = TestHarness::new("rpc_export_html_empty_session");
    let cassette_dir = cassette_root();
    let runtime = asupersync::runtime::RuntimeBuilder::current_thread()
        .build()
        .expect("build test runtime");
    let handle = runtime.handle();

    runtime.block_on(async move {
        let agent_session = build_agent_session(Session::in_memory(), &cassette_dir);
        let options = build_options(&handle, harness.temp_path("auth.json"), vec![], vec![]);
        let (in_tx, in_rx) = asupersync::channel::mpsc::channel::<String>(16);
        let (out_tx, out_rx) = std::sync::mpsc::channel::<String>();
        let out_rx = Arc::new(Mutex::new(out_rx));

        let server = handle.spawn(async move { run(agent_session, options, in_rx, out_tx).await });

        let output = harness.temp_path("export.html");
        let cmd = serde_json::json!({
            "id": "1",
            "type": "export_html",
            "outputPath": output.display().to_string()
        })
        .to_string();
        let resp = send_recv(&in_tx, &out_rx, &cmd, "export_html").await;
        assert_ok(&resp, "export_html");
        assert!(resp["data"]["path"].is_string(), "should return path");

        drop(in_tx);
        let result = server.await;
        assert!(result.is_ok(), "rpc server error: {result:?}");
    });
}

// ---------------------------------------------------------------------------
// Tests: abort (when not streaming)
// ---------------------------------------------------------------------------

#[test]
fn rpc_abort_when_idle() {
    let harness = TestHarness::new("rpc_abort_when_idle");
    let cassette_dir = cassette_root();
    let runtime = asupersync::runtime::RuntimeBuilder::current_thread()
        .build()
        .expect("build test runtime");
    let handle = runtime.handle();

    runtime.block_on(async move {
        let agent_session = build_agent_session(Session::in_memory(), &cassette_dir);
        let options = build_options(&handle, harness.temp_path("auth.json"), vec![], vec![]);
        let (in_tx, in_rx) = asupersync::channel::mpsc::channel::<String>(16);
        let (out_tx, out_rx) = std::sync::mpsc::channel::<String>();
        let out_rx = Arc::new(Mutex::new(out_rx));

        let server = handle.spawn(async move { run(agent_session, options, in_rx, out_tx).await });

        // Abort when nothing is streaming should still succeed.
        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"1","type":"abort"}"#,
            "abort(idle)",
        )
        .await;
        assert_ok(&resp, "abort");

        // abort_bash when nothing is running should also succeed.
        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"2","type":"abort_bash"}"#,
            "abort_bash(idle)",
        )
        .await;
        assert_ok(&resp, "abort_bash");

        drop(in_tx);
        let result = server.await;
        assert!(result.is_ok(), "rpc server error: {result:?}");
    });
}

// ---------------------------------------------------------------------------
// Helpers: Extension UI roundtrip
// ---------------------------------------------------------------------------

/// Build an `AgentSession` with an `ExtensionManager` attached so the RPC
/// server sets up the extension UI channel.  Returns both the session (to
/// hand to `run()`) and a cloned `ExtensionManager` the test can use to
/// call `request_ui()` / `respond_ui()`.
fn build_agent_session_with_extensions(
    session: Session,
    cassette_dir: &Path,
) -> (AgentSession, ExtensionManager) {
    let manager = ExtensionManager::default();
    let region = ExtensionRegion::new(manager.clone());
    let mut agent_session = build_agent_session(session, cassette_dir);
    agent_session.extensions = Some(region);
    (agent_session, manager)
}

async fn build_agent_session_with_js_extension(
    session: Session,
    cassette_dir: &Path,
    harness: &TestHarness,
    source: &str,
) -> (AgentSession, ExtensionManager) {
    let cwd = harness.temp_dir().to_path_buf();
    let ext_entry_path = harness.create_file("extensions/ext.mjs", source.as_bytes());
    let mut agent_session = build_agent_session(session, cassette_dir);
    agent_session
        .enable_extensions(&[], &cwd, None, &[ext_entry_path])
        .await
        .expect("enable extensions");
    let manager = agent_session
        .extensions
        .as_ref()
        .expect("extension region")
        .manager()
        .clone();
    (agent_session, manager)
}

const SESSION_SWITCH_CANCEL_EXT: &str = r#"
export default function init(pi) {
    pi.on("session_before_switch", () => ({ cancelled: true }));
}
"#;

const SESSION_SWITCH_RECORD_EXT: &str = r#"
export default function init(pi) {
    const events = [];

    pi.on("session_before_switch", () => ({ cancelled: false }));
    pi.on("session_switch", (event) => {
        events.push(event);
        return null;
    });

    pi.registerCommand("get-events", {
        description: "Return recorded session switch events",
        handler: async () => JSON.stringify(events),
    });
}
"#;

const RPC_PROMPT_EXTENSION_COMMAND_EXT: &str = r#"
export default function init(pi) {
    pi.registerCommand("emit-now", {
        description: "Record a custom message through RPC prompt dispatch",
        handler: async () => {
            await pi.events("sendMessage", {
                message: {
                    customType: "rpc-note",
                    content: "rpc-message",
                    display: true
                },
                options: { triggerTurn: false }
            });
            return "queued";
        }
    });
}
"#;

/// Wait for an `extension_ui_request` event on the RPC output channel.
/// Skips any non-event / non-ui-request lines.
async fn recv_ui_request(out_rx: &Arc<Mutex<Receiver<String>>>, label: &str) -> Value {
    let start = Instant::now();
    loop {
        let recv_result = {
            let rx = out_rx.lock().expect("lock rpc output receiver");
            rx.try_recv()
        };

        match recv_result {
            Ok(line) => {
                if let Ok(val) = serde_json::from_str::<Value>(&line) {
                    if val.get("type").and_then(Value::as_str) == Some("extension_ui_request") {
                        return val;
                    }
                }
                // Not our event — keep waiting.
            }
            Err(TryRecvError::Disconnected) => {
                panic!(
                    "{label}: output channel disconnected while waiting for extension_ui_request"
                );
            }
            Err(TryRecvError::Empty) => {}
        }

        assert!(
            start.elapsed() <= Duration::from_secs(10),
            "{label}: timed out waiting for extension_ui_request"
        );
        asupersync::time::sleep(asupersync::time::wall_now(), Duration::from_millis(5)).await;
    }
}

async fn wait_for_custom_message(
    in_tx: &asupersync::channel::mpsc::Sender<String>,
    out_rx: &Arc<Mutex<Receiver<String>>>,
    custom_type: &str,
    content: &str,
    label: &str,
) -> Value {
    let start = Instant::now();
    let mut attempt = 0usize;

    loop {
        attempt = attempt.saturating_add(1);
        let cmd = json!({
            "id": format!("wait-{attempt}"),
            "type": "get_messages",
        })
        .to_string();
        let resp = send_recv(in_tx, out_rx, &cmd, label).await;
        assert_ok(&resp, "get_messages");

        if resp["data"]["messages"].as_array().is_some_and(|messages| {
            messages.iter().any(|message| {
                message["role"] == "custom"
                    && message["customType"] == custom_type
                    && message["content"] == content
            })
        }) {
            return resp;
        }

        assert!(
            start.elapsed() <= Duration::from_secs(10),
            "{label}: timed out waiting for custom message {custom_type}/{content}"
        );
        asupersync::time::sleep(asupersync::time::wall_now(), Duration::from_millis(10)).await;
    }
}

// ---------------------------------------------------------------------------
// Tests: RPC session switch hooks
// ---------------------------------------------------------------------------

#[test]
fn rpc_new_session_can_be_cancelled_by_extension() {
    let harness = TestHarness::new("rpc_new_session_can_be_cancelled_by_extension");
    let cassette_dir = cassette_root();
    let runtime = asupersync::runtime::RuntimeBuilder::current_thread()
        .build()
        .expect("build test runtime");
    let handle = runtime.handle();

    runtime.block_on(async move {
        let session = Session::in_memory();
        let original_session_id = session.header.id.clone();
        let (agent_session, _manager) = build_agent_session_with_js_extension(
            session,
            &cassette_dir,
            &harness,
            SESSION_SWITCH_CANCEL_EXT,
        )
        .await;
        let options = build_options(&handle, harness.temp_path("auth.json"), vec![], vec![]);
        let (in_tx, in_rx) = asupersync::channel::mpsc::channel::<String>(16);
        let (out_tx, out_rx) = std::sync::mpsc::channel::<String>();
        let out_rx = Arc::new(Mutex::new(out_rx));

        let server = handle.spawn(async move { run(agent_session, options, in_rx, out_tx).await });

        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"1","type":"new_session"}"#,
            "new_session(cancelled)",
        )
        .await;
        assert_ok(&resp, "new_session");
        assert_eq!(resp["data"]["cancelled"], true);

        let state = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"2","type":"get_state"}"#,
            "get_state(after cancelled new_session)",
        )
        .await;
        assert_ok(&state, "get_state");
        assert_eq!(state["data"]["sessionId"], original_session_id);

        drop(in_tx);
        let result = server.await;
        assert!(result.is_ok(), "rpc server error: {result:?}");
    });
}

#[test]
fn rpc_switch_session_can_be_cancelled_by_extension() {
    let harness = TestHarness::new("rpc_switch_session_can_be_cancelled_by_extension");
    let cassette_dir = cassette_root();
    let runtime = asupersync::runtime::RuntimeBuilder::current_thread()
        .build()
        .expect("build test runtime");
    let handle = runtime.handle();

    runtime.block_on(async move {
        let session = Session::in_memory();
        let original_session_id = session.header.id.clone();

        let target_root = tempfile::tempdir().expect("target session dir");
        let mut target = Session::create_with_dir(Some(target_root.path().to_path_buf()));
        target.append_message(SessionMessage::User {
            content: UserContent::Text("target".to_string()),
            timestamp: Some(1_700_000_000_000),
        });
        target.save().await.expect("save target session");
        let target_path = target
            .path
            .as_ref()
            .map(|p| p.display().to_string())
            .expect("target session path");

        let (agent_session, _manager) = build_agent_session_with_js_extension(
            session,
            &cassette_dir,
            &harness,
            SESSION_SWITCH_CANCEL_EXT,
        )
        .await;
        let options = build_options(&handle, harness.temp_path("auth.json"), vec![], vec![]);
        let (in_tx, in_rx) = asupersync::channel::mpsc::channel::<String>(16);
        let (out_tx, out_rx) = std::sync::mpsc::channel::<String>();
        let out_rx = Arc::new(Mutex::new(out_rx));

        let server = handle.spawn(async move { run(agent_session, options, in_rx, out_tx).await });

        let cmd = json!({
            "id": "1",
            "type": "switch_session",
            "sessionPath": target_path,
        })
        .to_string();
        let resp = send_recv(&in_tx, &out_rx, &cmd, "switch_session(cancelled)").await;
        assert_ok(&resp, "switch_session");
        assert_eq!(resp["data"]["cancelled"], true);

        let state = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"2","type":"get_state"}"#,
            "get_state(after cancelled switch_session)",
        )
        .await;
        assert_ok(&state, "get_state");
        assert_eq!(state["data"]["sessionId"], original_session_id);

        drop(in_tx);
        let result = server.await;
        assert!(result.is_ok(), "rpc server error: {result:?}");
    });
}

#[test]
fn rpc_session_switch_events_are_emitted_for_new_and_resume() {
    let harness = TestHarness::new("rpc_session_switch_events_are_emitted_for_new_and_resume");
    let cassette_dir = cassette_root();
    let runtime = asupersync::runtime::RuntimeBuilder::current_thread()
        .build()
        .expect("build test runtime");
    let handle = runtime.handle();

    runtime.block_on(async move {
        let current_root = tempfile::tempdir().expect("current session dir");
        let mut current = Session::create_with_dir(Some(current_root.path().to_path_buf()));
        current.append_message(SessionMessage::User {
            content: UserContent::Text("current".to_string()),
            timestamp: Some(1_700_000_000_000),
        });
        current.save().await.expect("save current session");
        let current_path = current
            .path
            .as_ref()
            .map(|p| p.display().to_string())
            .expect("current session path");

        let target_root = tempfile::tempdir().expect("target session dir");
        let mut target = Session::create_with_dir(Some(target_root.path().to_path_buf()));
        target.append_message(SessionMessage::User {
            content: UserContent::Text("target".to_string()),
            timestamp: Some(1_700_000_000_100),
        });
        target.save().await.expect("save target session");
        let target_path = target
            .path
            .as_ref()
            .map(|p| p.display().to_string())
            .expect("target session path");

        let (agent_session, manager) = build_agent_session_with_js_extension(
            current,
            &cassette_dir,
            &harness,
            SESSION_SWITCH_RECORD_EXT,
        )
        .await;
        let options = build_options(&handle, harness.temp_path("auth.json"), vec![], vec![]);
        let (in_tx, in_rx) = asupersync::channel::mpsc::channel::<String>(16);
        let (out_tx, out_rx) = std::sync::mpsc::channel::<String>();
        let out_rx = Arc::new(Mutex::new(out_rx));

        let server = handle.spawn(async move { run(agent_session, options, in_rx, out_tx).await });

        let new_resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"1","type":"new_session"}"#,
            "new_session(recorded)",
        )
        .await;
        assert_ok(&new_resp, "new_session");
        assert_eq!(new_resp["data"]["cancelled"], false);

        let switch_cmd = json!({
            "id": "2",
            "type": "switch_session",
            "sessionPath": target_path.clone(),
        })
        .to_string();
        let switch_resp = send_recv(&in_tx, &out_rx, &switch_cmd, "switch_session(recorded)").await;
        assert_ok(&switch_resp, "switch_session");
        assert_eq!(switch_resp["data"]["cancelled"], false);

        let events_json = manager
            .execute_command("get-events", "", 5000)
            .await
            .expect("get recorded events");
        let events: Vec<Value> = serde_json::from_str(events_json.as_str().expect("events string"))
            .expect("parse events");

        assert_eq!(
            events.len(),
            2,
            "expected new + resume events, got {events:?}"
        );
        assert_eq!(events[0]["reason"], "new");
        assert_eq!(events[0]["previousSessionFile"], current_path);
        assert!(
            events[0]["sessionId"]
                .as_str()
                .is_some_and(|value| !value.is_empty()),
            "expected new-session event to include sessionId: {:?}",
            events[0]
        );

        assert_eq!(events[1]["reason"], "resume");
        assert_eq!(events[1]["previousSessionFile"], Value::Null);
        assert_eq!(events[1]["targetSessionFile"], target_path);
        assert!(
            events[1]["sessionId"]
                .as_str()
                .is_some_and(|value| !value.is_empty()),
            "expected resume event to include sessionId: {:?}",
            events[1]
        );

        drop(in_tx);
        let result = server.await;
        assert!(result.is_ok(), "rpc server error: {result:?}");
    });
}

// ---------------------------------------------------------------------------
// Tests: Extension UI roundtrip — confirm
// ---------------------------------------------------------------------------

#[test]
fn rpc_extension_ui_confirm_roundtrip() {
    let _harness = TestHarness::new("rpc_extension_ui_confirm_roundtrip");
    let cassette_dir = cassette_root();
    let runtime = asupersync::runtime::RuntimeBuilder::current_thread()
        .build()
        .expect("build test runtime");
    let handle = runtime.handle();

    runtime.block_on(async move {
        let (agent_session, manager) =
            build_agent_session_with_extensions(Session::in_memory(), &cassette_dir);
        let options = build_options(
            &handle,
            PathBuf::from("/tmp/auth_ui_confirm.json"),
            vec![],
            vec![],
        );
        let (in_tx, in_rx) = asupersync::channel::mpsc::channel::<String>(16);
        let (out_tx, out_rx) = std::sync::mpsc::channel::<String>();
        let out_rx = Arc::new(Mutex::new(out_rx));

        let server = handle.spawn(async move { run(agent_session, options, in_rx, out_tx).await });

        // Give the server a moment to set up the UI channel.
        asupersync::time::sleep(asupersync::time::wall_now(), Duration::from_millis(50)).await;

        // Spawn a request_ui call from the extension side.
        let mgr = manager.clone();
        let ui_task = handle.spawn(async move {
            let request = ExtensionUiRequest {
                id: "req-confirm-1".to_string(),
                method: "confirm".to_string(),
                payload: json!({
                    "title": "Delete file?",
                    "message": "This cannot be undone.",
                    "timeout": 5000
                }),
                timeout_ms: Some(5000),
                extension_id: Some("test-ext".to_string()),
            };
            mgr.request_ui(request).await
        });

        // Capture the emitted extension_ui_request event.
        let ui_event = recv_ui_request(&out_rx, "confirm").await;
        assert_eq!(ui_event["type"], "extension_ui_request");
        assert_eq!(ui_event["id"], "req-confirm-1");
        assert_eq!(ui_event["method"], "confirm");
        assert_eq!(ui_event["title"], "Delete file?");

        // Respond with confirmed = true.
        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"cmd-1","type":"extension_ui_response","requestId":"req-confirm-1","confirmed":true}"#,
            "confirm_response",
        )
        .await;
        assert_ok(&resp, "extension_ui_response");
        assert_eq!(resp["data"]["resolved"], true);

        // Verify the request_ui future resolved with the correct value.
        let ui_result = ui_task.await;
        let response = ui_result.expect("request_ui should succeed");
        let response = response.expect("should have a response");
        assert!(!response.cancelled);
        assert_eq!(response.value, Some(json!(true)));

        drop(in_tx);
        let result = server.await;
        assert!(result.is_ok(), "rpc server error: {result:?}");
    });
}

#[test]
fn rpc_extension_ui_confirm_denied() {
    let _harness = TestHarness::new("rpc_extension_ui_confirm_denied");
    let cassette_dir = cassette_root();
    let runtime = asupersync::runtime::RuntimeBuilder::current_thread()
        .build()
        .expect("build test runtime");
    let handle = runtime.handle();

    runtime.block_on(async move {
        let (agent_session, manager) =
            build_agent_session_with_extensions(Session::in_memory(), &cassette_dir);
        let options = build_options(
            &handle,
            PathBuf::from("/tmp/auth_ui_deny.json"),
            vec![],
            vec![],
        );
        let (in_tx, in_rx) = asupersync::channel::mpsc::channel::<String>(16);
        let (out_tx, out_rx) = std::sync::mpsc::channel::<String>();
        let out_rx = Arc::new(Mutex::new(out_rx));

        let server = handle.spawn(async move { run(agent_session, options, in_rx, out_tx).await });
        asupersync::time::sleep(asupersync::time::wall_now(), Duration::from_millis(50)).await;

        let mgr = manager.clone();
        let ui_task = handle.spawn(async move {
            let request = ExtensionUiRequest {
                id: "req-deny-1".to_string(),
                method: "confirm".to_string(),
                payload: json!({ "title": "Do risky thing?", "timeout": 5000 }),
                timeout_ms: Some(5000),
                extension_id: None,
            };
            mgr.request_ui(request).await
        });

        let ui_event = recv_ui_request(&out_rx, "confirm_denied").await;
        assert_eq!(ui_event["id"], "req-deny-1");

        // Respond with confirmed = false.
        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"cmd-2","type":"extension_ui_response","requestId":"req-deny-1","value":false}"#,
            "confirm_denied_response",
        )
        .await;
        assert_ok(&resp, "extension_ui_response");

        let ui_result = ui_task.await;
        let response = ui_result.expect("request_ui should succeed");
        let response = response.expect("should have a response");
        assert!(!response.cancelled);
        assert_eq!(response.value, Some(json!(false)));

        drop(in_tx);
        let _ = server.await;
    });
}

// ---------------------------------------------------------------------------
// Tests: Extension UI roundtrip — select
// ---------------------------------------------------------------------------

#[test]
fn rpc_extension_ui_select_roundtrip() {
    let _harness = TestHarness::new("rpc_extension_ui_select_roundtrip");
    let cassette_dir = cassette_root();
    let runtime = asupersync::runtime::RuntimeBuilder::current_thread()
        .build()
        .expect("build test runtime");
    let handle = runtime.handle();

    runtime.block_on(async move {
        let (agent_session, manager) =
            build_agent_session_with_extensions(Session::in_memory(), &cassette_dir);
        let options = build_options(
            &handle,
            PathBuf::from("/tmp/auth_ui_select.json"),
            vec![],
            vec![],
        );
        let (in_tx, in_rx) = asupersync::channel::mpsc::channel::<String>(16);
        let (out_tx, out_rx) = std::sync::mpsc::channel::<String>();
        let out_rx = Arc::new(Mutex::new(out_rx));

        let server = handle.spawn(async move { run(agent_session, options, in_rx, out_tx).await });
        asupersync::time::sleep(asupersync::time::wall_now(), Duration::from_millis(50)).await;

        let mgr = manager.clone();
        let ui_task = handle.spawn(async move {
            let request = ExtensionUiRequest {
                id: "req-select-1".to_string(),
                method: "select".to_string(),
                payload: json!({
                    "title": "Pick a model",
                    "options": ["claude-sonnet", "gpt-4o", "gemini-pro"],
                    "timeout": 5000
                }),
                timeout_ms: Some(5000),
                extension_id: Some("model-picker-ext".to_string()),
            };
            mgr.request_ui(request).await
        });

        let ui_event = recv_ui_request(&out_rx, "select").await;
        assert_eq!(ui_event["id"], "req-select-1");
        assert_eq!(ui_event["method"], "select");
        assert_eq!(ui_event["title"], "Pick a model");
        assert!(ui_event["options"].is_array());

        // Select the second option.
        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"cmd-3","type":"extension_ui_response","requestId":"req-select-1","value":"gpt-4o"}"#,
            "select_response",
        )
        .await;
        assert_ok(&resp, "extension_ui_response");

        let ui_result = ui_task.await;
        let response = ui_result.expect("request_ui should succeed");
        let response = response.expect("should have a response");
        assert!(!response.cancelled);
        assert_eq!(response.value, Some(json!("gpt-4o")));

        drop(in_tx);
        let _ = server.await;
    });
}

// ---------------------------------------------------------------------------
// Tests: Extension UI roundtrip — input
// ---------------------------------------------------------------------------

#[test]
fn rpc_extension_ui_input_roundtrip() {
    let _harness = TestHarness::new("rpc_extension_ui_input_roundtrip");
    let cassette_dir = cassette_root();
    let runtime = asupersync::runtime::RuntimeBuilder::current_thread()
        .build()
        .expect("build test runtime");
    let handle = runtime.handle();

    runtime.block_on(async move {
        let (agent_session, manager) =
            build_agent_session_with_extensions(Session::in_memory(), &cassette_dir);
        let options = build_options(
            &handle,
            PathBuf::from("/tmp/auth_ui_input.json"),
            vec![],
            vec![],
        );
        let (in_tx, in_rx) = asupersync::channel::mpsc::channel::<String>(16);
        let (out_tx, out_rx) = std::sync::mpsc::channel::<String>();
        let out_rx = Arc::new(Mutex::new(out_rx));

        let server = handle.spawn(async move { run(agent_session, options, in_rx, out_tx).await });
        asupersync::time::sleep(asupersync::time::wall_now(), Duration::from_millis(50)).await;

        let mgr = manager.clone();
        let ui_task = handle.spawn(async move {
            let request = ExtensionUiRequest {
                id: "req-input-1".to_string(),
                method: "input".to_string(),
                payload: json!({
                    "title": "Enter API key",
                    "message": "Paste your key below",
                    "timeout": 5000
                }),
                timeout_ms: Some(5000),
                extension_id: Some("api-key-ext".to_string()),
            };
            mgr.request_ui(request).await
        });

        let ui_event = recv_ui_request(&out_rx, "input").await;
        assert_eq!(ui_event["id"], "req-input-1");
        assert_eq!(ui_event["method"], "input");

        // Respond with typed text.
        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"cmd-4","type":"extension_ui_response","requestId":"req-input-1","value":"sk-test-12345"}"#,
            "input_response",
        )
        .await;
        assert_ok(&resp, "extension_ui_response");

        let ui_result = ui_task.await;
        let response = ui_result.expect("request_ui should succeed");
        let response = response.expect("should have a response");
        assert!(!response.cancelled);
        assert_eq!(response.value, Some(json!("sk-test-12345")));

        drop(in_tx);
        let _ = server.await;
    });
}

// ---------------------------------------------------------------------------
// Tests: Extension UI roundtrip — editor
// ---------------------------------------------------------------------------

#[test]
fn rpc_extension_ui_editor_roundtrip() {
    let _harness = TestHarness::new("rpc_extension_ui_editor_roundtrip");
    let cassette_dir = cassette_root();
    let runtime = asupersync::runtime::RuntimeBuilder::current_thread()
        .build()
        .expect("build test runtime");
    let handle = runtime.handle();

    runtime.block_on(async move {
        let (agent_session, manager) =
            build_agent_session_with_extensions(Session::in_memory(), &cassette_dir);
        let options = build_options(
            &handle,
            PathBuf::from("/tmp/auth_ui_editor.json"),
            vec![],
            vec![],
        );
        let (in_tx, in_rx) = asupersync::channel::mpsc::channel::<String>(16);
        let (out_tx, out_rx) = std::sync::mpsc::channel::<String>();
        let out_rx = Arc::new(Mutex::new(out_rx));

        let server = handle.spawn(async move { run(agent_session, options, in_rx, out_tx).await });
        asupersync::time::sleep(asupersync::time::wall_now(), Duration::from_millis(50)).await;

        let mgr = manager.clone();
        let ui_task = handle.spawn(async move {
            let request = ExtensionUiRequest {
                id: "req-editor-1".to_string(),
                method: "editor".to_string(),
                payload: json!({
                    "title": "Edit config",
                    "message": "Modify the YAML below",
                    "timeout": 5000
                }),
                timeout_ms: Some(5000),
                extension_id: None,
            };
            mgr.request_ui(request).await
        });

        let ui_event = recv_ui_request(&out_rx, "editor").await;
        assert_eq!(ui_event["id"], "req-editor-1");
        assert_eq!(ui_event["method"], "editor");

        // Respond with edited text.
        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"cmd-5","type":"extension_ui_response","requestId":"req-editor-1","value":"key: new_value"}"#,
            "editor_response",
        )
        .await;
        assert_ok(&resp, "extension_ui_response");

        let ui_result = ui_task.await;
        let response = ui_result.expect("request_ui should succeed");
        let response = response.expect("should have a response");
        assert!(!response.cancelled);
        assert_eq!(response.value, Some(json!("key: new_value")));

        drop(in_tx);
        let _ = server.await;
    });
}

// ---------------------------------------------------------------------------
// Tests: Extension UI — cancellation
// ---------------------------------------------------------------------------

#[test]
fn rpc_extension_ui_cancel_response() {
    let _harness = TestHarness::new("rpc_extension_ui_cancel_response");
    let cassette_dir = cassette_root();
    let runtime = asupersync::runtime::RuntimeBuilder::current_thread()
        .build()
        .expect("build test runtime");
    let handle = runtime.handle();

    runtime.block_on(async move {
        let (agent_session, manager) =
            build_agent_session_with_extensions(Session::in_memory(), &cassette_dir);
        let options = build_options(
            &handle,
            PathBuf::from("/tmp/auth_ui_cancel.json"),
            vec![],
            vec![],
        );
        let (in_tx, in_rx) = asupersync::channel::mpsc::channel::<String>(16);
        let (out_tx, out_rx) = std::sync::mpsc::channel::<String>();
        let out_rx = Arc::new(Mutex::new(out_rx));

        let server = handle.spawn(async move { run(agent_session, options, in_rx, out_tx).await });
        asupersync::time::sleep(asupersync::time::wall_now(), Duration::from_millis(50)).await;

        let mgr = manager.clone();
        let ui_task = handle.spawn(async move {
            let request = ExtensionUiRequest {
                id: "req-cancel-1".to_string(),
                method: "confirm".to_string(),
                payload: json!({ "title": "Proceed?", "timeout": 5000 }),
                timeout_ms: Some(5000),
                extension_id: None,
            };
            mgr.request_ui(request).await
        });

        let ui_event = recv_ui_request(&out_rx, "cancel").await;
        assert_eq!(ui_event["id"], "req-cancel-1");

        // Respond with cancelled: true.
        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"cmd-6","type":"extension_ui_response","requestId":"req-cancel-1","cancelled":true}"#,
            "cancel_response",
        )
        .await;
        assert_ok(&resp, "extension_ui_response");

        let ui_result = ui_task.await;
        let response = ui_result.expect("request_ui should succeed");
        let response = response.expect("should have a response");
        assert!(response.cancelled);

        drop(in_tx);
        let _ = server.await;
    });
}

// ---------------------------------------------------------------------------
// Tests: Extension UI — no extensions configured (noop fallback)
// ---------------------------------------------------------------------------

#[test]
fn rpc_extension_ui_response_without_extensions() {
    let _harness = TestHarness::new("rpc_extension_ui_response_without_extensions");
    let cassette_dir = cassette_root();
    let runtime = asupersync::runtime::RuntimeBuilder::current_thread()
        .build()
        .expect("build test runtime");
    let handle = runtime.handle();

    runtime.block_on(async move {
        // No extensions — build_agent_session leaves extensions = None.
        let agent_session = build_agent_session(Session::in_memory(), &cassette_dir);
        let options = build_options(
            &handle,
            PathBuf::from("/tmp/auth_ui_noext.json"),
            vec![],
            vec![],
        );
        let (in_tx, in_rx) = asupersync::channel::mpsc::channel::<String>(16);
        let (out_tx, out_rx) = std::sync::mpsc::channel::<String>();
        let out_rx = Arc::new(Mutex::new(out_rx));

        let server = handle.spawn(async move { run(agent_session, options, in_rx, out_tx).await });

        // Sending extension_ui_response when no extensions are configured
        // should return a success noop (no data).
        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"cmd-7","type":"extension_ui_response","requestId":"req-x","confirmed":true}"#,
            "no_extensions",
        )
        .await;
        assert_ok(&resp, "extension_ui_response");
        assert!(resp.get("data").is_none() || resp["data"].is_null());

        drop(in_tx);
        let _ = server.await;
    });
}

// ---------------------------------------------------------------------------
// Tests: Extension UI — mismatched request ID
// ---------------------------------------------------------------------------

#[test]
fn rpc_extension_ui_mismatched_request_id() {
    let _harness = TestHarness::new("rpc_extension_ui_mismatched_request_id");
    let cassette_dir = cassette_root();
    let runtime = asupersync::runtime::RuntimeBuilder::current_thread()
        .build()
        .expect("build test runtime");
    let handle = runtime.handle();

    runtime.block_on(async move {
        let (agent_session, manager) =
            build_agent_session_with_extensions(Session::in_memory(), &cassette_dir);
        let options = build_options(
            &handle,
            PathBuf::from("/tmp/auth_ui_mismatch.json"),
            vec![],
            vec![],
        );
        let (in_tx, in_rx) = asupersync::channel::mpsc::channel::<String>(16);
        let (out_tx, out_rx) = std::sync::mpsc::channel::<String>();
        let out_rx = Arc::new(Mutex::new(out_rx));

        let server = handle.spawn(async move { run(agent_session, options, in_rx, out_tx).await });
        asupersync::time::sleep(asupersync::time::wall_now(), Duration::from_millis(50)).await;

        let mgr = manager.clone();
        let _ui_task = handle.spawn(async move {
            let request = ExtensionUiRequest {
                id: "req-real-1".to_string(),
                method: "confirm".to_string(),
                payload: json!({ "title": "Do it?", "timeout": 5000 }),
                timeout_ms: Some(5000),
                extension_id: None,
            };
            mgr.request_ui(request).await
        });

        let ui_event = recv_ui_request(&out_rx, "mismatch").await;
        assert_eq!(ui_event["id"], "req-real-1");

        // Send response with WRONG request ID.
        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"cmd-8","type":"extension_ui_response","requestId":"req-WRONG","confirmed":true}"#,
            "wrong_id_response",
        )
        .await;
        assert_err(&resp, "extension_ui_response");
        let error_msg = resp["error"].as_str().unwrap_or("");
        assert!(
            error_msg.contains("Unexpected requestId"),
            "error should mention unexpected requestId: {error_msg}"
        );

        // Now send the correct one to clean up.
        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"cmd-9","type":"extension_ui_response","requestId":"req-real-1","confirmed":true}"#,
            "correct_id_response",
        )
        .await;
        assert_ok(&resp, "extension_ui_response");

        drop(in_tx);
        let _ = server.await;
    });
}

// ---------------------------------------------------------------------------
// Tests: Extension UI — missing requestId
// ---------------------------------------------------------------------------

#[test]
fn rpc_extension_ui_missing_request_id() {
    let _harness = TestHarness::new("rpc_extension_ui_missing_request_id");
    let cassette_dir = cassette_root();
    let runtime = asupersync::runtime::RuntimeBuilder::current_thread()
        .build()
        .expect("build test runtime");
    let handle = runtime.handle();

    runtime.block_on(async move {
        let (agent_session, _manager) =
            build_agent_session_with_extensions(Session::in_memory(), &cassette_dir);
        let options = build_options(
            &handle,
            PathBuf::from("/tmp/auth_ui_noid.json"),
            vec![],
            vec![],
        );
        let (in_tx, in_rx) = asupersync::channel::mpsc::channel::<String>(16);
        let (out_tx, out_rx) = std::sync::mpsc::channel::<String>();
        let out_rx = Arc::new(Mutex::new(out_rx));

        let server = handle.spawn(async move { run(agent_session, options, in_rx, out_tx).await });

        // Send extension_ui_response without requestId OR id — the parser
        // falls back to "id" as an alias, so we must omit both to trigger
        // the "Missing requestId" error.
        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"type":"extension_ui_response","confirmed":true}"#,
            "missing_id",
        )
        .await;
        assert_err(&resp, "extension_ui_response");
        let error_msg = resp["error"].as_str().unwrap_or("");
        assert!(
            error_msg.contains("Missing requestId"),
            "error should mention missing requestId: {error_msg}"
        );

        drop(in_tx);
        let _ = server.await;
    });
}

// ---------------------------------------------------------------------------
// Tests: Extension UI — legacy id alias accepted
// ---------------------------------------------------------------------------

#[test]
fn rpc_extension_ui_id_alias_roundtrip() {
    let _harness = TestHarness::new("rpc_extension_ui_id_alias_roundtrip");
    let cassette_dir = cassette_root();
    let runtime = asupersync::runtime::RuntimeBuilder::current_thread()
        .build()
        .expect("build test runtime");
    let handle = runtime.handle();

    runtime.block_on(async move {
        let (agent_session, manager) =
            build_agent_session_with_extensions(Session::in_memory(), &cassette_dir);
        let options = build_options(
            &handle,
            PathBuf::from("/tmp/auth_ui_id_alias.json"),
            vec![],
            vec![],
        );
        let (in_tx, in_rx) = asupersync::channel::mpsc::channel::<String>(16);
        let (out_tx, out_rx) = std::sync::mpsc::channel::<String>();
        let out_rx = Arc::new(Mutex::new(out_rx));

        let server = handle.spawn(async move { run(agent_session, options, in_rx, out_tx).await });
        asupersync::time::sleep(asupersync::time::wall_now(), Duration::from_millis(50)).await;

        let mgr = manager.clone();
        let ui_task = handle.spawn(async move {
            let request = ExtensionUiRequest {
                id: "req-legacy-1".to_string(),
                method: "confirm".to_string(),
                payload: json!({ "title": "Legacy id alias?", "timeout": 5000 }),
                timeout_ms: Some(5000),
                extension_id: None,
            };
            mgr.request_ui(request).await
        });

        let ui_event = recv_ui_request(&out_rx, "id_alias").await;
        assert_eq!(ui_event["id"], "req-legacy-1");
        assert_eq!(ui_event["method"], "confirm");

        // Upstream accepts top-level "id" as a requestId alias for
        // extension_ui_response.
        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"req-legacy-1","type":"extension_ui_response","confirmed":true}"#,
            "id_alias_response",
        )
        .await;
        assert_ok(&resp, "extension_ui_response");
        assert_eq!(resp["id"], "req-legacy-1");
        assert_eq!(resp["data"]["resolved"], true);

        let ui_result = ui_task.await;
        let response = ui_result.expect("request_ui should succeed");
        let response = response.expect("should have a response");
        assert_eq!(response.value, Some(json!(true)));
        assert!(!response.cancelled);

        drop(in_tx);
        let _ = server.await;
    });
}

// ---------------------------------------------------------------------------
// Tests: Extension UI — sequential ordering (one at a time)
// ---------------------------------------------------------------------------

#[test]
fn rpc_extension_ui_sequential_ordering() {
    let _harness = TestHarness::new("rpc_extension_ui_sequential_ordering");
    let cassette_dir = cassette_root();
    let runtime = asupersync::runtime::RuntimeBuilder::current_thread()
        .build()
        .expect("build test runtime");
    let handle = runtime.handle();

    runtime.block_on(async move {
        let (agent_session, manager) =
            build_agent_session_with_extensions(Session::in_memory(), &cassette_dir);
        let options = build_options(
            &handle,
            PathBuf::from("/tmp/auth_ui_seq.json"),
            vec![],
            vec![],
        );
        let (in_tx, in_rx) = asupersync::channel::mpsc::channel::<String>(16);
        let (out_tx, out_rx) = std::sync::mpsc::channel::<String>();
        let out_rx = Arc::new(Mutex::new(out_rx));

        let server = handle.spawn(async move { run(agent_session, options, in_rx, out_tx).await });
        asupersync::time::sleep(asupersync::time::wall_now(), Duration::from_millis(50)).await;

        // Fire two requests concurrently.
        let mgr1 = manager.clone();
        let ui_task_1 = handle.spawn(async move {
            let request = ExtensionUiRequest {
                id: "req-seq-1".to_string(),
                method: "confirm".to_string(),
                payload: json!({ "title": "First?", "timeout": 5000 }),
                timeout_ms: Some(5000),
                extension_id: Some("ext-a".to_string()),
            };
            mgr1.request_ui(request).await
        });

        // Wait for the first to be emitted before sending the second.
        let first_event = recv_ui_request(&out_rx, "seq_first").await;
        assert_eq!(first_event["id"], "req-seq-1");

        let mgr2 = manager.clone();
        let ui_task_2 = handle.spawn(async move {
            let request = ExtensionUiRequest {
                id: "req-seq-2".to_string(),
                method: "input".to_string(),
                payload: json!({ "title": "Second?", "timeout": 5000 }),
                timeout_ms: Some(5000),
                extension_id: Some("ext-b".to_string()),
            };
            mgr2.request_ui(request).await
        });

        // Give the second request time to enter the queue.
        asupersync::time::sleep(asupersync::time::wall_now(), Duration::from_millis(100)).await;

        // Respond to the first — this should dequeue the second.
        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"cmd-11","type":"extension_ui_response","requestId":"req-seq-1","confirmed":true}"#,
            "seq_first_response",
        )
        .await;
        assert_ok(&resp, "extension_ui_response");

        let r1 = ui_task_1
            .await
            .expect("first request_ui")
            .expect("has response");
        assert_eq!(r1.value, Some(json!(true)));

        // Now the second request should be emitted.
        let second_event = recv_ui_request(&out_rx, "seq_second").await;
        assert_eq!(second_event["id"], "req-seq-2");
        assert_eq!(second_event["method"], "input");

        // Respond to the second.
        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"cmd-12","type":"extension_ui_response","requestId":"req-seq-2","value":"hello"}"#,
            "seq_second_response",
        )
        .await;
        assert_ok(&resp, "extension_ui_response");

        let r2 = ui_task_2
            .await
            .expect("second request_ui")
            .expect("has response");
        assert_eq!(r2.value, Some(json!("hello")));

        drop(in_tx);
        let _ = server.await;
    });
}

// ---------------------------------------------------------------------------
// Tests: Extension UI — no active request error
// ---------------------------------------------------------------------------

#[test]
fn rpc_extension_ui_no_active_request() {
    let _harness = TestHarness::new("rpc_extension_ui_no_active_request");
    let cassette_dir = cassette_root();
    let runtime = asupersync::runtime::RuntimeBuilder::current_thread()
        .build()
        .expect("build test runtime");
    let handle = runtime.handle();

    runtime.block_on(async move {
        let (agent_session, _manager) =
            build_agent_session_with_extensions(Session::in_memory(), &cassette_dir);
        let options = build_options(
            &handle,
            PathBuf::from("/tmp/auth_ui_noactive.json"),
            vec![],
            vec![],
        );
        let (in_tx, in_rx) = asupersync::channel::mpsc::channel::<String>(16);
        let (out_tx, out_rx) = std::sync::mpsc::channel::<String>();
        let out_rx = Arc::new(Mutex::new(out_rx));

        let server = handle.spawn(async move { run(agent_session, options, in_rx, out_tx).await });

        // Send a response when no UI request is active.
        let resp = send_recv(
            &in_tx,
            &out_rx,
            r#"{"id":"cmd-13","type":"extension_ui_response","requestId":"req-ghost","confirmed":true}"#,
            "no_active",
        )
        .await;
        assert_err(&resp, "extension_ui_response");
        let error_msg = resp["error"].as_str().unwrap_or("");
        assert!(
            error_msg.contains("No active extension UI request"),
            "error should mention no active request: {error_msg}"
        );

        drop(in_tx);
        let _ = server.await;
    });
}

// ---------------------------------------------------------------------------
// Tests: Extension UI — fire-and-forget (notify) emitted but not queued
// ---------------------------------------------------------------------------

#[test]
fn rpc_extension_ui_notify_fire_and_forget() {
    let _harness = TestHarness::new("rpc_extension_ui_notify_fire_and_forget");
    let cassette_dir = cassette_root();
    let runtime = asupersync::runtime::RuntimeBuilder::current_thread()
        .build()
        .expect("build test runtime");
    let handle = runtime.handle();

    runtime.block_on(async move {
        let (agent_session, manager) =
            build_agent_session_with_extensions(Session::in_memory(), &cassette_dir);
        let options = build_options(
            &handle,
            PathBuf::from("/tmp/auth_ui_notify.json"),
            vec![],
            vec![],
        );
        let (in_tx, in_rx) = asupersync::channel::mpsc::channel::<String>(16);
        let (out_tx, out_rx) = std::sync::mpsc::channel::<String>();
        let out_rx = Arc::new(Mutex::new(out_rx));

        let server = handle.spawn(async move { run(agent_session, options, in_rx, out_tx).await });
        asupersync::time::sleep(asupersync::time::wall_now(), Duration::from_millis(50)).await;

        // Send a "notify" request — fire-and-forget, no response expected.
        let mgr = manager.clone();
        let ui_task = handle.spawn(async move {
            let request = ExtensionUiRequest {
                id: "req-notify-1".to_string(),
                method: "notify".to_string(),
                payload: json!({
                    "title": "Heads up!",
                    "message": "Something happened"
                }),
                timeout_ms: None,
                extension_id: Some("notifier-ext".to_string()),
            };
            mgr.request_ui(request).await
        });

        // The event should still be emitted to the RPC output.
        let ui_event = recv_ui_request(&out_rx, "notify").await;
        assert_eq!(ui_event["type"], "extension_ui_request");
        assert_eq!(ui_event["id"], "req-notify-1");
        assert_eq!(ui_event["method"], "notify");

        // request_ui should return Ok(None) for fire-and-forget.
        let ui_result = ui_task.await;
        let response = ui_result.expect("request_ui should succeed");
        assert!(response.is_none(), "notify should not return a response");

        drop(in_tx);
        let _ = server.await;
    });
}
