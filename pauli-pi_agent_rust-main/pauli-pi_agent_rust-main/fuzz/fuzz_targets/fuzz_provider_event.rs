//! Coverage-guided libfuzzer harness for provider stream event processing.
//!
//! Exercises `process_event()` across all 7 provider implementations
//! (Anthropic, OpenAI, Gemini, Cohere, OpenAI Responses, Azure, Vertex)
//! with arbitrary JSON strings and event sequences.
//!
//! The harness covers both single-event and multi-event sequences to
//! catch state-dependent bugs (e.g., content_block_delta before
//! content_block_start).

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use pi::fuzz_exports::{
    AnthropicProcessor, AzureProcessor, CohereProcessor, GeminiProcessor, OpenAIProcessor,
    OpenAIResponsesProcessor, VertexProcessor,
};

/// Provider selector — maps to one of the 7 implementations.
#[derive(Arbitrary, Debug, Clone, Copy)]
#[repr(u8)]
enum Provider {
    Anthropic,
    OpenAI,
    Gemini,
    Cohere,
    OpenAIResponses,
    Azure,
    Vertex,
}

/// Fuzz input: a provider selection and a sequence of event payloads.
///
/// Using a `Vec<String>` instead of a single string exercises
/// state-dependent code paths (e.g., content block lifecycle).
#[derive(Arbitrary, Debug)]
struct FuzzInput {
    provider: Provider,
    events: Vec<String>,
}

fuzz_target!(|input: FuzzInput| {
    // Skip excessively long sequences to bound execution time.
    if input.events.len() > 32 {
        return;
    }
    // Skip excessively large payloads to avoid OOM.
    let total_bytes: usize = input.events.iter().map(String::len).sum();
    if total_bytes > 64 * 1024 {
        return;
    }

    match input.provider {
        Provider::Anthropic => fuzz_anthropic(&input.events),
        Provider::OpenAI => fuzz_openai(&input.events),
        Provider::Gemini => fuzz_gemini(&input.events),
        Provider::Cohere => fuzz_cohere(&input.events),
        Provider::OpenAIResponses => fuzz_openai_responses(&input.events),
        Provider::Azure => fuzz_azure(&input.events),
        Provider::Vertex => fuzz_vertex(&input.events),
    }
});

fn fuzz_anthropic(events: &[String]) {
    let mut proc = AnthropicProcessor::new();
    for event in events {
        let _ = proc.process_event(event);
    }
}

fn fuzz_openai(events: &[String]) {
    let mut proc = OpenAIProcessor::new();
    for event in events {
        let _ = proc.process_event(event);
    }
}

fn fuzz_gemini(events: &[String]) {
    let mut proc = GeminiProcessor::new();
    for event in events {
        let _ = proc.process_event(event);
    }
}

fn fuzz_cohere(events: &[String]) {
    let mut proc = CohereProcessor::new();
    for event in events {
        let _ = proc.process_event(event);
    }
}

fn fuzz_openai_responses(events: &[String]) {
    let mut proc = OpenAIResponsesProcessor::new();
    for event in events {
        let _ = proc.process_event(event);
    }
}

fn fuzz_azure(events: &[String]) {
    let mut proc = AzureProcessor::new();
    for event in events {
        let _ = proc.process_event(event);
    }
}

fn fuzz_vertex(events: &[String]) {
    let mut proc = VertexProcessor::new();
    for event in events {
        let _ = proc.process_event(event);
    }
}
