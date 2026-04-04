#![no_main]

//! Fuzz harness for `SseParser` — coverage-guided fuzzing of `feed()`/`flush()`.
//!
//! **Invariant under test:** Parsing a complete SSE stream in one call vs.
//! feeding it character-by-character MUST produce the same events (the
//! "chunking invariant" from the proptest suite, now under libFuzzer guidance).

use libfuzzer_sys::fuzz_target;
use pi::fuzz_exports::SseParser;

fuzz_target!(|data: &[u8]| {
    // Convert to UTF-8 lossily — SseParser works on &str, not raw bytes.
    let input = String::from_utf8_lossy(data);

    // --- Strategy 1: Feed the entire input at once ---
    let mut parser_whole = SseParser::new();
    let events_whole = parser_whole.feed(&input);
    let flush_whole = parser_whole.flush();

    // --- Strategy 2: Feed byte-by-byte (char-by-char) ---
    let mut parser_char = SseParser::new();
    let mut events_char: Vec<_> = Vec::new();
    for ch in input.chars() {
        let mut buf = [0u8; 4];
        let s = ch.encode_utf8(&mut buf);
        events_char.extend(parser_char.feed(s));
    }
    let flush_char = parser_char.flush();

    // --- Strategy 3: Feed in two chunks split at a valid char boundary ---
    // We split the already-converted string (not raw bytes) to avoid
    // from_utf8_lossy producing different replacement chars at split points.
    if input.len() >= 2 {
        let mid = input.len() / 2;
        // Find a valid char boundary at or after midpoint
        let mut split_at = mid;
        while !input.is_char_boundary(split_at) && split_at < input.len() {
            split_at += 1;
        }
        let (part1, part2) = input.split_at(split_at);
        let mut parser_split = SseParser::new();
        let mut events_split: Vec<_> = parser_split.feed(part1);
        events_split.extend(parser_split.feed(part2));
        let flush_split = parser_split.flush();

        // Chunking invariant: whole == split
        assert_eq!(
            events_whole.len(),
            events_split.len(),
            "Event count mismatch: whole({}) vs split({})",
            events_whole.len(),
            events_split.len(),
        );
        for (i, (w, s)) in events_whole.iter().zip(events_split.iter()).enumerate() {
            assert_eq!(w, s, "Event {i} differs between whole-feed and split-feed");
        }
        assert_eq!(
            flush_whole, flush_split,
            "Flush differs between whole-feed and split-feed"
        );
    }

    // Chunking invariant: whole == char-by-char
    assert_eq!(
        events_whole.len(),
        events_char.len(),
        "Event count mismatch: whole({}) vs char-by-char({})",
        events_whole.len(),
        events_char.len(),
    );
    for (i, (w, c)) in events_whole.iter().zip(events_char.iter()).enumerate() {
        assert_eq!(w, c, "Event {i} differs between whole-feed and char-feed");
    }
    assert_eq!(
        flush_whole, flush_char,
        "Flush differs between whole-feed and char-feed"
    );
});
