//! Native adapter contract tests — request/response/tool schema validation.
//!
//! These tests verify the behavioral contract between provider adapters and the
//! Pi internal event model using VCR cassettes for deterministic, offline execution.
//! They complement the backward-lock tests (request shape) and native-verify tests
//! (scenario correctness) by focusing on:
//!
//! 1. **Response event decoding**: VCR cassettes produce the correct `StreamEvent` sequence.
//! 2. **Event contract invariants**: Every provider's stream follows `Start` → deltas → `Done`.
//! 3. **Tool-call round-trip**: Tool schemas sent → tool calls returned with valid JSON args.
//! 4. **Error event contract**: Error cassettes produce `Error` events with correct stop reasons.
//!
//! bd-3uqg.8.2

mod common;

use futures::StreamExt;
use pi::http::client::Client;
use pi::model::{Message, StopReason, StreamEvent, UserContent, UserMessage};
use pi::provider::{Context, Provider, StreamOptions, ToolDef};
use pi::vcr::{VcrMode, VcrRecorder};
use serde_json::{Value, json};
use std::env;
use std::path::PathBuf;
use std::sync::Arc;

// ═══════════════════════════════════════════════════════════════════════
// Shared helpers
// ═══════════════════════════════════════════════════════════════════════

fn cassette_root() -> PathBuf {
    env::var("VCR_CASSETTE_DIR").map_or_else(
        |_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/vcr"),
        PathBuf::from,
    )
}

fn user_text(text: &str) -> Message {
    Message::User(UserMessage {
        content: UserContent::Text(text.to_string()),
        timestamp: 0,
    })
}

fn simple_context() -> Context<'static> {
    Context::owned(
        Some("You are helpful.".to_string()),
        vec![user_text("Say hello in one sentence.")],
        Vec::new(),
    )
}

fn tool_context() -> Context<'static> {
    // Must match the tool definition used when recording VCR cassettes.
    Context::owned(
        Some("You are a coding assistant.".to_string()),
        vec![user_text("Echo the text: verification test")],
        vec![ToolDef {
            name: "echo".to_string(),
            description: "Echo text back".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "text": { "type": "string", "description": "Text to echo" }
                },
                "required": ["text"]
            }),
        }],
    )
}

fn context_with_tools(tools: Vec<ToolDef>) -> Context<'static> {
    Context::owned(
        Some("You are a coding assistant.".to_string()),
        vec![user_text("Validate tool schema wiring.")],
        tools,
    )
}

fn default_options() -> StreamOptions {
    StreamOptions {
        api_key: Some("vcr-playback".to_string()),
        max_tokens: Some(256),
        temperature: Some(0.0),
        ..Default::default()
    }
}

struct EventSummary {
    timeline: Vec<String>,
    has_start: bool,
    has_done: bool,
    has_error: bool,
    text: String,
    tool_call_count: usize,
    tool_call_names: Vec<String>,
    tool_call_args: Vec<Value>,
    stop_reason: Option<StopReason>,
    stream_error: Option<String>,
}

