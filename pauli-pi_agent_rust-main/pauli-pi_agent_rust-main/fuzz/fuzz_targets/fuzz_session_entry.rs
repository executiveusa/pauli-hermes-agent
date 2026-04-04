#![no_main]

//! Fuzz harness for standalone `SessionEntry` JSON deserialization.
//!
//! Focuses on malformed/ambiguous JSON snippets and lightweight round-trip checks
//! for successfully parsed entries.

use libfuzzer_sys::fuzz_target;
use pi::fuzz_exports::SessionEntry;

fn fuzz_entry(input: &str) {
    let parsed = serde_json::from_str::<SessionEntry>(input);

    if let Ok(entry) = parsed {
        // Re-serialize and deserialize to keep serde paths hot and ensure
        // type-tag stability for valid entries.
        if let Ok(serialized) = serde_json::to_string(&entry) {
            if let Ok(reparsed) = serde_json::from_str::<SessionEntry>(&serialized) {
                assert_eq!(
                    std::mem::discriminant(&entry),
                    std::mem::discriminant(&reparsed)
                );
            }
        }
    }
}

fuzz_target!(|data: &[u8]| {
    let lossy = String::from_utf8_lossy(data);

    // Whole payload.
    fuzz_entry(&lossy);

    // Line-oriented parsing to emulate JSONL corruption patterns.
    for line in lossy.lines().take(64) {
        fuzz_entry(line.trim_end_matches('\r'));
    }

    // BOM-prefixed variant.
    let mut bom_prefixed = String::from("\u{feff}");
    bom_prefixed.push_str(&lossy);
    fuzz_entry(&bom_prefixed);
});
