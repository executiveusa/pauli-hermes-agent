#[cfg(test)]
mod tests {
    use pi::tools::{EditTool, Tool};
    use serde_json::json;

    #[test]
    fn repro_edit_off_by_one_on_subsequent_lines() {
        asupersync::test_utils::run_test(|| async {
            let dir = tempfile::tempdir().unwrap();
            let file_path = dir.path().join("test.txt");

            // Setup: 2 lines. First line has length 5 + 1 newline.
            // Line 2 starts at index 6.
            let original_content = "line1\nline2";
            std::fs::write(&file_path, original_content).unwrap();

            let tool = EditTool::new(dir.path());

            // Try to replace "line2" on the second line.
            // If bug exists, it will delete '\n' from line 1 and leave last char of line 2.
            // Logic:
            // "line1" (5) + '\n' (1) = 6 chars.
            // map() iterates line1 (5 chars).
            // Misses '\n' count in norm_idx and orig_idx.
            // Line 2 starts. norm_idx=5. orig_idx=5.
            // Matches 'l' (index 5 in norm).
            // Returns match_start = orig_idx (5) + char_offset (0) = 5.
            // Index 5 is '\n'.
            // Replace range [5, 5 + len("line2")]. [5, 10].
            // "line1\nline2" (len 11).
            // Indices:
            // l i n e 1 \n l i n e 2
            // 0 1 2 3 4 5  6 7 8 9 10
            // Range [5, 10]: "\nline".
            // Replaced with "fixed".
            // Result: "line1" + "fixed" + "2". -> "line1fixed2".

            let input = json!({
                "path": "test.txt",
                "oldText": "line2",
                "newText": "fixed"
            });

            let result = tool.execute("call_1", input, None).await.unwrap();
            assert!(!result.is_error, "Tool execution failed: {result:?}");

            let new_content = std::fs::read_to_string(&file_path).unwrap();
            assert_eq!(
                new_content, "line1\nfixed",
                "Content mismatch. Got: {new_content:?}"
            );
        });
    }
}