async fn collect_and_summarize(
    provider: &dyn Provider,
    context: &Context<'_>,
    options: &StreamOptions,
) -> EventSummary {
    let stream_result = provider.stream(context, options).await;
    let mut summary = EventSummary {
        timeline: Vec::new(),
        has_start: false,
        has_done: false,
        has_error: false,
        text: String::new(),
        tool_call_count: 0,
        tool_call_names: Vec::new(),
        tool_call_args: Vec::new(),
        stop_reason: None,
        stream_error: None,
    };

    match stream_result {
        Ok(mut stream) => {
            while let Some(item) = stream.next().await {
                match item {
                    Ok(event) => match &event {
                        StreamEvent::Start { .. } => {
                            summary.has_start = true;
                            summary.timeline.push("start".into());
                        }
                        StreamEvent::TextStart { .. } => {
                            summary.timeline.push("text_start".into());
                        }
                        StreamEvent::TextDelta { delta, .. } => {
                            summary.text.push_str(delta);
                            summary.timeline.push("text_delta".into());
                        }
                        StreamEvent::TextEnd { content, .. } => {
                            summary.text.clone_from(content);
                            summary.timeline.push("text_end".into());
                        }
                        StreamEvent::ThinkingStart { .. } => {
                            summary.timeline.push("thinking_start".into());
                        }
                        StreamEvent::ThinkingDelta { .. } => {
                            summary.timeline.push("thinking_delta".into());
                        }
                        StreamEvent::ThinkingEnd { .. } => {
                            summary.timeline.push("thinking_end".into());
                        }
                        StreamEvent::ToolCallStart { .. } => {
                            summary.timeline.push("tool_call_start".into());
                        }
                        StreamEvent::ToolCallDelta { .. } => {
                            summary.timeline.push("tool_call_delta".into());
                        }
                        StreamEvent::ToolCallEnd { tool_call, .. } => {
                            summary.tool_call_count += 1;
                            summary.tool_call_names.push(tool_call.name.clone());
                            summary.tool_call_args.push(tool_call.arguments.clone());
                            summary.timeline.push("tool_call_end".into());
                        }
                        StreamEvent::Done { reason, .. } => {
                            summary.has_done = true;
                            summary.stop_reason = Some(*reason);
                            summary.timeline.push("done".into());
                        }
                        StreamEvent::Error { reason, .. } => {
                            summary.has_error = true;
                            summary.stop_reason = Some(*reason);
                            summary.timeline.push("error".into());
                        }
                    },
                    Err(err) => {
                        summary.stream_error = Some(err.to_string());
                        break;
                    }
                }
            }
        }
        Err(err) => {
            summary.stream_error = Some(err.to_string());
        }
    }

    summary
}

/// Load a VCR recorder from the standard cassette directory.
fn load_vcr(cassette_name: &str) -> Option<VcrRecorder> {
    let path = cassette_root().join(format!("{cassette_name}.json"));
    if !path.exists() {
        return None;
    }
    Some(VcrRecorder::new_with(
        cassette_name,
        VcrMode::Playback,
        cassette_root(),
    ))
}

// ═══════════════════════════════════════════════════════════════════════
// Contract: simple_text produces Start → TextDelta+ → Done(Stop)
// ═══════════════════════════════════════════════════════════════════════

macro_rules! simple_text_contract {
    ($test_name:ident, $provider_ctor:expr, $cassette:expr) => {
        #[test]
        fn $test_name() {
            let cassette_name = $cassette;
            let Some(vcr) = load_vcr(cassette_name) else {
                eprintln!("SKIP: cassette {cassette_name} not found");
                return;
            };
            let provider = $provider_ctor(vcr);

            asupersync::runtime::RuntimeBuilder::current_thread()
                .build()
                .expect("runtime")
                .block_on(async {
                    let summary =
                        collect_and_summarize(&*provider, &simple_context(), &default_options())
                            .await;

                    // Contract: must have Start event
                    assert!(summary.has_start, "{cassette_name}: missing Start event");

                    // Contract: must have Done event with Stop reason
                    assert!(summary.has_done, "{cassette_name}: missing Done event");
                    assert_eq!(
                        summary.stop_reason,
                        Some(StopReason::Stop),
                        "{cassette_name}: stop reason should be Stop for simple text"
                    );

                    // Contract: must produce non-empty text
                    assert!(
                        !summary.text.is_empty(),
                        "{cassette_name}: no text produced"
                    );

                    // Contract: no tool calls in simple text scenario
                    assert_eq!(
                        summary.tool_call_count, 0,
                        "{cassette_name}: unexpected tool calls in simple text"
                    );

                    // Contract: timeline must follow Start → ... → Done ordering
                    assert_eq!(
                        summary.timeline.first().map(String::as_str),
                        Some("start"),
                        "{cassette_name}: first event must be start"
                    );
                    assert_eq!(
                        summary.timeline.last().map(String::as_str),
                        Some("done"),
                        "{cassette_name}: last event must be done"
                    );

                    // Contract: no stream errors
                    assert!(
                        summary.stream_error.is_none(),
                        "{cassette_name}: stream error: {:?}",
                        summary.stream_error
                    );
                });
        }
    };
}

// ═══════════════════════════════════════════════════════════════════════
// Contract: tool_call produces Start → ToolCallDelta+ → ToolCallEnd → Done(ToolUse)
// ═══════════════════════════════════════════════════════════════════════

