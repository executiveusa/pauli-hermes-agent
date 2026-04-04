#![no_main]

//! Fuzz harness for SSE byte-level processing — tests UTF-8 decoding edge
//! cases that `SseStream` encounters when receiving raw bytes from the wire.
//!
//! Unlike `fuzz_sse_parser` (which operates on valid UTF-8 `&str`), this
//! harness feeds **raw arbitrary bytes** including intentionally malformed
//! UTF-8 sequences, testing the UTF-8 recovery paths.

use libfuzzer_sys::fuzz_target;
use pi::fuzz_exports::SseParser;

/// Simulate `SseStream`'s UTF-8 processing logic on raw bytes.
///
/// This mirrors the approach in `SseStream::process_chunk()` — accumulate
/// bytes, decode valid UTF-8 prefixes, buffer incomplete trailing sequences.
fn process_bytes_through_parser(data: &[u8]) -> (Vec<String>, Option<String>) {
    let mut parser = SseParser::new();
    let mut all_events = Vec::new();
    let mut utf8_buffer: Vec<u8> = Vec::new();

    // Split input into chunks of varying sizes (1..=64 bytes).
    let mut offset = 0;
    let mut chunk_size = 1;
    while offset < data.len() {
        let end = (offset + chunk_size).min(data.len());
        let chunk = &data[offset..end];
        offset = end;
        chunk_size = (chunk_size % 64) + 1; // Vary chunk sizes: 1,2,3,...,64,1,...

        // Append chunk to utf8_buffer
        utf8_buffer.extend_from_slice(chunk);

        // Try to decode as much valid UTF-8 as possible
        match std::str::from_utf8(&utf8_buffer) {
            Ok(s) => {
                let events = parser.feed(s);
                for e in events {
                    all_events.push(e.data.clone());
                }
                utf8_buffer.clear();
            }
            Err(err) => {
                let valid_len = err.valid_up_to();
                if valid_len > 0 {
                    // Feed the valid prefix
                    let valid = std::str::from_utf8(&utf8_buffer[..valid_len]).unwrap();
                    let events = parser.feed(valid);
                    for e in events {
                        all_events.push(e.data.clone());
                    }
                }

                if let Some(invalid_len) = err.error_len() {
                    // Hard error: skip invalid bytes, keep remainder
                    utf8_buffer.drain(..valid_len + invalid_len);
                } else {
                    // Incomplete sequence at end: keep only the tail bytes
                    utf8_buffer.drain(..valid_len);
                }
            }
        }
    }

    // Process any remaining valid UTF-8 in the buffer
    if !utf8_buffer.is_empty() {
        if let Ok(s) = std::str::from_utf8(&utf8_buffer) {
            let events = parser.feed(s);
            for e in events {
                all_events.push(e.data.clone());
            }
        }
    }

    // Flush
    let flush_data = parser.flush().map(|e| e.data);
    (all_events, flush_data)
}

fuzz_target!(|data: &[u8]| {
    // Main test: process arbitrary bytes through UTF-8 decoding + SSE parsing.
    // Must not panic on any input.
    let (_events, _flush) = process_bytes_through_parser(data);

    // Bonus: also test with the entire input as one chunk via lossy conversion
    let lossy = String::from_utf8_lossy(data);
    let mut parser = SseParser::new();
    let _ = parser.feed(&lossy);
    let _ = parser.flush();
});
