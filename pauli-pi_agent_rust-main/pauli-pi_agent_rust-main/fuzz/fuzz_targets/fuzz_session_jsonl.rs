#![no_main]

//! Fuzz harness for session JSONL loading (`Session::open_with_diagnostics`).
//!
//! Exercises both:
//! 1. Header parsing + file-level error handling with raw arbitrary bytes.
//! 2. Per-line `SessionEntry` decode paths by prepending a valid session header.

use futures::executor::block_on;
use libfuzzer_sys::fuzz_target;
use pi::fuzz_exports::Session;
use std::path::Path;
use tempfile::tempdir;

const SESSION_HEADER: &str = r#"{"type":"session","version":3,"id":"fuzz-seed","timestamp":"2026-02-14T00:00:00.000Z","cwd":"/tmp/pi-fuzz"}"#;
const MAX_BODY_BYTES: usize = 512 * 1024;

fn try_open_session(path: &Path) {
    let path_string = path.to_string_lossy().into_owned();
    let _ = block_on(async { Session::open_with_diagnostics(&path_string).await });
}

fn write_and_open(path: &Path, bytes: &[u8]) {
    if std::fs::write(path, bytes).is_ok() {
        try_open_session(path);
    }
}

fuzz_target!(|data: &[u8]| {
    // Bound per-input disk work for better fuzz iteration throughput.
    if data.len() > MAX_BODY_BYTES {
        return;
    }

    let Ok(dir) = tempdir() else {
        return;
    };

    // Variant 1: Raw bytes (full header/path error surface).
    let raw_path = dir.path().join("raw_input.jsonl");
    write_and_open(&raw_path, data);

    // Variant 2: Valid header + arbitrary body (line-level SessionEntry parsing path).
    let body = String::from_utf8_lossy(data);
    let mut normalized = String::with_capacity(SESSION_HEADER.len() + body.len() + 1);
    normalized.push_str(SESSION_HEADER);
    normalized.push('\n');
    normalized.push_str(&body);
    let normalized_path = dir.path().join("normalized_body.jsonl");
    write_and_open(&normalized_path, normalized.as_bytes());

    // Variant 3: Same body with CRLF normalization.
    let mut crlf = String::with_capacity(SESSION_HEADER.len() + body.len() + 2);
    crlf.push_str(SESSION_HEADER);
    crlf.push_str("\r\n");
    crlf.push_str(&body.replace('\n', "\r\n"));
    let crlf_path = dir.path().join("crlf_body.jsonl");
    write_and_open(&crlf_path, crlf.as_bytes());
});
