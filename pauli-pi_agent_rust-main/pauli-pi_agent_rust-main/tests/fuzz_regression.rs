use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use pi::sse::SseParser;

fn list_dir(path: &Path) -> io::Result<Vec<PathBuf>> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(path)? {
        entries.push(entry?.path());
    }
    entries.sort();
    Ok(entries)
}

fn assert_sse_chunking_invariant(data: &[u8]) {
    let input = String::from_utf8_lossy(data);

    let mut parser_whole = SseParser::new();
    let events_whole = parser_whole.feed(&input);
    let flush_whole = parser_whole.flush();

    let mut parser_char = SseParser::new();
    let mut events_char = Vec::new();
    for ch in input.chars() {
        let mut buf = [0u8; 4];
        let s = ch.encode_utf8(&mut buf);
        events_char.extend(parser_char.feed(s));
    }
    let flush_char = parser_char.flush();

    // Split the already-converted string at a valid char boundary (not raw
    // bytes) to avoid from_utf8_lossy producing different replacement chars
    // at the split point vs the whole-input parse.
    if input.len() >= 2 {
        let mid = input.len() / 2;
        let mut split_at = mid;
        while !input.is_char_boundary(split_at) && split_at < input.len() {
            split_at += 1;
        }
        let (part1, part2) = input.split_at(split_at);
        let mut parser_split = SseParser::new();
        let mut events_split = parser_split.feed(part1);
        events_split.extend(parser_split.feed(part2));
        let flush_split = parser_split.flush();

        assert_eq!(
            events_whole.len(),
            events_split.len(),
            "whole/split event count mismatch",
        );
        for (idx, (whole, split)) in events_whole.iter().zip(events_split.iter()).enumerate() {
            assert_eq!(whole, split, "whole/split event mismatch at index {idx}");
        }
        assert_eq!(flush_whole, flush_split, "whole/split flush mismatch");
    }

    assert_eq!(
        events_whole.len(),
        events_char.len(),
        "whole/char event count mismatch",
    );
    for (idx, (whole, ch)) in events_whole.iter().zip(events_char.iter()).enumerate() {
        assert_eq!(whole, ch, "whole/char event mismatch at index {idx}");
    }
    assert_eq!(flush_whole, flush_char, "whole/char flush mismatch");
}

#[test]
fn regression_inputs_have_target_assertions() -> io::Result<()> {
    let regression_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("fuzz/regression");
    if !regression_root.exists() {
        return Ok(());
    }

    let mut exercised = 0usize;
    for target_dir in list_dir(&regression_root)? {
        if !target_dir.is_dir() {
            continue;
        }
        let target = target_dir
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_string();
        for candidate in list_dir(&target_dir)? {
            if candidate.extension().and_then(|ext| ext.to_str()) != Some("bin") {
                continue;
            }
            let data = fs::read(&candidate)?;
            match target.as_str() {
                "fuzz_smoke" => {
                    assert!(
                        !data.is_empty(),
                        "regression input must not be empty: {}",
                        candidate.display()
                    );
                }
                "fuzz_sse_parser" => assert_sse_chunking_invariant(&data),
                _ => panic!(
                    "missing regression target handler for {} (file {})",
                    target,
                    candidate.display()
                ),
            }
            exercised += 1;
        }
    }

    assert!(
        exercised > 0,
        "expected at least one .bin regression input in {}",
        regression_root.display()
    );
    Ok(())
}