macro_rules! tool_call_contract {
    ($test_name:ident, $provider_ctor:expr, $cassette:expr) => {
        #[test]
        fn $test_name() {
            let cassette_name = $cassette;
            let Some(vcr) = load_vcr(cassette_name) else {
                eprintln!("SKIP: cassette {cassette_name} not found");
                return;
            };
            let provider = $provider_ctor(vcr);

            asupersync::runtime::RuntimeBuilder::current_thread()
                .build()
                .expect("runtime")
                .block_on(async {
                    let summary =
                        collect_and_summarize(&*provider, &tool_context(), &default_options())
                            .await;

                    // Contract: must have Start event
                    assert!(summary.has_start, "{cassette_name}: missing Start event");

                    // Contract: must have Done event with ToolUse reason
                    assert!(summary.has_done, "{cassette_name}: missing Done event");
                    assert_eq!(
                        summary.stop_reason,
                        Some(StopReason::ToolUse),
                        "{cassette_name}: stop reason should be ToolUse for tool call"
                    );

                    // Contract: at least one tool call
                    assert!(
                        summary.tool_call_count >= 1,
                        "{cassette_name}: expected at least 1 tool call, got {}",
                        summary.tool_call_count
                    );

                    // Contract: tool call name must be one of the tools we provided
                    for name in &summary.tool_call_names {
                        assert_eq!(
                            name, "echo",
                            "{cassette_name}: unexpected tool name: {name}"
                        );
                    }

                    // Contract: tool call arguments must be a JSON object
                    for args in &summary.tool_call_args {
                        assert!(
                            args.is_object(),
                            "{cassette_name}: tool call args must be a JSON object: {args}"
                        );
                    }

                    // Contract: timeline must have tool_call_end
                    assert!(
                        summary.timeline.contains(&"tool_call_end".to_string()),
                        "{cassette_name}: timeline missing tool_call_end"
                    );

                    // Contract: no stream errors
                    assert!(
                        summary.stream_error.is_none(),
                        "{cassette_name}: stream error: {:?}",
                        summary.stream_error
                    );
                });
        }
    };
}

// ═══════════════════════════════════════════════════════════════════════
// Contract: error_auth_401 produces Error or stream error
// ═══════════════════════════════════════════════════════════════════════

macro_rules! error_auth_contract {
    ($test_name:ident, $provider_ctor:expr, $cassette:expr) => {
        #[test]
        fn $test_name() {
            let cassette_name = $cassette;
            let Some(vcr) = load_vcr(cassette_name) else {
                eprintln!("SKIP: cassette {cassette_name} not found");
                return;
            };
            let provider = $provider_ctor(vcr);

            asupersync::runtime::RuntimeBuilder::current_thread()
                .build()
                .expect("runtime")
                .block_on(async {
                    let summary =
                        collect_and_summarize(&*provider, &simple_context(), &default_options())
                            .await;

                    // Contract: must signal an error (either via Error event or stream_error)
                    let has_any_error = summary.has_error || summary.stream_error.is_some();
                    assert!(
                        has_any_error,
                        "{cassette_name}: auth 401 should produce an error, got timeline: {:?}",
                        summary.timeline
                    );
                });
        }
    };
}

// ═══════════════════════════════════════════════════════════════════════
// Provider constructors that accept a VCR recorder
// ═══════════════════════════════════════════════════════════════════════

use pi::providers::anthropic::AnthropicProvider;
use pi::providers::cohere::CohereProvider;
use pi::providers::gemini::GeminiProvider;
use pi::providers::openai::OpenAIProvider;

fn anthropic_with_vcr(vcr: VcrRecorder) -> Arc<dyn Provider> {
    let client = Client::new().with_vcr(vcr);
    Arc::new(AnthropicProvider::new("claude-sonnet-4-5").with_client(client))
}

fn openai_with_vcr(vcr: VcrRecorder) -> Arc<dyn Provider> {
    let client = Client::new().with_vcr(vcr);
    Arc::new(OpenAIProvider::new("gpt-4o").with_client(client))
}

fn gemini_with_vcr(vcr: VcrRecorder) -> Arc<dyn Provider> {
    let client = Client::new().with_vcr(vcr);
    // Model must match the cassette URL (contains model ID in path).
    Arc::new(GeminiProvider::new("gemini-1.5-flash").with_client(client))
}

