#[cfg(test)]
mod tests {
    use pi::sse::SseParser;

    #[test]
    fn test_bom_fragmented() {
        let mut parser = SseParser::new();
        // Feed BOM in parts: EF, BB, BF
        let _events1 = parser.feed("\u{FEFF}".chars().take(1).collect::<String>().as_str()); // This is tricky, unicode chars.
        // \u{FEFF} is 3 bytes: EF BB BF.
        // We need to feed bytes that form invalid UTF-8 strings if we want to simulate byte-level fragmentation,
        // but SseParser::feed takes &str.
        // Ah, SseStream handles UTF-8 reassembly. SseParser assumes valid UTF-8 strings.
        // However, it IS possible for a valid UTF-8 string to be fed in chunks such that the BOM is split?
        // No, because BOM is a single code point. A single code point cannot be split across &str chunks.

        // WAIT. SseParser::feed takes &str.
        // If SseStream assembles UTF-8, it will wait until it has the full 3 bytes of BOM before calling feed with the BOM char.
        // So `parser.feed` will receive the full BOM character in the first call (if it's the start).

        // Let's look at SseStream again.
        // It buffers bytes in `utf8_buffer` until `std::str::from_utf8` succeeds.
        // So if we send [0xEF], `from_utf8` fails. `process_chunk` keeps it in `utf8_buffer`.
        // Next [0xBB], still fails.
        // Next [0xBF], `from_utf8` succeeds with "\u{FEFF}".
        // calls `parser.feed("\u{FEFF}")`.

        // So `SseParser` will see the full BOM.

        // HOWEVER, what if we feed "\u{FEFF}" (empty data) then "data: hello"?
        // feed("\u{FEFF}") -> buffer = "\u{FEFF}". strip_prefix works. buffer becomes empty. bom_checked = true.
        // Next feed("data: ...") -> buffer = "data: ...". bom_checked is true. No strip. Correct.

        // What if we feed "" (empty string)?
        // feed("") -> buffer empty. `!buffer.is_empty()` is false. bom_checked remains false.
        // Next feed("\u{FEFF}...") -> buffer has BOM. `!buffer.is_empty()` is true. bom_checked=true. strip succeeds. Correct.

        // Is there ANY case where `SseParser` receives partial BOM?
        // Only if the input `&str` somehow contains partial BOM? Impossible, `&str` must be valid UTF-8.

        // So the "BOM fragmentation" bug I suspected is actually IMPOSSIBLE at the `SseParser` level because it takes `&str`.
        // The `SseStream` handles the byte-level reassembly.

        // Is there a bug in `SseStream` regarding BOM?
        // If `SseStream` gets [0xEF, 0xBB, 0xBF, 'd', 'a', 't', 'a'], it converts to string "\u{FEFF}data".
        // Feeds to parser. Parser strips BOM.

        // So my "confirmed bug" was a hallucination based on thinking `feed` took bytes or that `&str` could split a char.
    }
}
