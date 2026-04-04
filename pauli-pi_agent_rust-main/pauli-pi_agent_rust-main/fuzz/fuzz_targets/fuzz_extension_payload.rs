#![no_main]

//! Fuzz harness for extension protocol payload parsing and validation.
//!
//! Exercises `ExtensionMessage::parse_and_validate` and related serde payload
//! structs used by JS<->host protocol boundaries.

use libfuzzer_sys::fuzz_target;
use pi::extensions::{
    ErrorPayload, EventHookPayload, ExtensionMessage, HostCallPayload, HostResultPayload,
    LogPayload, RegisterPayload, SlashCommandPayload, SlashResultPayload, ToolCallPayload,
    ToolResultPayload,
};
use pi::extensions_js::ExtensionToolDef;

const MAX_INPUT_BYTES: usize = 128 * 1024;

fn fuzz_json(input: &str) {
    if input.is_empty() {
        return;
    }

    let _ = serde_json::from_str::<ExtensionMessage>(input);
    let _ = ExtensionMessage::parse_and_validate(input);

    let _ = serde_json::from_str::<RegisterPayload>(input);
    let _ = serde_json::from_str::<ToolCallPayload>(input);
    let _ = serde_json::from_str::<ToolResultPayload>(input);
    let _ = serde_json::from_str::<SlashCommandPayload>(input);
    let _ = serde_json::from_str::<SlashResultPayload>(input);
    let _ = serde_json::from_str::<EventHookPayload>(input);
    let _ = serde_json::from_str::<HostCallPayload>(input);
    let _ = serde_json::from_str::<HostResultPayload>(input);
    let _ = serde_json::from_str::<LogPayload>(input);
    let _ = serde_json::from_str::<ErrorPayload>(input);
    let _ = serde_json::from_str::<ExtensionToolDef>(input);
}

fuzz_target!(|data: &[u8]| {
    if data.is_empty() || data.len() > MAX_INPUT_BYTES {
        return;
    }

    let lossy = String::from_utf8_lossy(data);

    // Whole payload.
    fuzz_json(&lossy);

    // Line-oriented parsing for JSONL-ish corpus and truncated fragments.
    for line in lossy.lines().take(256) {
        fuzz_json(line.trim_end_matches('\r'));
    }

    // BOM-prefixed variant.
    let mut bom_prefixed = String::from("\u{feff}");
    bom_prefixed.push_str(&lossy);
    fuzz_json(&bom_prefixed);
});