fn cohere_with_vcr(vcr: VcrRecorder) -> Arc<dyn Provider> {
    let client = Client::new().with_vcr(vcr);
    Arc::new(CohereProvider::new("command-r-plus").with_client(client))
}

// ═══════════════════════════════════════════════════════════════════════
// Anthropic contract tests
// ═══════════════════════════════════════════════════════════════════════

simple_text_contract!(
    contract_anthropic_simple_text,
    anthropic_with_vcr,
    "verify_anthropic_simple_text"
);

tool_call_contract!(
    contract_anthropic_tool_call,
    anthropic_with_vcr,
    "verify_anthropic_tool_call_single"
);

error_auth_contract!(
    contract_anthropic_error_auth,
    anthropic_with_vcr,
    "verify_anthropic_error_auth_401"
);

// ═══════════════════════════════════════════════════════════════════════
// OpenAI contract tests
// ═══════════════════════════════════════════════════════════════════════

simple_text_contract!(
    contract_openai_simple_text,
    openai_with_vcr,
    "verify_openai_simple_text"
);

tool_call_contract!(
    contract_openai_tool_call,
    openai_with_vcr,
    "verify_openai_tool_call_single"
);

error_auth_contract!(
    contract_openai_error_auth,
    openai_with_vcr,
    "verify_openai_error_auth_401"
);

// ═══════════════════════════════════════════════════════════════════════
// Gemini contract tests
// ═══════════════════════════════════════════════════════════════════════

simple_text_contract!(
    contract_gemini_simple_text,
    gemini_with_vcr,
    "verify_gemini_simple_text"
);

tool_call_contract!(
    contract_gemini_tool_call,
    gemini_with_vcr,
    "verify_gemini_tool_call_single"
);

error_auth_contract!(
    contract_gemini_error_auth,
    gemini_with_vcr,
    "verify_gemini_error_auth_401"
);

// ═══════════════════════════════════════════════════════════════════════
// Cohere contract tests
// ═══════════════════════════════════════════════════════════════════════

simple_text_contract!(
    contract_cohere_simple_text,
    cohere_with_vcr,
    "verify_cohere_simple_text"
);

tool_call_contract!(
    contract_cohere_tool_call,
    cohere_with_vcr,
    "verify_cohere_tool_call_single"
);

error_auth_contract!(
    contract_cohere_error_auth,
    cohere_with_vcr,
    "verify_cohere_error_auth_401"
);

// ═══════════════════════════════════════════════════════════════════════
// Cross-provider event ordering invariant
// ═══════════════════════════════════════════════════════════════════════

type ProviderEntry = (
    &'static str,
    &'static str,
    fn(VcrRecorder) -> Arc<dyn Provider>,
);

/// Verify that the `simple_text` timeline follows the universal ordering:
/// start is always first, done is always last, no out-of-order events.
#[test]
fn contract_cross_provider_event_ordering_simple_text() {
    let providers: Vec<ProviderEntry> = vec![
        (
            "anthropic",
            "verify_anthropic_simple_text",
            anthropic_with_vcr,
        ),
        ("openai", "verify_openai_simple_text", openai_with_vcr),
        ("gemini", "verify_gemini_simple_text", gemini_with_vcr),
        ("cohere", "verify_cohere_simple_text", cohere_with_vcr),
    ];

    asupersync::runtime::RuntimeBuilder::current_thread()
        .build()
        .expect("runtime")
        .block_on(async {
            for (name, cassette, ctor) in &providers {
                let Some(vcr) = load_vcr(cassette) else {
                    eprintln!("SKIP: cassette {cassette} not found");
                    continue;
                };
                let provider = ctor(vcr);
                let summary =
                    collect_and_summarize(&*provider, &simple_context(), &default_options()).await;

                // Universal ordering: start must be first
                assert_eq!(
                    summary.timeline.first().map(String::as_str),
                    Some("start"),
                    "{name}: first event must be 'start', got {:?}",
                    summary.timeline.first()
                );

                // Universal ordering: done must be last
                assert_eq!(
                    summary.timeline.last().map(String::as_str),
                    Some("done"),
                    "{name}: last event must be 'done', got {:?}",
                    summary.timeline.last()
                );

                // No error events in a successful scenario
                assert!(
                    !summary.has_error,
                    "{name}: unexpected error event in simple_text"
                );

                // No stream errors
                assert!(
                    summary.stream_error.is_none(),
                    "{name}: unexpected stream error: {:?}",
                    summary.stream_error
                );
            }
        });
}

