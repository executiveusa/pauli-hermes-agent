#![no_main]

//! Fuzz harness for edit-tool matching and replacement paths.
//!
//! Covers arbitrary content plus targeted edge cases (ambiguous matches,
//! empty old text) against real tool execution.

use arbitrary::{Arbitrary, Unstructured};
use futures::executor::block_on;
use libfuzzer_sys::fuzz_target;
use pi::tools::{EditTool, Tool};
use serde_json::json;
use tempfile::tempdir;

const MAX_FILE_BYTES: usize = 8 * 1024;
const MAX_EDIT_BYTES: usize = 1024;

#[derive(Arbitrary, Debug)]
struct EditCase {
    file_content: String,
    old_text: String,
    new_text: String,
}

fn clamp_bytes(value: &str, max_bytes: usize) -> String {
    let mut bytes = value.as_bytes().to_vec();
    if bytes.len() > max_bytes {
        bytes.truncate(max_bytes);
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

fuzz_target!(|data: &[u8]| {
    let mut unstructured = Unstructured::new(data);
    let Ok(case) = EditCase::arbitrary(&mut unstructured) else {
        return;
    };

    let file_content = clamp_bytes(&case.file_content, MAX_FILE_BYTES);
    let old_text = clamp_bytes(&case.old_text, MAX_EDIT_BYTES);
    let new_text = clamp_bytes(&case.new_text, MAX_EDIT_BYTES);

    let Ok(tmp) = tempdir() else {
        return;
    };
    let edit_tool = EditTool::new(tmp.path());

    let _ = std::fs::write(tmp.path().join("target.txt"), &file_content);

    let _ = block_on(edit_tool.execute(
        "edit-fuzz-main",
        json!({
            "path": "target.txt",
            "oldText": old_text.clone(),
            "newText": new_text.clone()
        }),
        None,
    ));

    // Force a common ambiguous-match shape (same old text repeated).
    let repeated_old = if old_text.is_empty() {
        "x".to_string()
    } else {
        old_text.clone()
    };
    let ambiguous_content = format!("{repeated_old}\n{repeated_old}\n");
    let _ = std::fs::write(tmp.path().join("ambiguous.txt"), ambiguous_content);
    let _ = block_on(edit_tool.execute(
        "edit-fuzz-ambiguous",
        json!({
            "path": "ambiguous.txt",
            "oldText": repeated_old,
            "newText": "replacement"
        }),
        None,
    ));

    // Empty-old edge case.
    let _ = block_on(edit_tool.execute(
        "edit-fuzz-empty-old",
        json!({
            "path": "target.txt",
            "oldText": "",
            "newText": "x"
        }),
        None,
    ));
});
