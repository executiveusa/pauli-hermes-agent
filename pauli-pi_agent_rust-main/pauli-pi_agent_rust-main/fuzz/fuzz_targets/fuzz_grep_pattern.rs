#![no_main]

//! Fuzz harness for grep tool pattern inputs.
//!
//! Exercises regex and literal search paths with bounded input sizes so
//! libFuzzer throughput remains usable.

use futures::executor::block_on;
use libfuzzer_sys::fuzz_target;
use pi::tools::{GrepTool, Tool};
use serde_json::json;
use tempfile::tempdir;

const MAX_TOTAL_BYTES: usize = 8 * 1024;
const MAX_PATTERN_CHARS: usize = 256;
const MAX_CONTENT_CHARS: usize = 2048;

fn lossy_limited(input: &[u8], max_chars: usize) -> String {
    String::from_utf8_lossy(input)
        .chars()
        .take(max_chars)
        .collect()
}

fuzz_target!(|data: &[u8]| {
    if data.len() < 2 || data.len() > MAX_TOTAL_BYTES {
        return;
    }

    // Keep per-iteration subprocess cost bounded.
    if data[0] % 4 != 0 {
        return;
    }

    let split = data.len() / 2;
    let pattern = lossy_limited(&data[..split], MAX_PATTERN_CHARS);
    if pattern.is_empty() {
        return;
    }
    let content = lossy_limited(&data[split..], MAX_CONTENT_CHARS);

    let Ok(tmp) = tempdir() else {
        return;
    };
    let _ = std::fs::write(tmp.path().join("fixture.txt"), content);

    let grep = GrepTool::new(tmp.path());

    let _ = block_on(grep.execute(
        "grep-fuzz-regex",
        json!({
            "pattern": pattern.clone(),
            "path": ".",
            "limit": 8,
            "context": 1
        }),
        None,
    ));

    let _ = block_on(grep.execute(
        "grep-fuzz-literal",
        json!({
            "pattern": pattern,
            "path": ".",
            "literal": true,
            "limit": 8,
            "context": 0
        }),
        None,
    ));
});