/// Verify that `tool_call` timelines contain the expected subsequence:
/// `tool_call_start` → `tool_call_delta`* → `tool_call_end`
#[test]
fn contract_cross_provider_tool_call_subsequence() {
    let providers: Vec<ProviderEntry> = vec![
        (
            "anthropic",
            "verify_anthropic_tool_call_single",
            anthropic_with_vcr,
        ),
        ("openai", "verify_openai_tool_call_single", openai_with_vcr),
        ("gemini", "verify_gemini_tool_call_single", gemini_with_vcr),
        ("cohere", "verify_cohere_tool_call_single", cohere_with_vcr),
    ];

    asupersync::runtime::RuntimeBuilder::current_thread()
        .build()
        .expect("runtime")
        .block_on(async {
            for (name, cassette, ctor) in &providers {
                let Some(vcr) = load_vcr(cassette) else {
                    eprintln!("SKIP: cassette {cassette} not found");
                    continue;
                };
                let provider = ctor(vcr);
                let summary =
                    collect_and_summarize(&*provider, &tool_context(), &default_options()).await;

                // Must contain tool_call_end
                assert!(
                    summary.timeline.contains(&"tool_call_end".to_string()),
                    "{name}: timeline missing tool_call_end, got: {:?}",
                    summary.timeline
                );

                // tool_call_end must appear before done
                let end_pos = summary
                    .timeline
                    .iter()
                    .position(|e| e == "tool_call_end")
                    .unwrap();
                let done_pos = summary.timeline.iter().position(|e| e == "done").unwrap();
                assert!(
                    end_pos < done_pos,
                    "{name}: tool_call_end (pos {end_pos}) must come before done (pos {done_pos})"
                );
            }
        });
}

// ═══════════════════════════════════════════════════════════════════════
// Request schema contract: tool definitions round-trip correctly
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn contract_anthropic_tool_schema_has_input_schema() {
    let provider = AnthropicProvider::new("claude-sonnet-4-5");
    let context = tool_context();
    let req = provider.build_request(&context, &default_options());
    let v = serde_json::to_value(&req).expect("serialize");

    let tools = v["tools"].as_array().expect("tools array");
    assert!(!tools.is_empty(), "tools should not be empty");

    for tool in tools {
        // Anthropic contract: tools use `input_schema`, not `parameters`
        assert!(
            tool.get("input_schema").is_some(),
            "Anthropic tool missing input_schema: {tool}"
        );
        assert!(
            tool.get("name").is_some(),
            "Anthropic tool missing name: {tool}"
        );
        assert!(
            tool.get("description").is_some(),
            "Anthropic tool missing description: {tool}"
        );
    }
}

#[test]
fn contract_openai_tool_schema_uses_function_wrapper() {
    let provider = OpenAIProvider::new("gpt-4o");
    let context = tool_context();
    let options = default_options();
    let req = provider.build_request(&context, &options);
    let v = serde_json::to_value(&req).expect("serialize");

    let tools = v["tools"].as_array().expect("tools array");
    assert!(!tools.is_empty(), "tools should not be empty");

    for tool in tools {
        // OpenAI Completions contract: tools use `function` wrapper with `parameters`
        assert_eq!(
            tool.get("type").and_then(Value::as_str),
            Some("function"),
            "OpenAI tool must have type=function: {tool}"
        );
        let func = tool.get("function").expect("function field");
        assert!(
            func.get("name").is_some(),
            "OpenAI function missing name: {func}"
        );
        assert!(
            func.get("parameters").is_some(),
            "OpenAI function missing parameters: {func}"
        );
    }
}

#[test]
fn contract_gemini_tool_schema_uses_function_declarations() {
    let provider = GeminiProvider::new("gemini-1.5-flash");
    let context = tool_context();
    let options = default_options();
    let req = provider.build_request(&context, &options);
    let v = serde_json::to_value(&req).expect("serialize");

    // Gemini contract: tools under `tools[0].functionDeclarations`
    let tools = v["tools"].as_array().expect("tools array");
    assert!(!tools.is_empty(), "tools should not be empty");

    let declarations = tools[0]["functionDeclarations"]
        .as_array()
        .expect("functionDeclarations array");
    assert!(
        !declarations.is_empty(),
        "functionDeclarations should not be empty"
    );

    for decl in declarations {
        assert!(
            decl.get("name").is_some(),
            "Gemini declaration missing name: {decl}"
        );
        assert!(
            decl.get("parameters").is_some(),
            "Gemini declaration missing parameters: {decl}"
        );
    }
}

