#![no_main]

//! Fuzz harness for `Message` serde round-trip invariants.
//!
//! Invariant:
//! valid JSON -> `Message` -> JSON -> `Message` preserves semantic shape.

use libfuzzer_sys::fuzz_target;
use pi::fuzz_exports::Message;

fn roundtrip_message(input: &str) {
    let Ok(message) = serde_json::from_str::<Message>(input) else {
        return;
    };

    let Ok(serialized) = serde_json::to_string(&message) else {
        return;
    };

    let Ok(reparsed) = serde_json::from_str::<Message>(&serialized) else {
        return;
    };

    let Ok(left) = serde_json::to_value(&message) else {
        return;
    };
    let Ok(right) = serde_json::to_value(&reparsed) else {
        return;
    };

    assert_eq!(left, right);
}

fuzz_target!(|data: &[u8]| {
    let lossy = String::from_utf8_lossy(data);

    // Whole payload.
    roundtrip_message(&lossy);

    // JSONL-style line parsing catches edge cases around truncation/newlines.
    for line in lossy.lines().take(128) {
        roundtrip_message(line.trim_end_matches('\r'));
    }

    // BOM-prefixed variant.
    let mut bom_prefixed = String::from("\u{feff}");
    bom_prefixed.push_str(&lossy);
    roundtrip_message(&bom_prefixed);
});
