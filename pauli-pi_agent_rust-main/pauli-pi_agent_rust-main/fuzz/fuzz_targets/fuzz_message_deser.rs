#![no_main]

//! Fuzz harness for message/content serde entry points.
//!
//! Focuses on untrusted JSON for:
//! - `Message` (tagged enum)
//! - `UserContent` (untagged enum)
//! - `ContentBlock`
//! - `AssistantMessage`
//! - `ToolCall`

use libfuzzer_sys::fuzz_target;
use pi::fuzz_exports::{AssistantMessage, ContentBlock, Message, ToolCall, UserContent};

fn fuzz_json(input: &str) {
    let _ = serde_json::from_str::<Message>(input);
    let _ = serde_json::from_str::<UserContent>(input);
    let _ = serde_json::from_str::<ContentBlock>(input);
    let _ = serde_json::from_str::<AssistantMessage>(input);
    let _ = serde_json::from_str::<ToolCall>(input);
}

fuzz_target!(|data: &[u8]| {
    let lossy = String::from_utf8_lossy(data);

    // Whole payload.
    fuzz_json(&lossy);

    // JSONL-style line parsing for truncated/corrupt lines.
    for line in lossy.lines().take(128) {
        fuzz_json(line.trim_end_matches('\r'));
    }

    // BOM-prefixed variant (common copy/paste corruption).
    let mut bom_prefixed = String::from("\u{feff}");
    bom_prefixed.push_str(&lossy);
    fuzz_json(&bom_prefixed);
});