#[test]
fn contract_cohere_tool_schema_matches_openai_format() {
    let provider = CohereProvider::new("command-r-plus");
    let context = tool_context();
    let options = default_options();
    let req = provider.build_request(&context, &options);
    let v = serde_json::to_value(&req).expect("serialize");

    let tools = v["tools"].as_array().expect("tools array");
    assert!(!tools.is_empty(), "tools should not be empty");

    for tool in tools {
        // Cohere v2 contract: uses same function wrapper as OpenAI
        assert_eq!(
            tool.get("type").and_then(Value::as_str),
            Some("function"),
            "Cohere tool must have type=function: {tool}"
        );
        let func = tool.get("function").expect("function field");
        assert!(
            func.get("name").is_some(),
            "Cohere function missing name: {func}"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Provider identity contract
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn contract_provider_identity_fields_nonempty() {
    let providers: Vec<Box<dyn Provider>> = vec![
        Box::new(AnthropicProvider::new("claude-sonnet-4-5")),
        Box::new(OpenAIProvider::new("gpt-4o")),
        Box::new(GeminiProvider::new("gemini-1.5-flash")),
        Box::new(CohereProvider::new("command-r-plus")),
    ];

    for provider in &providers {
        assert!(
            !provider.name().is_empty(),
            "provider name must not be empty"
        );
        assert!(
            !provider.api().is_empty(),
            "provider api must not be empty for {}",
            provider.name()
        );
        assert!(
            !provider.model_id().is_empty(),
            "provider model_id must not be empty for {}",
            provider.name()
        );
    }
}

#[test]
fn contract_provider_api_types_are_distinct() {
    let anthropic = AnthropicProvider::new("claude-sonnet-4-5");
    let openai = OpenAIProvider::new("gpt-4o");
    let gemini = GeminiProvider::new("gemini-1.5-flash");
    let cohere = CohereProvider::new("command-r-plus");

    // Each native provider must report a distinct API type
    let apis = vec![anthropic.api(), openai.api(), gemini.api(), cohere.api()];
    let mut unique = apis.clone();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(
        apis.len(),
        unique.len(),
        "native providers must have distinct API types: {apis:?}"
    );
}

mod proptests {
    use super::*;
    use proptest::prelude::*;
    use std::collections::HashSet;

    #[derive(Debug, Clone)]
    struct ToolSeed {
        name: String,
        description: String,
        parameters: Value,
    }

    fn schema_type_strategy() -> impl Strategy<Value = &'static str> {
        prop_oneof![
            Just("string"),
            Just("number"),
            Just("integer"),
            Just("boolean")
        ]
    }

    fn tool_seed_strategy() -> impl Strategy<Value = ToolSeed> {
        (
            "[a-z][a-z0-9_]{0,20}",
            "[a-zA-Z0-9][a-zA-Z0-9 _-]{0,47}",
            "[a-z][a-z0-9_]{0,20}",
            schema_type_strategy(),
            "[a-zA-Z0-9 _-]{0,40}",
        )
            .prop_map(|(name, description, field_name, field_type, field_desc)| {
                let required_field = field_name.clone();
                ToolSeed {
                    name,
                    description,
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            field_name: {
                                "type": field_type,
                                "description": field_desc
                            }
                        },
                        "required": [required_field]
                    }),
                }
            })
    }

    fn tool_seeds_strategy() -> impl Strategy<Value = Vec<ToolSeed>> {
        prop::collection::vec(tool_seed_strategy(), 1..6).prop_filter(
            "tool names must be unique",
            |seeds| {
                let mut seen = HashSet::new();
                seeds.iter().all(|seed| seen.insert(seed.name.clone()))
            },
        )
    }

    fn to_tool_defs(seeds: &[ToolSeed]) -> Vec<ToolDef> {
        seeds
            .iter()
            .map(|seed| ToolDef {
                name: seed.name.clone(),
                description: seed.description.clone(),
                parameters: seed.parameters.clone(),
            })
            .collect()
    }

    proptest! {
        #[test]
        fn openai_tool_schema_round_trips_generated_defs(seeds in tool_seeds_strategy()) {
            let provider = OpenAIProvider::new("gpt-4o");
            let defs = to_tool_defs(&seeds);
            let context = context_with_tools(defs.clone());
            let options = default_options();
            let req = provider.build_request(&context, &options);
            let payload = serde_json::to_value(&req).expect("serialize OpenAI request");
            let tools = payload["tools"].as_array().expect("tools array");

            prop_assert_eq!(tools.len(), defs.len());
            for (tool, expected) in tools.iter().zip(defs.iter()) {
                prop_assert_eq!(tool.get("type").and_then(Value::as_str), Some("function"));
                let function = tool.get("function").expect("function wrapper");
                prop_assert_eq!(
                    function.get("name").and_then(Value::as_str),
                    Some(expected.name.as_str())
                );
                prop_assert_eq!(
                    function.get("description").and_then(Value::as_str),
                    Some(expected.description.as_str())
                );
                prop_assert_eq!(function.get("parameters"), Some(&expected.parameters));
            }
        }

        #[test]
        fn cohere_tool_schema_round_trips_generated_defs(seeds in tool_seeds_strategy()) {
            let provider = CohereProvider::new("command-r-plus");
            let defs = to_tool_defs(&seeds);
            let context = context_with_tools(defs.clone());
            let options = default_options();
            let req = provider.build_request(&context, &options);
            let payload = serde_json::to_value(&req).expect("serialize Cohere request");
            let tools = payload["tools"].as_array().expect("tools array");

            prop_assert_eq!(tools.len(), defs.len());
            for (tool, expected) in tools.iter().zip(defs.iter()) {
                prop_assert_eq!(tool.get("type").and_then(Value::as_str), Some("function"));
                let function = tool.get("function").expect("function wrapper");
                prop_assert_eq!(
                    function.get("name").and_then(Value::as_str),
                    Some(expected.name.as_str())
                );
                prop_assert_eq!(
                    function.get("description").and_then(Value::as_str),
                    Some(expected.description.as_str())
                );
                prop_assert_eq!(function.get("parameters"), Some(&expected.parameters));
            }
        }

        #[test]
        fn anthropic_tool_schema_round_trips_generated_defs(seeds in tool_seeds_strategy()) {
            let provider = AnthropicProvider::new("claude-sonnet-4-5");
            let defs = to_tool_defs(&seeds);
            let context = context_with_tools(defs.clone());
            let req = provider.build_request(&context, &default_options());
            let payload = serde_json::to_value(&req).expect("serialize Anthropic request");
            let tools = payload["tools"].as_array().expect("tools array");

            prop_assert_eq!(tools.len(), defs.len());
            for (tool, expected) in tools.iter().zip(defs.iter()) {
                prop_assert_eq!(tool.get("name").and_then(Value::as_str), Some(expected.name.as_str()));
                prop_assert_eq!(
                    tool.get("description").and_then(Value::as_str),
                    Some(expected.description.as_str())
                );
                prop_assert_eq!(tool.get("input_schema"), Some(&expected.parameters));
            }
        }

        #[test]
        fn gemini_tool_schema_round_trips_generated_defs(seeds in tool_seeds_strategy()) {
            let provider = GeminiProvider::new("gemini-1.5-flash");
            let defs = to_tool_defs(&seeds);
            let context = context_with_tools(defs.clone());
            let options = default_options();
            let req = provider.build_request(&context, &options);
            let payload = serde_json::to_value(&req).expect("serialize Gemini request");
            let declarations = payload["tools"][0]["functionDeclarations"]
                .as_array()
                .expect("functionDeclarations array");

            prop_assert_eq!(declarations.len(), defs.len());
            for (declaration, expected) in declarations.iter().zip(defs.iter()) {
                prop_assert_eq!(
                    declaration.get("name").and_then(Value::as_str),
                    Some(expected.name.as_str())
                );
                prop_assert_eq!(
                    declaration.get("description").and_then(Value::as_str),
                    Some(expected.description.as_str())
                );
                prop_assert_eq!(declaration.get("parameters"), Some(&expected.parameters));
            }
        }
    }
}
